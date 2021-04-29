#![allow(dead_code)]

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

use crate::icfg::flowfuncs::{BlockResolver, SparseInitialFlowFunction, SparseNormalFlowFunction};
use crate::ir::ast::Program;

use crate::icfg::tabulation::sparse::defuse::DefUseChain;

const TAUT: usize = 1;

pub struct Ctx<'a> {
    pub graph: &'a mut Graph,
    pub state: &'a mut State,
    pub prog: &'a Program,
    pub block_resolver: BlockResolver,
}

type Function = String;
type PC = usize;
type VariableString = String;
type Facts = Vec<Fact>;
type LookupTable = HashMap<(Function, PC, VariableString), Facts>;
type Edges = Vec<Edge>;

/**
 let mut path_edge = Vec::new();
        let mut worklist = VecDeque::new();
        let mut summary_edge = Vec::new();
        let mut normal_flows_debug = Vec::new();

*/

#[derive(Default)]
pub struct EdgeCtx {
    end_summary: LookupTable,
    incoming: LookupTable,
    path_edge: Edges,
    worklist: VecDeque<Edge>,
    summary_edge: Edges,
    normal_flows_debug: Edges,
}

/// Central datastructure for the computation of the IFDS problem.
#[derive(Debug)]
pub struct TabulationSparse<I, F>
where
    I: SparseInitialFlowFunction,
    F: SparseNormalFlowFunction,
{
    block_counter: Counter,
    init_flow: I,
    normal_flow: F,
    defuse: DefUseChain,
}

impl<I, F> TabulationSparse<I, F>
where
    I: SparseInitialFlowFunction,
    F: SparseNormalFlowFunction,
{
    pub fn new(init_flow: I, flow: F) -> Self {
        Self {
            block_counter: Counter::default(),
            init_flow,
            normal_flow: flow,
            defuse: DefUseChain::default(),
        }
    }

    /// Computes a graph by a given program and `req` ([`Request`]).
    /// The `variable` in `req` doesn't matter. It only matters the `function` and `pc`.
    pub fn visit(&mut self, prog: &Program, req: &Request) -> Result<(Graph, State)> {
        let mut graph = Graph::default();
        let mut state = State::default();

        let mut ctx = Ctx {
            graph: &mut graph,
            state: &mut state,
            prog: &prog,
            block_resolver: HashMap::default(),
        };

        self.tabulate(&mut ctx, prog, req)?;

        Ok((graph, state))
    }

    /// Get a SCFG graph from the `def_use` chain
    pub fn get_scfg_graph(&self, function: &String, var: &String) -> Option<&Graph> {
        self.defuse.get_graph(function, var)
    }

    /// Computes call-to-start edges
    pub(crate) fn pass_args<'a>(
        &mut self,
        caller_function: &AstFunction,
        callee_function: &AstFunction,
        params: &Vec<String>,
        ctx: &mut Ctx<'a>,
        current_pc: usize,
        caller_var: &String,
    ) -> Result<Vec<Edge>> {
        let caller_variable = ctx
            .state
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
            let init_facts = ctx
                .state
                .init_function(&callee_function, start_pc)
                .context("Error during function init")?;

            self.pacemaker(callee_function, ctx, start_pc)
                .context("Pacemaker for pass_args failed")?;

            // Cache all params with `cache_when_already_defined`
            for fact in init_facts
                .iter()
                .filter(|x| params.contains(&x.belongs_to_var))
            {
                self.defuse.cache_when_already_defined(
                    ctx,
                    &callee_function,
                    &fact.belongs_to_var,
                    start_pc,
                )?;
            }
            // Cache the rest normal
            for fact in init_facts
                .iter()
                .filter(|x| !params.contains(&x.belongs_to_var))
            {
                self.defuse
                    .cache(ctx, &callee_function, &fact.belongs_to_var, start_pc)?;
            }

            // Save all blocks of the `callee_function`.
            // Because we want to jump to them later.
            self.resolve_block_ids(ctx, &callee_function, start_pc)?;

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

            /*
            if caller_variable.is_memory {
                let memory_edges = self
                    .pass_args_memory(
                        &caller_function,
                        &callee_function,
                        &caller_variable,
                        current_pc,
                        ctx,
                    )
                    .context("Passing memory variables to a called function failed")?;

                edges.extend(memory_edges);
            }*/

            // Last caller facts
            debug!(
                "caller {} with current_pc {}",
                caller_function.name, current_pc
            );

            // Add global edges
            if caller_variable.is_global {
                self.pass_args_globals(
                    ctx,
                    caller_function,
                    callee_function,
                    callee_fact,
                    &caller_variable,
                    current_pc,
                    &mut edges,
                )?;
            } else {
                assert!(!caller_variable.is_memory);

                let caller_facts =
                    self.defuse
                        .get_facts_at(&caller_function.name, caller_var, current_pc)?;

                if let Some(caller_fact) = caller_facts.first() {
                    // The corresponding edges have to match now, but filter `dest`.
                    // taut -> taut
                    // %0   -> %0
                    // %1   -> %1

                    // Create an edge.
                    edges.push(Edge::Call {
                        from: caller_fact.clone().clone(),
                        to: callee_fact.clone(),
                    });
                }
            }

            Ok(edges)
        } else {
            debug!(
                "Caller's variable is not a parameter {} in {:?} for {}",
                caller_var, params, callee_function.name
            );

            return Ok(vec![]);
        }
    }

    /// Compute the call-to-start edge for the `caller_variable` when it is a global
    fn pass_args_globals<'a>(
        &mut self,
        _ctx: &mut Ctx<'a>,
        caller_function: &AstFunction,
        _callee_function: &AstFunction,
        callee_fact: &Fact,
        caller_variable: &Variable,
        current_pc: usize,
        edges: &mut Edges,
    ) -> Result<()> {
        assert!(caller_variable.is_global);

        let caller_var = &caller_variable.name;

        let caller_facts =
            self.defuse
                .get_facts_at(&caller_function.name, caller_var, current_pc)?;

        if let Some(caller_fact) = caller_facts.first() {
            // Create an edge.
            edges.push(Edge::Call {
                from: caller_fact.clone().clone(),
                to: callee_fact.clone(),
            });
        }

        Ok(())
    }

    /// Handle the parameter argument handling for memory variables
    fn pass_args_memory<'a>(
        &mut self,
        caller_function: &AstFunction,
        callee_function: &AstFunction,
        caller_variable: &Variable,
        current_pc: usize,
        ctx: &mut Ctx<'a>,
    ) -> Result<Vec<Edge>> {
        let start_pc = 0;

        let caller_facts: Vec<_> = ctx
            .state
            .get_facts_at(&caller_function.name, current_pc)?
            .filter(|x| x.var_is_memory && caller_variable.name == x.belongs_to_var)
            .cloned()
            .collect();

        let callee_facts: Vec<_> = ctx
            .state
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
                let callee_fact = ctx
                    .state
                    .init_memory_fact(&callee_function.name, &caller_fact)?;

                edges.push(Edge::Call {
                    from: caller_fact,
                    to: callee_fact.clone().clone(),
                });
            }
        }

        Ok(edges)
    }

    fn get_function_by_name<'a>(
        &self,
        ctx: &mut Ctx<'a>,
        function: &String,
    ) -> Option<&'a AstFunction> {
        ctx.prog.functions.iter().find(|x| &x.name == function)
    }

    /// Computes exit-to-return edges
    fn return_val<'a>(
        &mut self,
        ctx: &mut Ctx<'a>,
        caller_function: &String, //d4
        callee_function: &String, //d2
        caller_pc: usize,         //d4
        callee_pc: usize,         //d2
        caller_instructions: &Vec<Instruction>,
        callee_var: &String,
    ) -> Result<Vec<Edge>> {
        debug!("Trying to compute return_val");
        debug!("Caller: {} ({})", caller_function, caller_pc);
        debug!("Callee: {} ({})", callee_function, callee_pc);

        let mut edges = Vec::new();

        let caller_function = ctx
            .prog
            .functions
            .iter()
            .find(|x| &x.name == caller_function)
            .context("Cannot find function")?;
        let callee_function = ctx
            .prog
            .functions
            .iter()
            .find(|x| &x.name == callee_function)
            .context("Cannot find function")?;

        if callee_var == &"taut".to_string() {
            let callee_taut = self
                .defuse
                .get_facts_at(&callee_function.name, &"taut".to_string(), callee_pc)?
                .into_iter()
                .map(|x| x.clone())
                .collect::<Vec<_>>();

            let caller_fact_var = self
                .defuse
                .get_next(ctx, caller_function, &"taut".to_string(), caller_pc)?
                .iter()
                .map(|x| x.apply())
                .collect::<Vec<_>>();

            for (from, to) in callee_taut.into_iter().zip(caller_fact_var) {
                edges.push(Edge::Return {
                    from: from.clone(),
                    to: to,
                });
            }
        }

        let dests = match caller_instructions.get(caller_pc).as_ref() {
            Some(Instruction::Call(_, _params, dest)) => dest.clone(),
            Some(x) => bail!("Wrong instruction passed to return val. Found {:?}", x),
            None => bail!("Cannot find instruction while trying to compute exit-to-return edges"),
        };

        let mut caller_facts = Vec::new();

        for dest in dests {
            let caller_fact_var = self
                .defuse
                .get_next(ctx, caller_function, &dest, caller_pc)?
                .into_iter()
                .map(|x| x.apply())
                .collect::<Vec<_>>();

            caller_facts.extend(caller_fact_var);
        }

        let callee_facts = self
            .defuse
            .get_facts_at(&callee_function.name, callee_var, callee_pc)?
            .into_iter()
            .collect::<Vec<_>>();

        for (i, caller_fact) in caller_facts.into_iter().enumerate() {
            if let Some(callee_fact) = callee_facts.get(i) {
                edges.push(Edge::Return {
                    from: callee_fact.clone().clone(),
                    to: caller_fact,
                });
            }
        }

        /*
        let caller_facts_memory: Vec<_> = ctx
            .state
            .get_facts_at(caller_function, caller_pc + 1)?
            .filter(|x| x.var_is_memory)
            .cloned()
            .collect();

        let mut edges = Vec::new();

        let mut caller_facts = caller_facts.into_iter().cloned().collect::<Vec<_>>();
        debug!("Caller facts {:#?}", caller_facts);

        let mut callee_facts_without_globals = ctx
            .state
            .get_facts_at(callee_function, callee_pc)?
            .filter(|x| !x.var_is_global && !x.var_is_memory)
            .cloned()
            .collect::<Vec<_>>();

        let mut callee_facts_with_globals = ctx
            .state
            .get_facts_at(callee_function, callee_pc)?
            .filter(|x| x.var_is_global)
            .cloned()
            .collect::<Vec<_>>();

        let callee_facts_with_memory = ctx
            .state
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
                let track = ctx
                    .state
                    .get_track(caller_function, &to_reg)
                    .with_context(|| format!("Cannot find track {}", to_reg))?;

                let fact = Fact {
                    belongs_to_var: to_reg.clone(),
                    function: caller_function.clone(),
                    pc: caller_pc,
                    next_pc: caller_pc + 1,
                    track,
                    var_is_global: false,
                    var_is_taut: from.var_is_taut,
                    memory_offset: from.memory_offset,
                    var_is_memory: from.var_is_memory,
                };

                let caller_function_ast = self
                    .get_function_by_name(ctx, caller_function)
                    .context("Cannot get function")?;

                let _ =
                    self.defuse
                        .cache(ctx, caller_function_ast, &fact.belongs_to_var, caller_pc)?;

                edges.push(Edge::Return {
                    from: from.clone().clone(),
                    to: fact,
                });
            }
        }

        // Edges only for globals
        // doesn't handle when you return into local from global
        for from in callee_facts_with_globals.clone().into_iter() {
            //Create the dest
            let track = ctx
                .state
                .get_track(caller_function, &from.belongs_to_var) //name must match
                .with_context(|| format!("Cannot find track {}", from.belongs_to_var))?;

            let fact = Fact {
                belongs_to_var: from.belongs_to_var.clone(),
                function: caller_function.clone(),
                pc: caller_pc,
                next_pc: caller_pc + 1,
                track,
                var_is_global: true,
                var_is_taut: from.var_is_taut,
                var_is_memory: false,
                memory_offset: None,
            };

            let caller_function_ast = self
                .get_function_by_name(ctx, caller_function)
                .context("Cannot get function")?;

            let _ = self
                .defuse
                .cache(ctx, caller_function_ast, &fact.belongs_to_var, caller_pc)?;

            //let to = ctx.state.cache_fact(caller_function, fact)?;

            edges.push(Edge::Return {
                from: from.clone().clone(),
                to: fact,
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
                ctx.state
                    .add_memory_var(caller_function.clone(), from.memory_offset.unwrap());

                let track = ctx
                    .state
                    .get_track(caller_function, &from.belongs_to_var) //name must match
                    .with_context(|| format!("Cannot find track {}", from.belongs_to_var))?;

                let fact = Fact {
                    belongs_to_var: from.belongs_to_var.clone(),
                    function: caller_function.clone(),
                    pc: caller_pc,
                    next_pc: caller_pc + 1,
                    track,
                    var_is_global: false,
                    var_is_taut: false,
                    var_is_memory: true,
                    memory_offset: from.memory_offset.clone(),
                };

                let caller_function_ast = self
                    .get_function_by_name(ctx, caller_function)
                    .context("Cannot get function")?;

                let _ =
                    self.defuse
                        .cache(ctx, caller_function_ast, &fact.belongs_to_var, caller_pc)?;

                //let to = ctx.state.cache_fact(caller_function, fact)?;

                edges.push(Edge::Return {
                    from: from.clone().clone(),
                    to: fact,
                });
            }
        }*/

        Ok(edges)
    }

    /// Computes call-to-return
    fn call_flow<'a>(
        &self,
        _program: &Program,
        caller_function: &AstFunction,
        callee: &String,
        _params: &Vec<String>,
        dests: &Vec<String>,
        ctx: &mut Ctx<'a>,
        pc: usize,
        caller: &String,
    ) -> Result<Vec<Edge>> {
        debug!(
            "Generating call-to-return edges for {} ({})",
            callee, caller
        );

        let before: Vec<_> = ctx
            .state
            .get_facts_at(&caller_function.name, pc)?
            .into_iter()
            .filter(|x| &x.belongs_to_var == caller)
            .filter(|x| !x.var_is_global)
            .map(|x| x.clone())
            .collect();
        debug!("Facts before statement {}", before.len());

        let after = self
            .defuse
            .get_facts_at(&caller_function.name, caller, pc)?;

        // Create a copy of `before`, but eliminate all not needed facts
        // and advance `next_pc`
        let after: Vec<_> = after
            .into_iter()
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
                //ctx.state.cache_fact(&b.function, b.clone())?;
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

    fn tabulate<'a>(&mut self, ctx: &mut Ctx<'a>, prog: &Program, req: &Request) -> Result<()> {
        debug!("Convert intermediate repr to graph");

        let function = prog
            .functions
            .iter()
            .find(|x| x.name == req.function)
            .context("Cannot find function")?;

        if ctx.state.is_function_defined(&function.name) {
            debug!("==> Function was already summarised.");
            return Ok(());
        }

        let facts = ctx.state.init_function(&function, req.pc)?;

        debug!("Initial facts {:?}", facts);

        let mut edge_ctx = EdgeCtx::default();

        let init = facts.get(0).unwrap().clone();

        // self loop for taut
        self.propagate(
            &mut ctx.graph,
            &mut edge_ctx,
            Edge::Path {
                from: init.clone(),
                to: init.clone(),
            },
        )?;

        self.pacemaker(function, ctx, req.pc)?;

        // Save all blocks from the beginning.
        self.resolve_block_ids(ctx, &function, 0)?;

        {
            // Compute init flows
            let init_normal_flows =
                self.init_flow
                    .flow(ctx, function, req.pc, &facts, &mut self.defuse)?;

            for edge in init_normal_flows.into_iter() {
                debug!("Propagating initial fact");
                self.propagate(&mut ctx.graph, &mut edge_ctx, edge)?;
            }
        }

        self.forward(&prog, ctx, req.pc, &mut edge_ctx)?;

        Ok(())
    }

    /// Adding path edges to the `worklist` and `path_edge` if it does not exist already.
    fn propagate(&self, graph: &mut Graph, edge_ctx: &mut EdgeCtx, e: Edge) -> Result<()> {
        let from = e.get_from();
        let to = e.to();

        let f = edge_ctx.path_edge.iter().find(|x| {
            x.get_from().pc == from.pc
                && x.to().pc == to.pc
                && x.get_from().belongs_to_var == from.belongs_to_var
                && x.to().belongs_to_var == to.belongs_to_var
                && x.get_from().function == from.function
                && x.to().function == to.function
        });

        if f.is_none() {
            debug!("Propagate {:#?}", e);
            graph.edges.push(e.clone());
            edge_ctx.path_edge.push(e.clone());
            edge_ctx.worklist.push_back(e);
        }

        Ok(())
    }

    /// Iterates over all instructions and remembers the pc of a
    /// BLOCK declaration. Then saves it into `block_resolver`.
    /// Those values will be used for JUMP instructions.
    fn resolve_block_ids<'a>(
        &mut self,
        ctx: &mut Ctx<'a>,
        function: &AstFunction,
        start_pc: usize,
    ) -> Result<()> {
        debug!("Resolving block ids for {}", function.name);
        for (pc, instruction) in function
            .instructions
            .iter()
            .enumerate()
            .skip(start_pc)
            .filter(|x| matches!(x.1, Instruction::Block(_)))
        {
            match instruction {
                Instruction::Block(block) => {
                    ctx.block_resolver
                        .insert((function.name.clone(), block.clone()), pc);
                }
                _ => {
                    bail!("This code should be unreachable.");
                }
            }
        }

        Ok(())
    }

    /// Handles the forward tabulation.
    fn forward<'a>(
        &mut self,
        program: &Program,
        ctx: &mut Ctx<'a>,
        start_pc: usize,
        edge_ctx: &mut EdgeCtx,
    ) -> Result<()> {
        while let Some(edge) = edge_ctx.worklist.pop_front() {
            debug!("Popping edge from worklist {:#?}", edge);

            assert!(
                matches!(edge, Edge::Path { .. }),
                "Edge has wrong type in the worklist"
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
                            ctx, edge_ctx, &program, d1, d2, callee, params, dest, start_pc,
                        )?;
                    }
                    Instruction::Return(_dest) => {
                        let to_function = program
                            .functions
                            .iter()
                            .find(|x| x.name == d2.function)
                            .context("Cannot find function")?;

                        self.handle_return(to_function, d1, d2, ctx, edge_ctx)?;
                    }
                    _ => {
                        let to_function = program
                            .functions
                            .iter()
                            .find(|x| x.name == d2.function)
                            .unwrap();
                        for d3 in self
                            .normal_flow
                            .flow(
                                ctx,
                                &to_function,
                                d2.next_pc,
                                &d2.belongs_to_var,
                                &mut self.defuse,
                            )?
                            .iter()
                        {
                            debug!("d3 is {:#?}", d3);

                            self.propagate(
                                &mut ctx.graph,
                                edge_ctx,
                                Edge::Path {
                                    from: d1.clone(),
                                    to: d3.clone(),
                                },
                            )?;
                        }
                    }
                }
            } else {
                self.end_procedure(ctx, &program, edge_ctx, d1, d2)?;
            }
        }

        Ok(())
    }

    fn handle_call<'a>(
        &mut self,
        ctx: &mut Ctx<'a>,
        edge_ctx: &mut EdgeCtx,
        program: &Program,
        d1: &Fact,
        d2: &Fact,
        callee: &String,
        params: &Vec<String>,
        dests: &Vec<String>,
        start_pc: usize,
    ) -> Result<(), anyhow::Error> {
        let pc = d2.pc;

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

        self.resolve_block_ids(ctx, callee_function, d2.next_pc)?;

        let call_edges = self
            .pass_args(
                caller_function,
                callee_function,
                params,
                ctx,
                d2.next_pc,
                caller_var,
            )
            .with_context(|| {
                format!(
                    "Error occured during `pass_args` for function {} at {}",
                    callee, pc
                )
            })?;

        let call_facts = call_edges.iter().map(|x| x.to()).collect::<Vec<_>>();

        for d3 in call_edges.iter() {
            debug!("d3 {:#?}", d3);
            let d3 = d3.to();

            self.propagate(
                &mut ctx.graph,
                edge_ctx,
                Edge::Path {
                    from: d3.clone(),
                    to: d3.clone(),
                },
            )?; //self loop

            //Add incoming
            if let Some(incoming) =
                edge_ctx
                    .incoming
                    .get_mut(&(d3.function.clone(), d3.pc, d3.belongs_to_var.clone()))
            {
                if !incoming.contains(&d2) {
                    incoming.push(d2.clone());
                }
            } else {
                edge_ctx.incoming.insert(
                    (d3.function.clone(), d3.pc, d3.belongs_to_var.clone()),
                    vec![d2.clone()],
                );
            }

            debug!("Incoming in call {:#?}", edge_ctx.incoming);
            debug!("end summary {:#?}", edge_ctx.end_summary);

            if let Some(end_summary) =
                edge_ctx
                    .end_summary
                    .get(&(d3.function.clone(), d3.pc, d3.belongs_to_var.clone()))
            {
                for d4 in end_summary.iter() {
                    debug!("d4 {:#?}", d4);

                    for d5 in self.return_val(
                        ctx,
                        &d2.function,
                        &d4.function,
                        d2.next_pc,
                        d4.next_pc,
                        caller_instructions,
                        &d4.belongs_to_var,
                    )? {
                        debug!("d5 {:#?}", d5);
                        assert_eq!(
                            d2.function,
                            d5.to().function,
                            "Summary edges must be intraprocedural"
                        );
                        edge_ctx.summary_edge.push(Edge::Normal {
                            from: d2.clone(),
                            to: d5.to().clone(),
                            curved: false,
                        });
                    }
                }
            }

            debug!("end summary {:#?}", edge_ctx.end_summary);
        }

        let first_statement_pc_callee = ctx.state.get_min_pc(&d1.function)?;
        let taut = self
            .defuse
            .get_facts_at(
                &callee_function.name,
                &"taut".to_string(),
                first_statement_pc_callee,
            )?
            .first()
            .context("Cannot get the taut fact")?
            .clone()
            .clone();

        for d3 in call_facts.into_iter() {
            // add all other usages of the variable
            debug!(
                "Next usages of {} on {} at {}",
                callee_function.name, d3.belongs_to_var, d3.pc
            );

            let usages = self
                .normal_flow
                .flow(
                    ctx,
                    &callee_function,
                    d3.next_pc,
                    &d3.belongs_to_var,
                    &mut self.defuse,
                )?
                .into_iter()
                .collect::<Vec<_>>();

            debug!("usages {:#?}", usages);

            for x in usages.into_iter() {
                self.propagate(
                    &mut ctx.graph,
                    edge_ctx,
                    Edge::Path {
                        from: taut.clone(),
                        to: x.clone(),
                    },
                )?;
            }
        }

        let call_flow = self
            .call_flow(
                program,
                caller_function,
                callee,
                params,
                dests,
                ctx,
                pc,
                &d2.belongs_to_var,
            )?
            .into_iter()
            .map(|x| x.to().clone())
            .collect::<Vec<_>>();

        debug!("call flow {:#?}", call_flow);
        let return_sites = edge_ctx
            .summary_edge
            .clone()
            .into_iter()
            .filter(|x| {
                x.get_from().belongs_to_var == d2.belongs_to_var
                    && x.get_from().function == d2.function
                    && x.get_from().next_pc == d2.next_pc
                    && x.to().next_pc == d2.next_pc + 1
            })
            .map(|x| x.to().clone())
            .collect::<Vec<_>>();
        debug!("return_sites {:#?}", return_sites);

        for d3 in call_flow.into_iter().chain(return_sites) {
            assert_eq!(
                d1.function, d3.function,
                "Call flow edges must be intraprocedural"
            );
            let taut = ctx
                .state
                .get_facts_at(&d1.function, start_pc)
                .context("Cannot find start facts")?
                .find(|x| x.var_is_taut)
                .context("Cannot find tautological start fact")?
                .clone();

            self.propagate(
                &mut ctx.graph,
                edge_ctx,
                Edge::Path {
                    from: taut,
                    to: d3.clone(),
                },
            )?; // adding edges to return site of caller from d1
        }

        Ok(())
    }

    pub(crate) fn handle_return<'a>(
        &mut self,
        to_function: &AstFunction,
        d1: &Fact,
        d2: &Fact,
        ctx: &mut Ctx<'a>,
        edge_ctx: &mut EdgeCtx,
    ) -> Result<()> {
        assert_eq!(d1.function, d2.function);

        for d3 in self
            .normal_flow
            .flow(
                ctx,
                &to_function,
                d2.next_pc,
                &d2.belongs_to_var,
                &mut self.defuse,
            )?
            .iter()
        {
            debug!("d3 is  {:#?}", d3);

            self.propagate(
                &mut ctx.graph,
                edge_ctx,
                Edge::Path {
                    from: d1.clone(),
                    to: d3.clone(),
                },
            )?;
        }

        // first pc of the function, because it could be offsetted
        let first_statement_pc_callee = ctx.state.get_min_pc(&d1.function)?;
        self.union_end_summary_edge(ctx, edge_ctx, d1, d2, first_statement_pc_callee)?;

        Ok(())
    }

    /// Add a path edge between `d1` and `d2` to `edge_ctx.end_summary` if it does not already
    /// exists.
    fn union_end_summary_edge<'a>(
        &self,
        _ctx: &mut Ctx<'a>,
        edge_ctx: &mut EdgeCtx,
        d1: &Fact,
        d2: &Fact,
        pc: PC,
    ) -> Result<()> {
        // If there is already an entry, then append
        // else, insert into the hashmap
        if let Some(end_summary) =
            edge_ctx
                .end_summary
                .get_mut(&(d1.function.clone(), pc, d1.belongs_to_var.clone()))
        {
            let facts = self
                .defuse
                .get_facts_at(&d2.function.clone(), &d2.belongs_to_var, d2.next_pc)?
                .into_iter()
                .filter(|x| x.pc == x.next_pc) // get only real end points
                .map(|x| x.clone())
                .collect::<Vec<_>>();
            end_summary.extend(facts);
            end_summary.dedup();
        } else {
            let facts = self
                .defuse
                .get_facts_at(&d2.function.clone(), &d2.belongs_to_var, d2.next_pc)?
                .into_iter()
                .map(|x| x.clone())
                .collect::<Vec<_>>();

            edge_ctx.end_summary.insert(
                (d1.function.clone(), d1.next_pc, d1.belongs_to_var.clone()),
                facts,
            );
        }
        debug!("End Summary {:#?}", edge_ctx.end_summary);

        Ok(())
    }

    /// Creating intraprocedural summary edges
    /// A summary function is a function from the beginning to the end.
    pub(crate) fn end_procedure<'a>(
        &mut self,
        ctx: &mut Ctx<'a>,
        program: &Program,
        edge_ctx: &mut EdgeCtx,
        d1: &Fact,
        d2: &Fact,
    ) -> Result<()> {
        debug!("=> Reached end of procedure");
        assert!(d1.pc <= d2.pc); // path edge from `taut` to end of function.

        if d1.function != d2.function {
            debug!("=> From and End of the edge are not the same function. Therefore aborting.");
            return Ok(());
        }

        self.union_end_summary_edge(ctx, edge_ctx, d1, d2, d1.next_pc)?;

        // Incoming has as key the beginning of procedure
        // The values are the callers of the procedure.
        let mut path_edges = Vec::new();
        if let Some(incoming) =
            edge_ctx
                .incoming
                .get(&(d1.function.clone(), d1.next_pc, d1.belongs_to_var.clone()))
        {
            debug!("Incoming {:#?}", incoming);
            for d4 in incoming.iter() {
                debug!("Computing return to fact to {:#?}", d4);

                let instructions = &program
                    .functions
                    .iter()
                    .find(|x| x.name == d4.function)
                    .context("Cannot find function")?
                    .instructions;

                // Computes all return-to-exit edges
                // Use only `d4`'s var
                let ret_vals = self.return_val(
                    ctx,
                    &d4.function,
                    &d2.function,
                    d4.next_pc,
                    d2.next_pc,
                    &instructions,
                    &d2.belongs_to_var,
                )?;

                let ret_vals = ret_vals.iter().map(|x| x.to()).collect::<Vec<_>>();

                debug!("Exit-To-Return edges are {:#?}", ret_vals);

                for d5 in ret_vals.into_iter() {
                    debug!("Handling var {:#?}", d5);

                    debug!("summary_edge {:#?}", edge_ctx.summary_edge);
                    if edge_ctx
                        .summary_edge
                        .iter()
                        .find(|x| x.get_from() == d4 && x.to() == d5)
                        .is_none()
                    {
                        edge_ctx.summary_edge.push(Edge::Summary {
                            from: d4.clone(),
                            to: d5.clone().clone(),
                        });

                        // Get all path edges
                        // from `d3` to `d4`
                        let edges: Vec<_> = edge_ctx
                            .path_edge
                            .iter()
                            .filter(|x| {
                                x.to() == d4 && &x.get_from().function == &d4.function
                                //&& x.get_from().next_pc == 0
                            })
                            .cloned()
                            .collect();

                        for d3 in edges.into_iter() {
                            // here d5 should be var of caller
                            let d3 = d3.get_from();

                            path_edges.push(Edge::Path {
                                from: d3.clone(),
                                to: d5.clone(),
                            });
                        }
                    }
                }
            }

            for edge in path_edges.into_iter() {
                self.propagate(&mut ctx.graph, edge_ctx, edge)?;
            }
        }

        Ok(())
    }

    /// Creates the instruction labels
    /// for the graph.
    pub(crate) fn pacemaker<'a>(
        &self,
        function: &AstFunction,
        ctx: &mut Ctx<'a>,
        start_pc: usize,
    ) -> Result<(), anyhow::Error> {
        for (i, instruction) in function.instructions.iter().enumerate().skip(start_pc) {
            ctx.state.add_statement_with_note(
                function,
                format!("{:?}", instruction),
                i,
                &"taut".to_string(),
            )?;
        }

        ctx.state.add_statement_with_note(
            function,
            "end".to_string(),
            function.instructions.len(),
            &"taut".to_string(),
        )?;

        Ok(())
    }
}
