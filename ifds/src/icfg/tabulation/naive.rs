#![allow(dead_code)]

/// This module is responsible to parse
/// the webassembly AST to a graph
use crate::icfg::graph::*;
use crate::icfg::state::State;
use crate::ir::ast::Function as AstFunction;
use crate::ir::ast::Instruction;

use rayon::prelude::*;

use log::debug;

use anyhow::{bail, Context, Result};

use std::collections::HashMap;

use crate::icfg::flowfuncs::BlockResolver;
use crate::ir::ast::Program;

use std::collections::hash_map::Entry;
use std::sync::{Arc, Mutex};

/// Central datastructure for the computation of the IFDS problem.
#[derive(Debug, Default)]
pub struct TabulationNaive;

struct Ctx<'a> {
    program: &'a Program,
    graph: &'a mut Graph,
    state: &'a mut State,
}

type Function = String;
type PC = usize;
type CallerFunction = String;

type CallResolver = HashMap<Function, Vec<(CallerFunction, PC, Vec<String>)>>;

impl TabulationNaive {
    pub fn visit(&mut self, prog: &Program) -> Result<(Graph, State, CallResolver)> {
        let mut graph = Graph::default();
        let mut state = State::default();

        let mut ctx = Ctx {
            program: &prog,
            graph: &mut graph,
            state: &mut state,
        };

        let mut block_resolver: BlockResolver = HashMap::default();
        let mut call_resolver: CallResolver = HashMap::default();

        {
            let block_resolver = Arc::new(Mutex::new(&mut block_resolver));
            let call_resolver = Arc::new(Mutex::new(&mut call_resolver));
            let state = Arc::new(Mutex::new(&mut ctx.state));

            prog.functions.par_iter().for_each(|function| {
                let vars = &function.definitions;

                {
                    let mut lock = state.lock().unwrap();
                    let init = lock
                        .init_function(function, 0)
                        .expect("Cannot init function");
                    let _ = lock
                        .cache_facts(&function.name, init)
                        .expect("Cannot cache facts");
                }

                function
                    .instructions
                    .par_iter()
                    .enumerate()
                    .for_each(|(pc, instruction)| {
                        let mut lock = state.lock().unwrap();
                        let _ = lock.add_statement_with_note_naive(
                            function,
                            format!("{:?}", instruction),
                            pc,
                            &"taut".to_string(),
                        );
                    });

                function
                    .instructions
                    .par_iter()
                    .enumerate()
                    .for_each(|(pc, instruction)| {
                        {
                            let mut lock = state.lock().unwrap();

                            for var in vars.iter() {
                                let _ = lock.add_statement(
                                    function,
                                    format!("{:?}", instruction),
                                    pc + 1,
                                    var,
                                );
                            }
                        }

                        if let Instruction::Block(num) = instruction {
                            let mut lock = block_resolver.lock().unwrap();
                            lock.insert((function.name.clone(), num.clone()), pc);
                        }

                        if let Instruction::Call(callee, _, dest) = instruction {
                            let mut lock = call_resolver.lock().unwrap();
                            match lock.entry(callee.clone()) {
                                Entry::Occupied(mut entry) => {
                                    let entry = entry.get_mut();
                                    entry.push((function.name.clone(), pc, dest.clone()));
                                }
                                Entry::Vacant(entry) => {
                                    entry.insert(vec![(function.name.clone(), pc, dest.clone())]);
                                }
                            }
                        }

                        if let Instruction::CallIndirect(callees, _, dest) = instruction {
                            for callee in callees {
                                let mut lock = call_resolver.lock().unwrap();
                                match lock.entry(callee.clone()) {
                                    Entry::Occupied(mut entry) => {
                                        let entry = entry.get_mut();
                                        entry.push((function.name.clone(), pc, dest.clone()));
                                    }
                                    Entry::Vacant(entry) => {
                                        entry.insert(vec![(
                                            function.name.clone(),
                                            pc,
                                            dest.clone(),
                                        )]);
                                    }
                                }
                            }
                        }
                    });
            });
        }

        debug!("call resolver {:#?}", call_resolver);

        for function in prog.functions.iter() {
            self.once_func(&mut ctx, function, &mut block_resolver, &mut call_resolver)?;
        }

        Ok((graph, state, call_resolver))
    }

    fn once_func<'a>(
        &mut self,
        ctx: &mut Ctx<'a>,
        function: &AstFunction,
        block_resolver: &mut BlockResolver,
        call_resolver: &mut CallResolver,
    ) -> Result<()> {
        for (pc, instruction) in function.instructions.iter().enumerate() {
            self.once(
                ctx,
                function,
                instruction,
                pc,
                block_resolver,
                call_resolver,
            )?;
        }

        Ok(())
    }

    /// A fact has been dynamically added. This method creates all facts to the end.
    fn create_line<'a>(&mut self, ctx: &mut Ctx<'a>, function: &String, fact: Fact) -> Result<()> {
        let function = ctx
            .program
            .functions
            .iter()
            .find(|x| &x.name == function)
            .context("Cannot find function")?;

        for (pc, _instruction) in function
            .instructions
            .iter()
            .enumerate()
            .skip(fact.next_pc.saturating_sub(1))
        {
            let mut new_fact = fact.clone();
            new_fact.next_pc = pc;

            let _ = ctx.state.cache_fact(&function.name, new_fact)?;
        }

        let mut new_fact = fact;
        new_fact.next_pc = function.instructions.len();
        let _ = ctx.state.cache_fact(&function.name, new_fact)?;

        Ok(())
    }

    fn once<'a>(
        &mut self,
        ctx: &mut Ctx<'a>,
        function: &AstFunction,
        instruction: &Instruction,
        pc: usize,
        block_resolver: &mut BlockResolver,
        call_resolver: &mut CallResolver,
    ) -> Result<()> {
        match instruction {
            Instruction::Const(dest, _) | Instruction::Unknown(dest) => {
                let in_ = ctx.state.get_facts_at(&function.name, pc)?;
                let out_ = ctx.state.get_facts_at(&function.name, pc + 1)?;

                for (from, after) in in_
                    .zip(out_)
                    .filter(|(from, to)| &from.belongs_to_var != dest && &to.belongs_to_var != dest)
                {
                    ctx.graph.add_normal(from.clone(), after.clone())?;
                }

                let in_ = ctx
                    .state
                    .get_facts_at(&function.name, pc)?
                    .filter(|x| x.var_is_taut);
                let out_ = ctx
                    .state
                    .get_facts_at(&function.name, pc + 1)?
                    .filter(|x| &x.belongs_to_var == dest);

                for (from, after) in in_.zip(out_) {
                    ctx.graph.add_normal(from.clone(), after.clone())?;
                }
            }
            Instruction::Assign(dest, src) | Instruction::Unop(dest, src) => {
                let in_ = ctx.state.get_facts_at(&function.name, pc)?;
                let out_ = ctx.state.get_facts_at(&function.name, pc + 1)?;

                for (from, after) in in_
                    .zip(out_)
                    .filter(|(from, to)| &from.belongs_to_var != dest && &to.belongs_to_var != dest)
                {
                    ctx.graph.add_normal(from.clone(), after.clone())?;
                }

                let in_ = ctx
                    .state
                    .get_facts_at(&function.name, pc)?
                    .filter(|x| &x.belongs_to_var == src);
                let out_ = ctx
                    .state
                    .get_facts_at(&function.name, pc + 1)?
                    .filter(|x| &x.belongs_to_var == dest);

                for (from, after) in in_.zip(out_) {
                    ctx.graph.add_normal(from.clone(), after.clone())?;
                }
            }
            Instruction::BinOp(dest, src1, src2) | Instruction::Phi(dest, src1, src2) => {
                let in_ = ctx.state.get_facts_at(&function.name, pc)?;
                let out_ = ctx.state.get_facts_at(&function.name, pc + 1)?;

                for (from, after) in in_
                    .zip(out_)
                    .filter(|(from, to)| &from.belongs_to_var != dest && &to.belongs_to_var != dest)
                {
                    ctx.graph.add_normal(from.clone(), after.clone())?;
                }

                let in_ = ctx
                    .state
                    .get_facts_at(&function.name, pc)?
                    .filter(|x| &x.belongs_to_var == src1);
                let out_ = ctx
                    .state
                    .get_facts_at(&function.name, pc + 1)?
                    .filter(|x| &x.belongs_to_var == dest);

                for (from, after) in in_.zip(out_) {
                    ctx.graph.add_normal(from.clone(), after.clone())?;
                }

                let in_ = ctx
                    .state
                    .get_facts_at(&function.name, pc)?
                    .filter(|x| &x.belongs_to_var == src2);
                let out_ = ctx
                    .state
                    .get_facts_at(&function.name, pc + 1)?
                    .filter(|x| &x.belongs_to_var == dest);

                for (from, after) in in_.zip(out_) {
                    ctx.graph.add_normal(from.clone(), after.clone())?;
                }
            }
            Instruction::Kill(dest) => {
                let in_ = ctx.state.get_facts_at(&function.name, pc)?;
                let out_ = ctx.state.get_facts_at(&function.name, pc + 1)?;

                for (from, after) in in_
                    .zip(out_)
                    .filter(|(from, to)| &from.belongs_to_var != dest && &to.belongs_to_var != dest)
                {
                    ctx.graph.add_normal(from.clone(), after.clone())?;
                }
            }
            Instruction::Call(callee, params, dests) => {
                self.handle_call(ctx, function, pc, callee, params, dests)?;
            }
            Instruction::CallIndirect(callees, params, dests) => {
                for callee in callees {
                    self.handle_call(ctx, function, pc, callee, params, dests)?;
                }
            }
            Instruction::Return(src) => {
                let in_ = ctx.state.get_facts_at(&function.name, pc)?;
                let out_ = ctx.state.get_facts_at(&function.name, pc + 1)?;

                for (from, after) in in_.zip(out_) {
                    ctx.graph.add_normal(from.clone(), after.clone())?;
                }

                if let Some(incoming) = call_resolver.get(&function.name) {
                    for incoming in incoming.iter() {
                        // taut
                        let in_ = ctx
                            .state
                            .get_facts_at(&function.name, pc)?
                            .filter(|x| x.var_is_taut);
                        let out_ = ctx
                            .state
                            .get_facts_at(&incoming.0, incoming.1 + 1)?
                            .filter(|x| x.var_is_taut);

                        for (from, after) in in_.zip(out_) {
                            ctx.graph.add_return(from.clone(), after.clone())?;
                        }

                        let in_ = ctx
                            .state
                            .get_facts_at(&function.name, pc)?
                            .filter(|x| src.contains(&x.belongs_to_var));
                        let out_ = ctx
                            .state
                            .get_facts_at(&incoming.0, incoming.1 + 1)?
                            .filter(|x| incoming.2.contains(&x.belongs_to_var));

                        for (from, after) in in_.zip(out_) {
                            ctx.graph.add_return(from.clone(), after.clone())?;
                        }

                        // Globals

                        let in_ = ctx
                            .state
                            .get_facts_at(&function.name, pc)?
                            .filter(|x| x.var_is_global)
                            .cloned()
                            .collect::<Vec<_>>();

                        for from in in_ {
                            let out_ =
                                ctx.state
                                    .get_facts_at(&incoming.0, incoming.1 + 1)?
                                    .find(|x| {
                                        x.var_is_global && x.belongs_to_var == from.belongs_to_var
                                    });

                            if let Some(after) = out_ {
                                ctx.graph.add_return(from.clone(), after.clone())?;
                            } else {
                                // create it
                                let var = ctx.state.add_global_var(
                                    incoming.0.clone(),
                                    from.belongs_to_var.clone(),
                                );
                                let fact = Fact {
                                    belongs_to_var: from.belongs_to_var.clone(),
                                    function: incoming.0.clone(),
                                    var_is_global: true,
                                    next_pc: incoming.1 + 1,
                                    track: ctx
                                        .state
                                        .get_track(&incoming.0, &var.name)
                                        .context("Cannot find track")?,
                                    ..Default::default()
                                };
                                let to = ctx.state.cache_fact(&incoming.0, fact)?.clone();
                                self.create_line(ctx, &incoming.0, to.clone())?;

                                ctx.graph.add_return(from.clone(), to)?;
                            }
                        }

                        // Memories

                        let in_ = ctx
                            .state
                            .get_facts_at(&function.name, pc)?
                            .filter(|x| x.var_is_memory)
                            .cloned()
                            .collect::<Vec<_>>();

                        for from in in_ {
                            let out_ =
                                ctx.state
                                    .get_facts_at(&incoming.0, incoming.1 + 1)?
                                    .find(|x| {
                                        x.var_is_memory && x.belongs_to_var == from.belongs_to_var
                                    });

                            if let Some(after) = out_ {
                                ctx.graph.add_return(from.clone(), after.clone())?;
                            } else {
                                // create it
                                let var = ctx.state.add_memory_var(
                                    incoming.0.clone(),
                                    from.memory_offset.unwrap(),
                                );
                                let fact = Fact {
                                    belongs_to_var: from.belongs_to_var.clone(),
                                    function: incoming.0.clone(),
                                    var_is_memory: true,
                                    memory_offset: var.memory_offset,
                                    next_pc: incoming.1 + 1,
                                    track: ctx
                                        .state
                                        .get_track(&incoming.0, &var.name)
                                        .context("Cannot find track")?,
                                    ..Default::default()
                                };
                                let to = ctx.state.cache_fact(&incoming.0, fact)?.clone();
                                self.create_line(ctx, &incoming.0, to.clone())?;

                                ctx.graph.add_return(from.clone(), to)?;
                            }
                        }
                    }
                }
            }
            Instruction::Jump(num) => {
                if let Some(jump_to_block) =
                    block_resolver.get(&(function.name.clone(), num.clone()))
                {
                    let in_ = ctx.state.get_facts_at(&function.name, pc)?;
                    let out_ = ctx.state.get_facts_at(&function.name, *jump_to_block)?;

                    for (from, after) in in_.zip(out_) {
                        ctx.graph.add_normal_curved(from.clone(), after.clone())?;
                    }
                } else {
                    bail!("Cannot find block to jump to");
                }
            }
            Instruction::Conditional(_src, jumps) => {
                for num in jumps.iter() {
                    if let Some(jump_to_block) =
                        block_resolver.get(&(function.name.clone(), num.clone()))
                    {
                        let in_ = ctx.state.get_facts_at(&function.name, pc)?;
                        let out_ = ctx.state.get_facts_at(&function.name, *jump_to_block)?;

                        for (from, after) in in_.zip(out_) {
                            ctx.graph.add_normal_curved(from.clone(), after.clone())?;
                        }
                    } else {
                        bail!("Cannot find block to jump to");
                    }
                }

                // NORMAL

                let in_ = ctx.state.get_facts_at(&function.name, pc)?;
                let out_ = ctx.state.get_facts_at(&function.name, pc + 1)?;

                for (from, after) in in_.zip(out_) {
                    ctx.graph.add_normal(from.clone(), after.clone())?;
                }
            }
            Instruction::Block(_) => {
                let in_ = ctx.state.get_facts_at(&function.name, pc)?;
                let out_ = ctx.state.get_facts_at(&function.name, pc + 1)?;

                for (from, after) in in_.zip(out_) {
                    ctx.graph.add_normal(from.clone(), after.clone())?;
                }
            }
            Instruction::Table(jumps) => {
                for num in jumps.iter() {
                    if let Some(jump_to_block) =
                        block_resolver.get(&(function.name.clone(), num.clone()))
                    {
                        let in_ = ctx.state.get_facts_at(&function.name, pc)?;
                        let out_ = ctx.state.get_facts_at(&function.name, *jump_to_block)?;

                        for (from, after) in in_.zip(out_) {
                            ctx.graph.add_normal_curved(from.clone(), after.clone())?;
                        }
                    } else {
                        bail!("Cannot find block to jump to");
                    }
                }
            }
            Instruction::Store(src, offset, variable) => {
                let in_ = ctx.state.get_facts_at(&function.name, pc)?;
                let out_ = ctx
                    .state
                    .get_facts_at(&function.name, pc + 1)?
                    .filter(|x| !(x.var_is_memory && x.memory_offset == Some(*offset as usize)));

                for (from, after) in in_.zip(out_) {
                    ctx.graph.add_normal(from.clone(), after.clone())?;
                }

                // Create

                let in_ = ctx
                    .state
                    .get_facts_at(&function.name, pc)?
                    .filter(|x| &x.belongs_to_var == src || &x.belongs_to_var == variable)
                    .cloned()
                    .collect::<Vec<_>>();

                for from in in_ {
                    let var = ctx
                        .state
                        .add_memory_var(function.name.clone(), *offset as usize);

                    let out_ = ctx
                        .state
                        .get_facts_at(&function.name, pc + 1)?
                        .find(|x| x.belongs_to_var == var.name && x.var_is_memory)
                        .cloned();

                    if let Some(to) = out_ {
                        ctx.graph.add_normal(from.clone(), to.clone())?;
                    } else {
                        let fact = Fact {
                            belongs_to_var: var.name.clone(),
                            function: function.name.clone(),
                            var_is_memory: true,
                            memory_offset: var.memory_offset,
                            next_pc: pc + 1,
                            track: ctx
                                .state
                                .get_track(&function.name, &var.name)
                                .context("Cannot find track")?,
                            ..Default::default()
                        };
                        let to = ctx.state.cache_fact(&function.name, fact)?.clone();
                        self.create_line(ctx, &function.name, to.clone())?;

                        ctx.graph.add_normal(from.clone(), to)?;
                    }
                }
            }
            Instruction::Load(dest, offset, src) => {
                let in_ = ctx.state.get_facts_at(&function.name, pc)?;

                for from in in_ {
                    let out_ = ctx
                        .state
                        .get_facts_at(&function.name, pc + 1)?
                        .filter(|x| &x.belongs_to_var == &from.belongs_to_var);

                    for after in out_ {
                        ctx.graph.add_normal(from.clone(), after.clone())?;
                    }
                }

                // Assigment
                let in_ = ctx
                    .state
                    .get_facts_at(&function.name, pc)?
                    .filter(|x| &x.belongs_to_var == src);
                let out_ = ctx
                    .state
                    .get_facts_at(&function.name, pc + 1)?
                    .filter(|x| &x.belongs_to_var == dest);

                for (from, after) in in_.zip(out_) {
                    ctx.graph.add_normal(from.clone(), after.clone())?;
                }

                let mem_var = ctx.state.calculate_mem_var_name(*offset as usize);
                // Assignment from memory
                let in_ = ctx
                    .state
                    .get_facts_at(&function.name, pc)?
                    .filter(|x| x.var_is_memory && &x.belongs_to_var == &mem_var);
                let out_ = ctx
                    .state
                    .get_facts_at(&function.name, pc + 1)?
                    .filter(|x| &x.belongs_to_var == dest);

                for (from, after) in in_.zip(out_) {
                    ctx.graph.add_normal(from.clone(), after.clone())?;
                }
            }
        }

        Ok(())
    }

    fn handle_call(
        &mut self,
        ctx: &mut Ctx,
        function: &AstFunction,
        pc: usize,
        callee: &String,
        params: &[String],
        dests: &[String],
    ) -> Result<()> {
        // Call-to-return edges
        let fi =
            |x: &&Fact| !dests.contains(&x.belongs_to_var) && !x.var_is_global && !x.var_is_memory;

        let in_ = ctx.state.get_facts_at(&function.name, pc)?.filter(fi);
        let out_ = ctx.state.get_facts_at(&function.name, pc + 1)?.filter(fi);

        for (from, after) in in_.zip(out_) {
            ctx.graph.add_normal(from.clone(), after.clone())?;
        }

        // Call edges
        let in_ = ctx
            .state
            .get_facts_at(&function.name, pc)?
            .filter(|x| params.contains(&x.belongs_to_var) || x.var_is_taut);

        let out_ = ctx
            .state
            .get_facts_at(&callee, 0)?
            .filter(|x| params.contains(&x.belongs_to_var) || x.var_is_taut);

        for (from, after) in in_.zip(out_) {
            ctx.graph.add_call(from.clone(), after.clone())?;
        }

        let globals = ctx
            .state
            .get_facts_at(&function.name, pc)?
            .filter(|x| x.var_is_global)
            .cloned()
            .collect::<Vec<_>>();

        for global in globals.into_iter() {
            let var = ctx
                .state
                .add_global_var(callee.clone(), global.belongs_to_var.clone());
            let fact = Fact {
                belongs_to_var: global.belongs_to_var.clone(),
                function: callee.clone(),
                var_is_global: true,
                next_pc: 0,
                track: ctx
                    .state
                    .get_track(&callee, &var.name)
                    .context("Cannot find track")?,
                ..Default::default()
            };
            let to = ctx.state.cache_fact(callee, fact)?.clone();
            self.create_line(ctx, callee, to.clone())?;

            ctx.graph.add_call(global, to)?;
        }

        let memories = ctx
            .state
            .get_facts_at(&function.name, pc)?
            .filter(|x| x.var_is_memory)
            .cloned()
            .collect::<Vec<_>>();

        for mem in memories.into_iter() {
            let var = ctx
                .state
                .add_memory_var(callee.clone(), mem.memory_offset.unwrap());
            let fact = Fact {
                belongs_to_var: mem.belongs_to_var.clone(),
                function: callee.clone(),
                var_is_memory: true,
                next_pc: 0,
                track: ctx
                    .state
                    .get_track(&callee, &var.name)
                    .context("Cannot find track")?,
                memory_offset: var.memory_offset,
                ..Default::default()
            };
            let to = ctx.state.cache_fact(callee, fact)?.clone();
            self.create_line(ctx, callee, to.clone())?;

            ctx.graph.add_call(mem, to)?;
        }

        Ok(())
    }
}
