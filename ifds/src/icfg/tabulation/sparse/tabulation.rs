#![allow(dead_code)]

/// This module is responsible to parse
/// the webassembly AST to a graph
use crate::icfg::graph::*;
use crate::icfg::state::State;
use crate::ir::ast::Function as AstFunction;
use crate::ir::ast::Instruction;

use rayon::prelude::*;

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

#[derive(Default)]
pub struct EdgeCtx {
    end_summary: LookupTable,
    incoming: LookupTable,
    path_edge: Edges,
    worklist: VecDeque<Edge>,
    summary_edge: Edges,
    normal_flows_debug: Edges,
}

/// Central data structure for the computation of the IFDS problem.
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

    pub fn get_defuse(&self) -> &DefUseChain {
        &self.defuse
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
    pub fn get_scfg_graph(&self, function: &str, var: &str) -> Option<&Graph> {
        self.defuse.get_graph(function, var)
    }

    /// Computes call-to-start edges
    pub(crate) fn pass_args<'a>(
        &mut self,
        caller_function: &AstFunction,
        callee_function: &AstFunction,
        params: &[String],
        ctx: &mut Ctx<'a>,
        current_pc: usize,
        caller_var: &str,
    ) -> Result<Vec<Edge>> {
        let caller_variable = ctx
            .state
            .get_var(&caller_function.name, caller_var)
            .context("Variable is not defined")?
            .clone();

        // Why not dests? Because we don't care about
        // the destination for the function call in
        // `pass_args`
        if params.contains(&caller_var.to_string())
            || caller_variable.is_taut
            || caller_variable.is_global
            || caller_variable.is_memory
        {
            let mut edges = Vec::new();
            // Init facts of the called function
            // Start from the beginning.
            let start_pc = 0;
            let init_facts = ctx
                .state
                .init_function(&callee_function, start_pc)
                .context("Error during function init")?;

            self.pacemaker(callee_function, ctx, start_pc)
                .context("Pacemaker for pass_args failed")?;

            // Save all blocks of the `callee_function`.
            // Because we want to jump to them later.
            self.resolve_block_ids(ctx, &callee_function, start_pc)?;

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

            // Cache globals
            for fact in init_facts.iter().filter(|x| x.var_is_global) {
                self.defuse.cache_when_already_defined(
                    ctx,
                    &callee_function,
                    &fact.belongs_to_var,
                    start_pc,
                )?;
            }

            // Cache memory
            for fact in init_facts.iter().filter(|x| x.var_is_memory) {
                // actually this path is not possible, because
                // memory variables are not initialized by definitions.
                // But, it is still here for sake of completeness.
                self.defuse
                    .cache(ctx, &callee_function, &fact.belongs_to_var, start_pc)?;
            }

            // Cache the rest
            for fact in init_facts.iter().filter(|x| x.var_is_taut) {
                // actually this path is not possible, because
                // memory variables are not initialized by definitions.
                // But, it is still here for sake of completeness.
                self.defuse
                    .cache(ctx, &callee_function, &fact.belongs_to_var, start_pc)?;
            }

            let mut callee_facts = Vec::new();
            // Add global edges
            if caller_variable.is_global {
                callee_facts.push(self.pass_args_globals(
                    ctx,
                    caller_function,
                    callee_function,
                    &caller_variable,
                    &init_facts,
                )?);
            } else if caller_variable.is_memory {
                callee_facts.push(self.pass_args_memory(
                    ctx,
                    caller_function,
                    callee_function,
                    &caller_variable,
                    start_pc,
                )?);
            } else {
                // Get the position in the parameters. If it does not exist then
                // it is `taut` or a `global`.
                let callee_offset = {
                    if caller_variable.is_taut {
                        0
                    } else if !caller_variable.is_memory && !caller_variable.is_global {
                        // Filter by variable type
                        let callee_globals = init_facts.iter().filter(|x| x.var_is_global).count();
                        params
                            .iter()
                            .position(|x| x == caller_var)
                            .map(|x| x + TAUT + callee_globals)
                            .context("Param must exist")?
                    } else {
                        bail!("This cannot happen");
                    }
                };

                let callee_fact = init_facts
                    .get(callee_offset)
                    .context("Cannot find callee's fact")?;

                callee_facts.push(callee_fact.clone());
            }

            let caller_facts =
                self.defuse
                    .get_facts_at(ctx, &caller_function, caller_var, current_pc)?;

            if let Some(caller_fact) = caller_facts.first() {
                for callee_fact in callee_facts.into_iter() {
                    // Create an edge.
                    edges.push(Edge::Call {
                        from: (*caller_fact).clone(),
                        to: callee_fact,
                    });
                }
            }

            return Ok(edges);
        }

        // not a parameter, therefore skipping
        Ok(vec![])
    }

    /// Compute the call-to-start edge for the `caller_variable` when it is a global
    fn pass_args_globals<'a>(
        &mut self,
        _ctx: &mut Ctx<'a>,
        caller_function: &AstFunction,
        _callee_function: &AstFunction,
        caller_variable: &Variable,
        init_facts: &[Fact],
    ) -> Result<Fact> {
        assert!(caller_variable.is_global);

        let pos = caller_function
            .definitions
            .iter()
            .position(|x| x == &caller_variable.name)
            .map(|x| x + TAUT) //the first is taut in `init_facts`
            .context("Global must be defined")?;

        let callee_fact = init_facts.get(pos).context("Cannot find callee's fact")?;

        Ok(callee_fact.clone())
    }

    /// Compute the call-to-start edge for the `caller_variable` when it is a memory var.
    fn pass_args_memory<'a>(
        &mut self,
        ctx: &mut Ctx<'a>,
        _caller_function: &AstFunction,
        callee_function: &AstFunction,
        caller_variable: &Variable,
        start_pc: usize,
    ) -> Result<Fact> {
        assert!(caller_variable.is_memory);
        let mem = ctx.state.add_memory_var(
            callee_function.name.clone(),
            caller_variable
                .memory_offset
                .context("Cannot unpack offset")?,
        );

        let fact = self
            .defuse
            .get_entry_fact(ctx, &callee_function, &mem.name, start_pc)?;

        Ok(fact)
    }

    fn get_function_by_name<'a>(
        &self,
        ctx: &mut Ctx<'a>,
        function: &str,
    ) -> Option<&'a AstFunction> {
        ctx.prog.functions.iter().find(|x| &x.name == function)
    }

    /// Computes exit-to-return edges
    fn return_val<'a>(
        &mut self,
        ctx: &mut Ctx<'a>,
        caller_function: &str, //d4
        callee_function: &str, //d2
        caller_pc: usize,      //d4
        callee_pc: usize,      //d2
        caller_instructions: &[Instruction],
        callee_var: &String,
    ) -> Result<Vec<Edge>> {
        debug!("Trying to compute return_val");
        debug!("Caller: {} ({})", caller_function, caller_pc);
        debug!("Callee: {} ({})", callee_function, callee_pc);
        debug!("Callee var: {} ({})", callee_var, callee_pc);

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
                .get_facts_at(ctx, &callee_function, &"taut".to_string(), callee_pc)
                .context("Cannot get the callee facts")?
                .into_iter()
                .cloned()
                .collect::<Vec<_>>();

            let caller_fact_var = self
                .defuse
                .get_facts_at(ctx, &caller_function, &"taut".to_string(), caller_pc)
                .context("Cannot get the caller facts")?
                .iter()
                .map(|x| x.apply())
                .collect::<Vec<_>>();

            for (from, to) in callee_taut.into_iter().zip(caller_fact_var) {
                edges.push(Edge::Return {
                    from: from.clone(),
                    to,
                });
            }
        } else {
            let dests = match caller_instructions.get(caller_pc).as_ref() {
                Some(Instruction::Call(_, _params, dest)) => dest.clone(),
                Some(x) => bail!("Wrong instruction passed to return val. Found {:?}", x),
                None => {
                    bail!("Cannot find instruction while trying to compute exit-to-return edges")
                }
            };

            let mut caller_facts = Vec::new();

            for dest in dests {
                let caller_fact_var = self
                    .defuse
                    .get_facts_at(ctx, &caller_function, &dest, caller_pc)
                    .context("Cannot get the facts for the new assigned caller's var")?
                    .into_iter()
                    .map(|x| x.apply())
                    .collect::<Vec<_>>();

                if let Some(fact) = caller_fact_var.first() {
                    caller_facts.push(fact.clone());
                }
            }

            let callee_facts = self
                .defuse
                .get_facts_at(ctx, &callee_function, callee_var, callee_pc)
                .context("Retrieving callee_facts failed")?
                .into_iter()
                .collect::<Vec<_>>();

            for (i, caller_fact) in caller_facts.into_iter().enumerate() {
                if let Some(callee_fact) = callee_facts.get(i) {
                    edges.push(Edge::Return {
                        from: (*callee_fact).clone(),
                        to: caller_fact.clone(),
                    });
                }
            }
        }

        // handle globals
        {
            let callee_variable = ctx.state.get_var(&callee_function.name, callee_var);
            if let Some(callee_variable) = callee_variable {
                if callee_variable.is_global {
                    // callee is a global

                    // Now, getting the next caller variable for the callee
                    // The names of the caller's variable and callee's variable is the same.
                    // %-2 -> %-2
                    let caller_fact_var = self
                        .defuse
                        .get_next(ctx, &caller_function, &callee_var, caller_pc)?
                        .into_iter()
                        .collect::<Vec<_>>();

                    let callee_facts = self
                        .defuse
                        .points_to(ctx, &callee_function, callee_var, callee_pc)?
                        .into_iter()
                        .collect::<Vec<_>>();

                    if let Some(callee_fact) = callee_facts.first() {
                        for caller_var in caller_fact_var {
                            self.defuse.force_remove_if_outdated(
                                caller_function,
                                &caller_var.belongs_to_var,
                                caller_pc,
                            )?;

                            edges.push(Edge::Return {
                                from: callee_fact.clone().apply(),
                                to: caller_var.clone(),
                            });
                        }
                    } else {
                        log::warn!("There is no global variable for the callee");
                    }
                }
            }
        }

        // handle memory
        {
            let callee_variable = ctx.state.get_var(&callee_function.name, callee_var);
            if let Some(callee_variable) = callee_variable {
                if callee_variable.is_memory {
                    let is_existing = ctx
                        .state
                        .get_var(&caller_function.name, &callee_variable.name)
                        .is_none();

                    if is_existing {
                        // Check if the caller has the same memory variable,
                        // if not then create one.
                        // This handles when the memory variable was initialized in the callee's method
                        // and needs to be propagated to the caller, but it does not exist
                        // in the caller's function.
                        log::warn!("Memory variable of the caller was not initialized");
                        let memory_offset = callee_variable
                            .memory_offset
                            .context("Memory offset cannot be `None` on a memory variable")?;

                        ctx.state
                            .add_memory_var(caller_function.name.clone(), memory_offset);
                    }

                    let caller_fact_var = self
                        .defuse
                        .get_next(ctx, &caller_function, &callee_var, caller_pc)
                        .context("Cannot retrieve next occurrence of the caller's var")?
                        .into_iter()
                        .collect::<Vec<_>>();

                    let callee_facts = self
                        .defuse
                        .points_to(ctx, &callee_function, callee_var, callee_pc)?
                        .into_iter()
                        .collect::<Vec<_>>();

                    if let Some(callee_fact) = callee_facts.first() {
                        for caller_var in caller_fact_var {
                            self.defuse.force_remove_if_outdated(
                                caller_function,
                                &caller_var.belongs_to_var,
                                caller_pc,
                            )?;

                            edges.push(Edge::Return {
                                from: callee_fact.clone().apply(),
                                to: caller_var.clone(),
                            });
                        }
                    } else {
                        log::warn!("There is no memory variable for the callee");
                    }
                }
            }
        }

        Ok(edges)
    }

    /// Computes call-to-return
    fn call_flow<'a>(
        &mut self,
        ctx: &mut Ctx<'a>,
        _program: &Program,
        caller_function: &AstFunction,
        callee: &str,
        _params: &[String],
        dests: &[String],
        pc: usize,
        caller: &str,
    ) -> Result<Vec<Edge>> {
        debug!(
            "Generating call-to-return edges for {} ({}) at {}",
            callee, caller, pc
        );

        let after = self.defuse.get_next(ctx, &caller_function, caller, pc)?;

        let before = self
            .defuse
            .get_facts_at(ctx, &caller_function, caller, pc)?;

        debug!("Facts before call {:#?}", before);
        debug!("Facts after call {:#?}", after);

        let after: Vec<_> = after
            .into_iter()
            .filter(|x| !dests.contains(&x.belongs_to_var))
            .collect();

        debug!("Facts after statement without dests {}", after.len());

        debug!("before {:#?}", before);
        debug!("after {:#?}", after);

        let mut edges = Vec::with_capacity(after.len());
        for (from, to) in before.into_iter().zip(after) {
            edges.push(Edge::CallToReturn {
                from: from.clone(),
                to: to.clone(),
            });
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

        let facts = ctx.state.init_function(&function, req.pc + 1)?; //the +1 is required for the sparsed variant. Usually, I give it `next_pc`

        debug!("Initial facts {:?}", facts);

        let mut edge_ctx = EdgeCtx::default();

        let init = facts.get(0).unwrap().clone();

        // self loop for taut
        self.propagate(
            &mut ctx.graph,
            &mut edge_ctx,
            Edge::Path {
                from: init.clone(),
                to: init,
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

        let found = edge_ctx.path_edge.par_iter().any(|x| {
            x.get_from().pc == from.pc
                && x.to().pc == to.pc
                && x.get_from().next_pc == from.next_pc
                && x.to().next_pc == to.next_pc
                && x.get_from().belongs_to_var == from.belongs_to_var
                && x.to().belongs_to_var == to.belongs_to_var
                && x.get_from().function == from.function
                && x.to().function == to.function
        });

        if !found {
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
        // We need this variable, because we want to skip a possible
        // CALL instruction if it starts at the first instruction for the first function.
        // The reason is that we already taint it correctly in the initial flows.
        // Handling, additionally, the call would be incorrect.
        let mut is_first_instruction = true;

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
                    Instruction::Call(callee, params, dest)
                        if start_pc != pc || !is_first_instruction =>
                    {
                        self.handle_call(
                            ctx, edge_ctx, &program, d1, d2, callee, params, dest, start_pc,
                        )?;
                    }
                    Instruction::Return(dest)
                        if dest.contains(&d2.belongs_to_var)
                            || d2.var_is_taut
                            || d2.var_is_global
                            || d2.var_is_memory =>
                    {
                        self.end_procedure(ctx, &program, edge_ctx, d1, d2)?;
                    }
                    Instruction::Return(_dest) => {
                        // kill
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

            is_first_instruction = false;
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
        callee: &str,
        params: &[String],
        dests: &[String],
        _start_pc: usize,
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

        // Get the edges from call to argument at the callee
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

        for d3 in call_edges.iter() {
            debug!("d3 {:#?}", d3);
            let d3 = d3.to();

            // self loop
            self.propagate(
                &mut ctx.graph,
                edge_ctx,
                Edge::Path {
                    from: d3.clone(),
                    to: d3.clone(),
                },
            )?;

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

        let first_statement_pc_callee = ctx.state.get_min_pc(&callee_function.name);

        if let Some(first_statement_pc_callee) = first_statement_pc_callee {
            let tauts = self.defuse.get_facts_at(
                ctx,
                &callee_function,
                &"taut".to_string(),
                first_statement_pc_callee,
            )?;

            let taut = (*tauts.first().context("Cannot get the taut fact")?).clone();

            // Extract the goals of the edges.
            let call_facts = call_edges.iter().map(|x| x.to());

            for d3 in call_facts {
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
                    ctx,
                    program,
                    caller_function,
                    callee,
                    params,
                    dests,
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
                let taut = (*self
                    .defuse
                    .get_facts_at(ctx, &caller_function, &"taut".to_string(), 0)
                    .context("Cannot find facts")?
                    .first()
                    .context("Cannot find tautological start fact")?)
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
        } else {
            log::warn!("No initial pc fact found. That's why cannot handle the call");
        }

        Ok(())
    }

    /*
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
    }*/

    /// Add a path edge between `d1` and `d2` to `edge_ctx.end_summary` if it does not already
    /// exists.
    fn union_end_summary_edge<'a>(
        &mut self,
        ctx: &mut Ctx<'a>,
        edge_ctx: &mut EdgeCtx,
        d1: &Fact,
        d2: &Fact,
        pc: PC,
    ) -> Result<()> {
        let d2_function = ctx
            .prog
            .functions
            .iter()
            .find(|x| x.name == d2.function)
            .context("Cannot find function")?;

        // If there is already an entry, then append
        // else, insert into the hashmap
        if let Some(end_summary) =
            edge_ctx
                .end_summary
                .get_mut(&(d1.function.clone(), pc, d1.belongs_to_var.clone()))
        {
            let facts = self
                .defuse
                .get_facts_at(ctx, &d2_function, &d2.belongs_to_var, d2.next_pc)
                .context("Cannot get the facts when unionizing the end_summary edges. Key does already exist")?
                .into_iter()
                .filter(|x| x.pc == x.next_pc).cloned()
                .collect::<Vec<_>>();

            for fact in facts {
                if !end_summary.contains(&fact) {
                    end_summary.push(fact);
                }
            }
        } else {
            let facts = self
                .defuse
                .get_facts_at(ctx, &d2_function, &d2.belongs_to_var, d2.next_pc)
                .context(
                    "Cannot get the facts when unionizing the end_summary edges. Key does not exist",
                )?
                .into_iter()
                .filter(|x| x.pc == x.next_pc).cloned()
                .collect::<Vec<_>>();

            if !facts.is_empty() {
                edge_ctx.end_summary.insert(
                    (d1.function.clone(), d1.pc, d1.belongs_to_var.clone()),
                    facts,
                );
            }
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

        self.union_end_summary_edge(ctx, edge_ctx, d1, d2, d1.pc)?;

        // Incoming has as key the beginning of procedure
        // The values are the callers of the procedure.
        let mut path_edges = Vec::new();
        debug!("Incoming {:#?}", edge_ctx.incoming);
        if let Some(incoming) =
            edge_ctx
                .incoming
                .get(&(d1.function.clone(), d1.pc, d1.belongs_to_var.clone()))
        {
            debug!("Incoming {:#?}", incoming);
            for d4 in incoming.iter() {
                debug!("Computing return from {:#?}", d4);

                let instructions = &program
                    .functions
                    .iter()
                    .find(|x| x.name == d4.function)
                    .context("Cannot find function")?
                    .instructions;

                // Computes the return-to-exit edges
                // Use only `d4`'s var
                let ret_vals = self
                    .return_val(
                        ctx,
                        &d4.function,
                        &d2.function,
                        d4.next_pc,
                        d2.next_pc,
                        &instructions,
                        &d2.belongs_to_var,
                    )
                    .context("Calculating return-to-exit edges failed")?;

                let ret_vals = ret_vals.iter().map(|x| x.to()).collect::<Vec<_>>();

                debug!("Exit-To-Return edges are {:#?}", ret_vals);

                for d5 in ret_vals.into_iter() {
                    debug!("Handling var {:#?}", d5);

                    debug!("summary_edge {:#?}", edge_ctx.summary_edge);
                    if !edge_ctx
                        .summary_edge
                        .iter()
                        .any(|x| x.get_from() == d4 && x.to() == d5)
                    {
                        edge_ctx.summary_edge.push(Edge::Normal {
                            from: d4.clone(),
                            to: (*d5).clone(),
                            curved: false,
                        });

                        // Get all path edges
                        // from `d3` to `d4`
                        let edges: Vec<_> = edge_ctx
                            .path_edge
                            .iter()
                            .filter(|x| x.to() == d4 && x.get_from().function == d4.function)
                            .cloned()
                            .collect();

                        let function = program
                            .functions
                            .iter()
                            .find(|x| x.name == d5.function)
                            .context("Cannot find function")?;

                        for d3 in edges.into_iter() {
                            // here d5 should be var of caller
                            let d3 = d3.get_from();

                            // The `d5` was created and it is the lhs of the CALL instruction.
                            // However only `pc` is correct, but not `next_pc` because we didn't query it

                            let next_d5 = self
                                .defuse
                                .get_next(ctx, &function, &d5.belongs_to_var, d5.pc)
                                .context("Cannot query next facts")?;

                            if next_d5.len() == 0 {
                                // if last instruction, then there won't be a next one.
                                // that's why this is an edge case
                                let mut updated_d5 = d5.clone();
                                updated_d5.next_pc = updated_d5.pc; //they equal
                                path_edges.push(Edge::Path {
                                    from: d3.clone(),
                                    to: updated_d5,
                                });
                            } else {
                                for next_d5 in next_d5 {
                                    let mut updated_d5 = d5.clone();
                                    updated_d5.next_pc = next_d5.pc;
                                    path_edges.push(Edge::Path {
                                        from: d3.clone(),
                                        to: updated_d5,
                                    });
                                }
                            }
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
