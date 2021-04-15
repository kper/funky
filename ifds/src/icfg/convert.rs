/// This module is responsible to parse
/// the webassembly AST to a graph
use crate::icfg::graph::*;
use crate::icfg::state::State;
use crate::ir::ast::Function as AstFunction;
use crate::ir::ast::Instruction;

use crate::{counter::Counter, solver::Request};
use anyhow::{bail, Context, Result};
use std::collections::VecDeque;

use log::debug;

use std::collections::HashMap;

use crate::icfg::flowfuncs::{BlockResolver, InitialFlowFunction, NormalFlowFunction};
use crate::ir::ast::Program;

const TAUT: usize = 1;

/// Central datastructure for the computation of the IFDS problem.
#[derive(Debug)]
pub struct ConvertSummary<I, F>
where
    I: InitialFlowFunction,
    F: NormalFlowFunction,
{
    block_counter: Counter,
    block_resolver: BlockResolver,
    init_flow: I,
    normal_flow: F,
}

impl<I, F> ConvertSummary<I, F>
where
    I: InitialFlowFunction,
    F: NormalFlowFunction,
{
    pub fn new(init_flow: I, flow: F) -> Self {
        Self {
            block_counter: Counter::default(),
            block_resolver: HashMap::new(),
            init_flow,
            normal_flow: flow,
        }
    }

    /// Computes a graph by a given program and `req` ([`Request`]).
    /// The `variable` in `req` doesn't matter. It only matters the `function` and `pc`.
    pub fn visit(&mut self, prog: &Program, req: &Request) -> Result<(Graph, State)> {
        let mut graph = Graph::default();
        let mut state = State::default();

        self.tabulate(&mut graph, prog, req, &mut state)?;

        Ok((graph, state))
    }

    /// Computes call-to-start edges
    pub(crate) fn pass_args(
        &mut self,
        caller_function: &AstFunction,
        callee_function: &AstFunction,
        params: &Vec<String>,
        graph: &mut Graph,
        current_pc: usize,
        caller_var: &String,
        normal_flows_debug: &mut Vec<Edge>,
        path_edge: &mut Vec<Edge>,
        worklist: &mut VecDeque<Edge>,
        state: &mut State,
    ) -> Result<Vec<Edge>> {
        let caller_variable = state
            .get_var(&caller_function.name, caller_var)
            .context("Variable is not defined")?
            .clone();

        // Why not dests? Because we don't care about
        // the destination for the function call in
        // `pass_args`
        if params.contains(&caller_var)
            || caller_variable.is_taut
            || caller_variable.is_global
            || caller_variable.is_memory
        {
            // Init facts of the called function
            // Start from the beginning.
            let start_pc = 0;
            let init_facts = state
                .init_function(&callee_function, start_pc)
                .context("Error during function init")?;

            self.pacemaker(
                callee_function,
                graph,
                path_edge,
                worklist,
                normal_flows_debug,
                &init_facts,
                state,
            )
            .context("Pacemaker for pass_args failed")?;

            // Create all params
            state.cache_facts(&callee_function.name, init_facts.clone())?;

            // Save all blocks of the `callee_function`.
            // Because we want to jump to them later.
            self.resolve_block_ids(&callee_function, start_pc)?;

            // Filter by variable type
            let callee_globals = init_facts.iter().filter(|x| x.var_is_global).count();

            // Get the position in the parameters. If it does not exist then
            // it is `taut`.
            let pos_in_param = params
                .iter()
                .position(|x| x == caller_var)
                .map(|x| x + TAUT)
                .unwrap_or(callee_globals); // because, globals are before the parameters

            let callee_offset = match (caller_variable.is_taut, caller_variable.is_global) {
                (true, _) => 0,
                (false, false) => callee_globals + pos_in_param, // if not global, than start at normal beginning
                (false, true) => pos_in_param,                   //look for the global
            };

            let callee_fact = init_facts
                .get(callee_offset)
                .context("Cannot find callee's fact")?;

            let mut edges = vec![];

            if caller_variable.is_memory {
                let memory_edges = self
                    .pass_args_memory(
                        &caller_function,
                        &callee_function,
                        &caller_variable,
                        current_pc,
                        state,
                    )
                    .context("Passing memory variables to a called function failed")?;

                edges.extend(memory_edges);

                return Ok(edges);
            }

            // Last caller facts
            debug!(
                "caller {} with current_pc {}",
                caller_function.name, current_pc
            );

            let mut caller_facts = state.get_facts_at(&caller_function.name, current_pc)?;

            // Filter by variable
            let caller_fact = caller_facts
                .find(|x| &x.belongs_to_var == caller_var)
                .with_context(|| {
                    format!(
                        "Cannot find caller's fact {} for \"{}\" at {}",
                        caller_var, caller_function.name, current_pc
                    )
                })?;

            // The corresponding edges have to match now, but filter `dest`.
            // taut -> taut
            // %0   -> %0
            // %1   -> %1

            // Create an edge.
            edges.push(Edge::Call {
                from: caller_fact.clone().clone(),
                to: callee_fact.clone(),
            });

            Ok(edges)
        } else {
            debug!(
                "Caller's variable is not a parameter {} in {:?} for {}",
                caller_var, params, callee_function.name
            );

            return Ok(vec![]);
        }
    }

    /// Handle the parameter argument handling for memory variables
    fn pass_args_memory(
        &mut self,
        caller_function: &AstFunction,
        callee_function: &AstFunction,
        caller_variable: &Variable,
        current_pc: usize,
        state: &mut State,
    ) -> Result<Vec<Edge>> {
        let start_pc = 0;

        let caller_facts: Vec<_> = state
            .get_facts_at(&caller_function.name, current_pc)?
            .filter(|x| x.var_is_memory && caller_variable.name == x.belongs_to_var)
            .cloned()
            .collect();

        let callee_facts: Vec<_> = state
            .get_facts_at(&callee_function.name, start_pc)?
            .filter(|x| x.var_is_memory && caller_variable.name == x.belongs_to_var)
            .cloned()
            .collect();

        let mut edges = Vec::new();
        for caller_fact in caller_facts.into_iter() {
            if let Some(callee_fact) = callee_facts
                .iter()
                .find(|x| caller_fact.memory_offset == x.memory_offset)
            {
                edges.push(Edge::Call {
                    from: caller_fact,
                    to: callee_fact.clone().clone(),
                });
            } else {
                // the variable does not exist, therefore creating it
                let callee_fact = state.init_memory_fact(&callee_function.name, &caller_fact)?;

                edges.push(Edge::Call {
                    from: caller_fact,
                    to: callee_fact.clone().clone(),
                });
            }
        }

        Ok(edges)
    }

    /// Computes exit-to-return edges
    fn return_val(
        &self,
        caller_function: &String, //d4
        callee_function: &String, //d2
        caller_pc: usize,         //d4
        callee_pc: usize,         //d2
        caller_instructions: &Vec<Instruction>,
        _graph: &mut Graph,
        return_vals: &Vec<String>,
        state: &mut State,
    ) -> Result<Vec<Edge>> {
        debug!("Trying to compute return_val");
        debug!("Caller: {} ({})", caller_function, caller_pc);
        debug!("Callee: {} ({})", callee_function, callee_pc);

        let dest = match caller_instructions.get(caller_pc).as_ref() {
            Some(Instruction::Call(_, _params, dest)) => {
                let mut dd = Vec::with_capacity(dest.len());
                dd.push("taut".to_string());
                dd.extend(dest.clone());
                dd
            }
            Some(x) => bail!("Wrong instruction passed to return val. Found {:?}", x),
            None => bail!("Cannot find instruction while trying to compute exit-to-return edges"),
        };

        let caller_facts = state
            .get_facts_at(caller_function, caller_pc + 1)?
            .filter(|x| !x.var_is_memory);

        let caller_facts_memory: Vec<_> = state
            .get_facts_at(caller_function, caller_pc + 1)?
            .filter(|x| x.var_is_memory)
            .cloned()
            .collect();

        let mut edges = Vec::new();

        let mut caller_facts = caller_facts.into_iter().cloned().collect::<Vec<_>>();
        debug!("Caller facts {:#?}", caller_facts);

        let mut callee_facts_without_globals = state
            .get_facts_at(callee_function, callee_pc)?
            .filter(|x| !x.var_is_global && !x.var_is_memory)
            .cloned()
            .collect::<Vec<_>>();

        let mut callee_facts_with_globals = state
            .get_facts_at(callee_function, callee_pc)?
            .filter(|x| x.var_is_global)
            .cloned()
            .collect::<Vec<_>>();

        let callee_facts_with_memory = state
            .get_facts_at(callee_function, callee_pc)?
            .filter(|x| x.var_is_memory)
            .cloned()
            .collect::<Vec<_>>();

        caller_facts.sort_by(|a, b| a.track.cmp(&b.track));
        callee_facts_without_globals.sort_by(|a, b| a.track.cmp(&b.track));
        callee_facts_with_globals.sort_by(|a, b| a.track.cmp(&b.track));

        caller_facts.dedup();
        callee_facts_without_globals.dedup();
        callee_facts_with_globals.dedup();

        debug!("caller_facts {:#?}", caller_facts);
        debug!(
            "callee_facts without globals {:#?}",
            callee_facts_without_globals
        );
        debug!("callee_facts with globals {:#?}", callee_facts_with_globals);
        debug!("callee_facts with memory {:#?}", callee_facts_with_memory);

        // Generate edges for all dest + taut
        debug!("=> dest {:?}", dest);

        // we need to chain because we can also return globals
        let callee_facts_globals_that_were_returned = callee_facts_with_globals
            .clone()
            .into_iter()
            .filter(|x| return_vals.contains(&x.belongs_to_var));

        for (from, to_reg) in callee_facts_without_globals
            .clone()
            .into_iter()
            .chain(callee_facts_globals_that_were_returned)
            .zip(dest.into_iter())
        {
            if let Some(to) = caller_facts.iter().find(|x| x.belongs_to_var == to_reg) {
                edges.push(Edge::Return {
                    from: from,
                    to: to.clone(),
                });
            } else {
                //Create the dest
                let track = state
                    .get_track(caller_function, &to_reg)
                    .with_context(|| format!("Cannot find track {}", to_reg))?;

                let fact = Fact {
                    belongs_to_var: to_reg.clone(),
                    function: caller_function.clone(),
                    next_pc: caller_pc + 1,
                    track,
                    var_is_global: false,
                    var_is_taut: from.var_is_taut,
                    memory_offset: from.memory_offset,
                    var_is_memory: from.var_is_memory,
                };

                let to = state.cache_fact(caller_function, fact)?;

                edges.push(Edge::Return {
                    from: from.clone().clone(),
                    to: to.clone(),
                });
            }
        }

        // Edges only for globals
        // doesn't handle when you return into local from global
        for from in callee_facts_with_globals.clone().into_iter() {
            //Create the dest
            let track = state
                .get_track(caller_function, &from.belongs_to_var) //name must match
                .with_context(|| format!("Cannot find track {}", from.belongs_to_var))?;

            let fact = Fact {
                belongs_to_var: from.belongs_to_var.clone(),
                function: caller_function.clone(),
                next_pc: caller_pc + 1,
                track,
                var_is_global: true,
                var_is_taut: from.var_is_taut,
                var_is_memory: false,
                memory_offset: None,
            };

            let to = state.cache_fact(caller_function, fact)?;

            edges.push(Edge::Return {
                from: from.clone().clone(),
                to: to.clone(),
            });
        }

        // Edges only for memory
        for from in callee_facts_with_memory.clone().into_iter() {
            if let Some(caller_fact) = caller_facts_memory
                .iter()
                .find(|x| x.belongs_to_var == from.belongs_to_var)
            {
                edges.push(Edge::Return {
                    from: from.clone().clone(),
                    to: caller_fact.clone(),
                });
            } else {
                state.add_memory_var(caller_function.clone(), from.memory_offset.unwrap());

                let track = state
                    .get_track(caller_function, &from.belongs_to_var) //name must match
                    .with_context(|| format!("Cannot find track {}", from.belongs_to_var))?;

                let fact = Fact {
                    belongs_to_var: from.belongs_to_var.clone(),
                    function: caller_function.clone(),
                    next_pc: caller_pc + 1,
                    track,
                    var_is_global: false,
                    var_is_taut: false,
                    var_is_memory: true,
                    memory_offset: from.memory_offset.clone(),
                };

                let to = state.cache_fact(caller_function, fact)?;

                edges.push(Edge::Return {
                    from: from.clone().clone(),
                    to: to.clone(),
                });
            }
        }

        Ok(edges)
    }

    /// Computes call-to-return
    fn call_flow(
        &self,
        _program: &Program,
        caller_function: &AstFunction,
        callee: &String,
        _params: &Vec<String>,
        dests: &Vec<String>,
        _graph: &mut Graph,
        pc: usize,
        caller: &String,
        state: &mut State,
    ) -> Result<Vec<Edge>> {
        debug!(
            "Generating call-to-return edges for {} ({})",
            callee, caller
        );

        let before: Vec<_> = state
            .get_facts_at(&caller_function.name, pc)?
            .into_iter()
            .filter(|x| &x.belongs_to_var == caller)
            .filter(|x| !x.var_is_global)
            .map(|x| x.clone())
            .collect();
        debug!("Facts before statement {}", before.len());

        let after = state.get_facts_at(&caller_function.name, pc)?; //clone

        // Create a copy of `before`, but eliminate all not needed facts
        // and advance `next_pc`
        let after: Vec<_> = after
            .filter(|x| !x.var_is_taut)
            .filter(|x| !x.var_is_global)
            .filter(|x| !dests.contains(&x.belongs_to_var))
            .cloned()
            .map(|x| {
                let mut y = x;
                y.next_pc += 1;
                y
            })
            .collect();

        debug!("Facts after statement without dests {}", after.len());

        debug!("before {:#?}", before);
        debug!("after {:#?}", after);

        let mut edges = Vec::with_capacity(after.len());
        for fact in before {
            let b = after
                .iter()
                .find(|x| x.belongs_to_var == fact.belongs_to_var);

            if let Some(b) = b {
                // Save the new fact because it is relevant
                state.cache_fact(&b.function, b.clone())?;
                edges.push(Edge::CallToReturn {
                    from: fact.clone(),
                    to: b.clone(),
                });
            } else {
                debug!(
                    "Creating CallToReturn edge for \"{}\" because no match",
                    fact.belongs_to_var
                );
            }
        }

        Ok(edges)
    }

    fn tabulate(
        &mut self,
        mut graph: &mut Graph,
        prog: &Program,
        req: &Request,
        state: &mut State,
    ) -> Result<()> {
        debug!("Convert intermediate repr to graph");

        let function = prog
            .functions
            .iter()
            .find(|x| x.name == req.function)
            .context("Cannot find function")?;

        if state.is_function_defined(&function.name) {
            debug!("==> Function was already summarised.");
            return Ok(());
        }

        let facts = state.init_function(&function, req.pc)?;

        let mut path_edge = Vec::new();
        let mut worklist = VecDeque::new();
        let mut summary_edge = Vec::new();
        let mut normal_flows_debug = Vec::new();

        let init = facts.get(0).unwrap().clone();

        // self loop for taut
        self.propagate(
            graph,
            &mut path_edge,
            &mut worklist,
            Edge::Path {
                from: init.clone(),
                to: init.clone(),
            },
        )?;

        self.pacemaker(
            function,
            &mut graph,
            &mut path_edge,
            &mut worklist,
            &mut normal_flows_debug,
            &facts,
            state,
        )?;

        // Compute init flows
        let init_normal_flows = self.init_flow.flow(
            function,
            graph,
            req.pc,
            &facts,
            &mut normal_flows_debug,
            state,
        )?;

        for edge in init_normal_flows.into_iter() {
            self.propagate(graph, &mut path_edge, &mut worklist, edge)?;
        }

        self.forward(
            &prog,
            &function,
            &mut path_edge,
            &mut worklist,
            &mut summary_edge,
            &mut normal_flows_debug,
            &mut graph,
            state,
            req.pc,
        )?;

        Ok(())
    }

    fn propagate(
        &self,
        graph: &mut Graph,
        path_edge: &mut Vec<Edge>,
        worklist: &mut VecDeque<Edge>,
        e: Edge,
    ) -> Result<()> {
        let from = e.get_from();
        let to = e.to();

        let f = path_edge
            .iter()
            .find(|x| x.get_from() == from && x.to() == to);

        if f.is_none() {
            debug!("Propagate {:#?}", e);
            graph.edges.push(e.clone());
            path_edge.push(e.clone());
            worklist.push_back(e);
        }

        Ok(())
    }

    /// Iterates over all instructions and remembers the pc of a
    /// BLOCK declaration. Then saves it into `block_resolver`.
    /// Those values will be used for JUMP instructions.
    fn resolve_block_ids(&mut self, function: &AstFunction, start_pc: usize) -> Result<()> {
        for (pc, instruction) in function
            .instructions
            .iter()
            .enumerate()
            .skip(start_pc)
            .filter(|x| matches!(x.1, Instruction::Block(_)))
        {
            match instruction {
                Instruction::Block(block) => {
                    self.block_resolver
                        .insert((function.name.clone(), block.clone()), pc);
                }
                _ => {
                    bail!("This code should be unreachable.");
                }
            }
        }

        Ok(())
    }

    fn forward(
        &mut self,
        program: &Program,
        function: &AstFunction,
        path_edge: &mut Vec<Edge>,
        worklist: &mut VecDeque<Edge>,
        summary_edge: &mut Vec<Edge>,
        normal_flows_debug: &mut Vec<Edge>,
        graph: &mut Graph,
        state: &mut State,
        start_pc: usize,
    ) -> Result<()> {
        let mut end_summary: HashMap<(String, usize, String), Vec<Fact>> = HashMap::new();
        let mut incoming: HashMap<(String, usize, String), Vec<Fact>> = HashMap::new();

        // Save all blocks from the beginning.
        self.resolve_block_ids(&function, 0)?;

        while let Some(edge) = worklist.pop_front() {
            debug!("Popping edge from worklist {:#?}", edge);

            assert!(
                matches!(edge, Edge::Path { .. }),
                "Edge in the worklist has wrong type"
            );

            let d1 = edge.get_from();
            let d2 = edge.to();

            let pc = edge.to().next_pc;
            debug!("Instruction pointer is {}", pc);

            let instructions = &program
                .functions
                .iter()
                .find(|x| x.name == d2.function)
                .context("Cannot find function")?
                .instructions;
            let n = instructions.get(pc);
            debug!("=> Instruction {:?}", n);

            if let Some(n) = n {
                match n {
                    Instruction::Call(callee, params, dest) => {
                        self.handle_call(
                            &program,
                            d1,
                            d2,
                            callee,
                            params,
                            graph,
                            path_edge,
                            worklist,
                            &mut incoming,
                            &end_summary,
                            function,
                            summary_edge,
                            dest,
                            pc,
                            normal_flows_debug,
                            state,
                            start_pc,
                        )?;
                    }
                    Instruction::Return(_dest) => {
                        let new_function = program
                            .functions
                            .iter()
                            .find(|x| x.name == d2.function)
                            .context("Cannot find function")?;

                        self.handle_return(
                            new_function,
                            d2,
                            graph,
                            normal_flows_debug,
                            path_edge,
                            worklist,
                            d1,
                            &mut end_summary,
                            state,
                        )?;
                    }
                    _ => {
                        let new_function = program
                            .functions
                            .iter()
                            .find(|x| x.name == d2.function)
                            .unwrap();
                        for f in self
                            .normal_flow
                            .flow(
                                &new_function,
                                graph,
                                d2.next_pc,
                                &d2.belongs_to_var,
                                &self.block_resolver,
                                state,
                            )?
                            .iter()
                        {
                            debug!("Normal flow {:#?}", f);
                            let to = f.to();

                            normal_flows_debug.push(f.clone());

                            self.propagate(
                                graph,
                                path_edge,
                                worklist,
                                Edge::Path {
                                    from: d1.clone(),
                                    to: to.clone(),
                                },
                            )?;
                        }
                    }
                }
            } else {
                self.end_procedure(
                    &program,
                    graph,
                    summary_edge,
                    &mut incoming,
                    &mut end_summary,
                    d1,
                    d2,
                    path_edge,
                    worklist,
                    state,
                )?;
            }
        }

        //graph.edges.extend_from_slice(&path_edge);
        //graph.edges.extend_from_slice(&normal_flows_debug);
        //graph.edges.extend_from_slice(&summary_edge);

        Ok(())
    }

    fn handle_call(
        &mut self,
        program: &Program,
        d1: &Fact,
        d2: &Fact,
        callee: &String,
        params: &Vec<String>,
        graph: &mut Graph,
        path_edge: &mut Vec<Edge>,
        worklist: &mut VecDeque<Edge>,
        incoming: &mut HashMap<(String, usize, String), Vec<Fact>>,
        end_summary: &HashMap<(String, usize, String), Vec<Fact>>,
        function: &AstFunction,
        summary_edge: &mut Vec<Edge>,
        dest: &Vec<String>,
        pc: usize,
        normal_flows_debug: &mut Vec<Edge>,
        state: &mut State,
        start_pc: usize,
    ) -> Result<(), anyhow::Error> {
        let caller_var = &d2.belongs_to_var;
        let caller_function = &program
            .functions
            .iter()
            .find(|x| x.name == d1.function)
            .context("Cannot find function for the caller")?;

        let caller_instructions = &caller_function.instructions;

        let callee_function = program
            .functions
            .iter()
            .find(|x| &x.name == callee)
            .context("Cannot find function")?;

        let call_edges = self
            .pass_args(
                caller_function,
                callee_function,
                params,
                graph,
                d2.next_pc,
                caller_var,
                normal_flows_debug,
                path_edge,
                worklist,
                state,
            )
            .with_context(|| {
                format!(
                    "Error occured during `pass_args` for function {} at {}",
                    callee, pc
                )
            })?;
        for d3 in call_edges.into_iter() {
            debug!("d3 {:#?}", d3);

            self.propagate(
                graph,
                path_edge,
                worklist,
                Edge::Path {
                    from: d3.to().clone(),
                    to: d3.to().clone(),
                },
            )?; //self loop

            //Add incoming
            if let Some(incoming) = incoming.get_mut(&(
                d3.to().function.clone(),
                d3.to().next_pc,
                d3.to().belongs_to_var.clone(),
            )) {
                if !incoming.contains(&d2) {
                    incoming.push(d2.clone());
                }
            } else {
                incoming.insert(
                    (
                        d3.to().function.clone(),
                        d3.to().next_pc,
                        d3.to().belongs_to_var.clone(),
                    ),
                    vec![d2.clone()],
                );
            }

            debug!("Incoming in call {:#?}", incoming);
            debug!("end summary {:#?}", end_summary);

            if let Some(end_summary) = end_summary.get(&(
                d3.to().function.clone(),
                d3.to().next_pc,
                d3.to().belongs_to_var.clone(),
            )) {
                for d4 in end_summary.iter() {
                    debug!("d4 {:#?}", d4);

                    let return_vals = &program
                        .functions
                        .iter()
                        .find(|x| x.name == d2.function)
                        .context("Cannot find function")?
                        .instructions
                        .get(d2.next_pc - 1)
                        .map(|x| match x {
                            Instruction::Return(x) => x.clone(),
                            _ => Vec::new(),
                        })
                        .unwrap_or(Vec::new());

                    for d5 in self.return_val(
                        &d2.function,
                        &d4.function,
                        d2.next_pc,
                        d4.next_pc,
                        caller_instructions,
                        graph,
                        &return_vals,
                        state,
                    )? {
                        summary_edge.push(Edge::Summary {
                            from: d2.clone(),
                            to: d5.get_from().clone(),
                        });
                    }
                }
            }

            debug!("End summary {:#?}", end_summary);
        }
        let call_flow = self.call_flow(
            program,
            function,
            callee,
            params,
            dest,
            graph,
            pc,
            &d2.belongs_to_var,
            state,
        )?;
        let return_sites = summary_edge.iter().filter(|x| {
            x.get_from().belongs_to_var == d2.belongs_to_var
                && x.get_from().next_pc == d2.next_pc
                && x.to().next_pc == d2.next_pc + 1
        });
        Ok(for d3 in call_flow.iter().chain(return_sites) {
            let taut = state
                .get_facts_at(&d3.get_from().function, start_pc)
                .context("Cannot find start facts")?
                .find(|x| x.var_is_taut)
                .context("Cannot find tautological start fact")?
                .clone();

            normal_flows_debug.push(d3.clone());
            self.propagate(
                graph,
                path_edge,
                worklist,
                Edge::Path {
                    from: taut,
                    to: d3.to().clone(),
                },
            )?; // adding edges to return site of caller from d1
        })
    }

    pub(crate) fn handle_return(
        &mut self,
        new_function: &AstFunction,
        d2: &Fact,
        graph: &mut Graph,
        normal_flows_debug: &mut Vec<Edge>,
        path_edge: &mut Vec<Edge>,
        worklist: &mut VecDeque<Edge>,
        d1: &Fact,
        end_summary: &mut HashMap<(String, usize, String), Vec<Fact>>,
        state: &mut State,
    ) -> Result<(), anyhow::Error> {
        for f in self
            .normal_flow
            .flow(
                &new_function,
                graph,
                d2.next_pc,
                &d2.belongs_to_var,
                &self.block_resolver,
                state,
            )?
            .iter()
        {
            debug!("Normal flow {:#?}", f);
            let to = f.to();

            normal_flows_debug.push(f.clone());

            self.propagate(
                graph,
                path_edge,
                worklist,
                Edge::Path {
                    from: d1.clone(),
                    to: to.clone(),
                },
            )?;
        }
        assert_eq!(d1.function, d2.function);

        let first_statement_pc_callee = state.get_min_pc(&d1.function)?;

        if let Some(end_summary) = end_summary.get_mut(&(
            d1.function.clone(),
            first_statement_pc_callee,
            d1.belongs_to_var.clone(),
        )) {
            let facts = state
                .get_facts_at(&d2.function.clone(), d2.next_pc)?
                .into_iter()
                .filter(|x| x.belongs_to_var == d2.belongs_to_var)
                .map(|x| x.clone());
            end_summary.extend(facts);
        } else {
            let facts = state
                .get_facts_at(&d2.function.clone(), d2.next_pc)?
                .into_iter()
                .filter(|x| x.belongs_to_var == d2.belongs_to_var)
                .map(|x| x.clone())
                .collect();
            end_summary.insert(
                (d1.function.clone(), d1.next_pc, d1.belongs_to_var.clone()),
                facts,
            );
        }
        debug!("End Summary {:#?}", end_summary);
        Ok(())
    }

    pub(crate) fn end_procedure(
        &mut self,
        program: &Program,
        graph: &mut Graph,
        summary_edge: &mut Vec<Edge>,
        incoming: &mut HashMap<(String, usize, String), Vec<Fact>>,
        end_summary: &mut HashMap<(String, usize, String), Vec<Fact>>,
        d1: &Fact,
        d2: &Fact,
        path_edge: &mut Vec<Edge>,
        worklist: &mut VecDeque<Edge>,
        state: &mut State,
    ) -> Result<()> {
        // this is E_p
        debug!("=> Reached end of procedure");

        if d1.function != d2.function {
            debug!("=> From and End of the edge are not the same function. Therefore aborting.");
            return Ok(());
        }

        // Summary
        if let Some(end_summary) =
            end_summary.get_mut(&(d1.function.clone(), d1.next_pc, d1.belongs_to_var.clone()))
        {
            let facts = state
                .get_facts_at(&d2.function.clone(), d2.next_pc)?
                .filter(|x| x.belongs_to_var == d2.belongs_to_var)
                .map(|x| x.clone());
            end_summary.extend(facts);
        } else {
            let facts = state
                .get_facts_at(&d2.function.clone(), d2.next_pc)?
                .filter(|x| x.belongs_to_var == d2.belongs_to_var)
                .map(|x| x.clone())
                .collect();
            end_summary.insert(
                (d1.function.clone(), d1.next_pc, d1.belongs_to_var.clone()),
                facts,
            );
        }

        debug!("End Summary {:#?}", end_summary);

        // Incoming has as key the beginning of procedure
        // The values are the callers of the procedure.
        if let Some(incoming) =
            incoming.get_mut(&(d1.function.clone(), d1.next_pc, d1.belongs_to_var.clone()))
        {
            debug!("Incoming {:#?}", incoming);
            for d4 in incoming {
                debug!("Computing return to fact to {:#?}", d4);

                let instructions = &program
                    .functions
                    .iter()
                    .find(|x| x.name == d4.function)
                    .context("Cannot find function")?
                    .instructions;

                let return_vals = &program
                    .functions
                    .iter()
                    .find(|x| x.name == d2.function)
                    .context("Cannot find function")?
                    .instructions
                    .get(d2.next_pc - 1)
                    .map(|x| match x {
                        Instruction::Return(x) => x.clone(),
                        _ => Vec::new(),
                    })
                    .unwrap_or(Vec::new());

                // Use only `d4`'s var
                let ret_vals = self.return_val(
                    &d4.function,
                    &d2.function,
                    d4.next_pc,
                    d2.next_pc,
                    &instructions,
                    graph,
                    return_vals,
                    state,
                )?;

                let ret_vals = ret_vals.iter().map(|x| x.to()).collect::<Vec<_>>();

                debug!("Exit-To-Return edges are {:#?}", ret_vals);

                for d5 in ret_vals.into_iter() {
                    debug!("Handling var {:#?}", d5);

                    debug!("summary_edge {:#?}", summary_edge);
                    if summary_edge
                        .iter()
                        .find(|x| x.get_from() == d4 && x.to() == d5)
                        .is_none()
                    {
                        summary_edge.push(Edge::Summary {
                            from: d4.clone(),
                            to: d5.clone().clone(),
                        });

                        // Get all path edges
                        // from `d3` to `d4`
                        let edges: Vec<_> = path_edge
                            .iter()
                            .filter(|x| {
                                x.to() == d4 && &x.get_from().function == &d4.function
                                //&& x.get_from().next_pc == 0
                            })
                            .cloned()
                            .collect();

                        for d3 in edges.into_iter() {
                            // here d5 should be var of caller
                            let root = d3.get_from();
                            let d3 = d3.to();

                            // Take the old and replace it with new var.
                            let new_return_site_d5 = Fact {
                                next_pc: d3.next_pc + 1,
                                belongs_to_var: d5.belongs_to_var.clone(),
                                function: d3.function.clone(),
                                var_is_global: d5.var_is_global,
                                var_is_taut: d5.var_is_taut,
                                track: d5.track,
                                memory_offset: d5.memory_offset,
                                var_is_memory: d5.var_is_memory,
                            };

                            self.propagate(
                                graph,
                                path_edge,
                                worklist,
                                Edge::Path {
                                    from: root.clone(),
                                    to: new_return_site_d5,
                                },
                            )?;
                        }
                    }
                }
            }
        }

        Ok(())
    }

    /// Creates the control flow of taut facts.
    /// This is the backbone of the program.
    /// It also propagates them to the `path_edge`.
    pub(crate) fn pacemaker(
        &self,
        function: &AstFunction,
        graph: &mut Graph,
        path_edge: &mut Vec<Edge>,
        worklist: &mut VecDeque<Edge>,
        normal_flows_debug: &mut Vec<Edge>,
        init_facts: &Vec<Fact>,
        state: &mut State,
    ) -> Result<(), anyhow::Error> {
        let mut edges = Vec::new();

        let start_taut = init_facts.get(0).context("Cannot find taut")?;
        let mut last_taut: Option<Fact> = Some(start_taut.clone());

        for (i, instruction) in function.instructions.iter().enumerate() {
            state.add_statement_with_note(
                function,
                format!("{:?}", instruction),
                i,
                &"taut".to_string(),
            )?;
            let facts = state
                .get_facts_at(&function.name, i)?
                .filter(|x| x.belongs_to_var == "taut".to_string())
                .collect::<Vec<_>>();
            let taut = facts.get(0).context("Expected only taut")?.clone();
            debug_assert!(taut.var_is_taut);

            if let Some(last_taut) = last_taut {
                edges.push(Edge::Normal {
                    from: last_taut.clone(),
                    to: taut.clone(),
                    curved: false,
                });
                normal_flows_debug.push(Edge::Normal {
                    from: last_taut.clone(),
                    to: taut.clone(),
                    curved: false,
                });
            }

            last_taut = Some(taut.clone());
        }

        //end
        state.add_statement_with_note(
            function,
            "end".to_string(),
            function.instructions.len(),
            &"taut".to_string(),
        )?;
        let facts = state
            .get_facts_at(&function.name, function.instructions.len())?
            .filter(|x| x.belongs_to_var == "taut".to_string())
            .collect::<Vec<_>>();

        let taut = facts.get(0).context("Expected only taut")?.clone();
        debug_assert!(taut.var_is_taut);

        if let Some(last_taut) = last_taut {
            edges.push(Edge::Normal {
                from: last_taut.clone(),
                to: taut.clone(),
                curved: false,
            });
            normal_flows_debug.push(Edge::Normal {
                from: last_taut.clone(),
                to: taut.clone(),
                curved: false,
            });
        }

        for edge in edges.into_iter() {
            self.propagate(
                graph,
                path_edge,
                worklist,
                Edge::Path {
                    from: start_taut.clone(),
                    to: edge.to().clone(),
                },
            )?;
        }

        Ok(())
    }
}
