/// This module is responsible to parse
/// the webassembly AST to a graph
use crate::icfg::graph::*;
use crate::icfg::state::State;
use crate::ir::ast::Function as AstFunction;
use crate::ir::ast::Instruction;

use crate::{counter::Counter, solver::Request};
use anyhow::{bail, Context, Result};
use std::collections::VecDeque;
use tui::widgets::Block;

use log::debug;

use std::collections::HashMap;

use crate::icfg::flowfuncs::{BlockResolver, InitialFlowFunction, NormalFlowFunction};
use crate::ir::ast::Program;

const TAUT: usize = 1;

/// Central datastructure for the computation of the IFDS problem.
#[derive(Debug, Default)]
pub struct Convert {}

struct Ctx<'a> {
    program: &'a Program,
    graph: &'a mut Graph,
    state: &'a mut State,
}

type Function = String;
type PC = usize;
type CallerFunction = String;

type CallResolver = HashMap<Function, (CallerFunction, PC, Vec<String>)>;

impl Convert {
    pub fn visit(&mut self, prog: &Program) -> Result<(Graph, State)> {
        let mut graph = Graph::default();
        let mut state = State::default();

        let mut ctx = Ctx {
            program: &prog,
            graph: &mut graph,
            state: &mut state,
        };

        let mut block_resolver: BlockResolver = HashMap::default();
        let mut call_resolver: CallResolver = HashMap::default();

        for function in prog.functions.iter() {
            let vars = &function.definitions;

            let init = ctx.state.init_function(function, 0)?;
            let _ = ctx.state.cache_facts(&function.name, init)?;

            for (pc, instruction) in function.instructions.iter().enumerate() {
                ctx.state.add_statement_with_note_naive(
                    function,
                    format!("{:?}", instruction),
                    pc,
                    &"taut".to_string(),
                )?;

                for var in vars.iter() {
                    ctx.state
                        .add_statement(function, format!("{:?}", instruction), pc + 1, var)?;
                }

                if let Instruction::Block(num) = instruction {
                    block_resolver.insert((function.name.clone(), num.clone()), pc);
                }

                if let Instruction::Call(callee, _, dest) = instruction {
                    call_resolver.insert(callee.clone(), (function.name.clone(), pc, dest.clone()));
                }

                if let Instruction::CallIndirect(callees, _, dest) = instruction {
                    for callee in callees {
                        call_resolver
                            .insert(callee.clone(), (function.name.clone(), pc, dest.clone()));
                    }
                }
            }
        }

        for function in prog.functions.iter() {
            self.once_func(&mut ctx, function, &mut block_resolver, &mut call_resolver)?;
        }

        Ok((graph, state))
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
            .skip(fact.next_pc.checked_sub(1).unwrap_or(0))
        {
            let mut new_fact = fact.clone();
            new_fact.next_pc = pc;

            let _ = ctx.state.cache_fact(&function.name, new_fact)?;
        }

        let mut new_fact = fact.clone();
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
            Instruction::Const(dest, _num) => {
                let in_ = ctx.state.get_facts_at(&function.name, pc)?;
                let out_ = ctx.state.get_facts_at(&function.name, pc + 1)?;

                for (from, after) in in_
                    .zip(out_)
                    .filter(|(from, to)| &from.belongs_to_var != dest && &to.belongs_to_var != dest)
                    .map(|(from, after)| (from.clone(), after.clone()))
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
                    .map(|(from, after)| (from.clone(), after.clone()))
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
                    .map(|(from, after)| (from.clone(), after.clone()))
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
                    .map(|(from, after)| (from.clone(), after.clone()))
                {
                    ctx.graph.add_normal(from.clone(), after.clone())?;
                }
            }
            Instruction::Call(callee, params, dests) => {
                // Call-to-return edges
                let fi = |x: &&Fact| {
                    !params.contains(&x.belongs_to_var) || !dests.contains(&x.belongs_to_var)
                };

                let in_ = ctx.state.get_facts_at(&function.name, pc)?.filter(fi);
                let out_ = ctx.state.get_facts_at(&function.name, pc + 1)?.filter(fi);

                for (from, after) in in_
                    .zip(out_)
                    .map(|(from, after)| (from.clone(), after.clone()))
                {
                    ctx.graph.add_normal(from.clone(), after.clone())?;
                }

                // Call edges

                let in_ = ctx
                    .state
                    .get_facts_at(&function.name, pc)?
                    .filter(|x| params.contains(&x.belongs_to_var));

                let out_ = ctx.state.get_facts_at(&callee, 0)?;

                for (from, after) in in_
                    .zip(out_)
                    .map(|(from, after)| (from.clone(), after.clone()))
                {
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
                        memory_offset: var.memory_offset.clone(),
                        ..Default::default()
                    };
                    let to = ctx.state.cache_fact(callee, fact)?.clone();
                    self.create_line(ctx, callee, to.clone())?;

                    ctx.graph.add_call(mem, to)?;
                }
            }
            Instruction::CallIndirect(callees, params, dests) => {
                // Call-to-return edges
                let fi = |x: &&Fact| {
                    !params.contains(&x.belongs_to_var) || !dests.contains(&x.belongs_to_var)
                };

                let in_ = ctx.state.get_facts_at(&function.name, pc)?.filter(fi);
                let out_ = ctx.state.get_facts_at(&function.name, pc + 1)?.filter(fi);

                for (from, after) in in_
                    .zip(out_)
                    .map(|(from, after)| (from.clone(), after.clone()))
                {
                    ctx.graph.add_normal(from.clone(), after.clone())?;
                }

                // Call edges

                for callee in callees {
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
                            memory_offset: var.memory_offset.clone(),
                            ..Default::default()
                        };
                        let to = ctx.state.cache_fact(callee, fact)?.clone();
                        self.create_line(ctx, callee, to.clone())?;

                        ctx.graph.add_call(mem, to)?;
                    }
                }
            }
            Instruction::Return(src) => {
                let in_ = ctx.state.get_facts_at(&function.name, pc)?;
                let out_ = ctx.state.get_facts_at(&function.name, pc + 1)?;

                for (from, after) in in_
                    .zip(out_)
                    .map(|(from, after)| (from.clone(), after.clone()))
                {
                    ctx.graph.add_normal(from.clone(), after.clone())?;
                }

                if let Some(incoming) = call_resolver.get(&function.name) {
                    let in_ = ctx
                        .state
                        .get_facts_at(&function.name, pc)?
                        .filter(|x| src.contains(&x.belongs_to_var) || x.var_is_taut);
                    let out_ = ctx
                        .state
                        .get_facts_at(&incoming.0, incoming.1 + 1)?
                        .filter(|x| incoming.2.contains(&x.belongs_to_var) || x.var_is_taut);

                    for (from, after) in in_
                        .zip(out_)
                        .map(|(from, after)| (from.clone(), after.clone()))
                    {
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
                        let out_ = ctx
                            .state
                            .get_facts_at(&incoming.0, incoming.1 + 1)?
                            .find(|x| x.var_is_global && x.belongs_to_var == from.belongs_to_var);

                        if let Some(after) = out_ {
                            ctx.graph.add_return(from.clone(), after.clone())?;
                        } else {
                            // create it
                            let var = ctx
                                .state
                                .add_global_var(incoming.0.clone(), from.belongs_to_var.clone());
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
                        let out_ = ctx
                            .state
                            .get_facts_at(&incoming.0, incoming.1 + 1)?
                            .find(|x| x.var_is_memory && x.belongs_to_var == from.belongs_to_var);

                        if let Some(after) = out_ {
                            ctx.graph.add_return(from.clone(), after.clone())?;
                        } else {
                            // create it
                            let var = ctx
                                .state
                                .add_memory_var(incoming.0.clone(), from.memory_offset.unwrap());
                            let fact = Fact {
                                belongs_to_var: from.belongs_to_var.clone(),
                                function: incoming.0.clone(),
                                var_is_memory: true,
                                memory_offset: var.memory_offset.clone(),
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
            Instruction::Jump(num) => {
                if let Some(jump_to_block) =
                    block_resolver.get(&(function.name.clone(), num.clone()))
                {
                    let in_ = ctx.state.get_facts_at(&function.name, pc)?;
                    let out_ = ctx.state.get_facts_at(&function.name, *jump_to_block)?;

                    for (from, after) in in_
                        .zip(out_)
                        .map(|(from, after)| (from.clone(), after.clone()))
                    {
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

                        for (from, after) in in_
                            .zip(out_)
                            .map(|(from, after)| (from.clone(), after.clone()))
                        {
                            ctx.graph.add_normal_curved(from.clone(), after.clone())?;
                        }
                    } else {
                        bail!("Cannot find block to jump to");
                    }
                }

                // NORMAL

                let in_ = ctx.state.get_facts_at(&function.name, pc)?;
                let out_ = ctx.state.get_facts_at(&function.name, pc + 1)?;

                for (from, after) in in_
                    .zip(out_)
                    .map(|(from, after)| (from.clone(), after.clone()))
                {
                    ctx.graph.add_normal(from.clone(), after.clone())?;
                }
            }
            Instruction::Block(_) => {
                let in_ = ctx.state.get_facts_at(&function.name, pc)?;
                let out_ = ctx.state.get_facts_at(&function.name, pc + 1)?;

                for (from, after) in in_
                    .zip(out_)
                    .map(|(from, after)| (from.clone(), after.clone()))
                {
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

                        for (from, after) in in_
                            .zip(out_)
                            .map(|(from, after)| (from.clone(), after.clone()))
                        {
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
                    .filter(|x| !(x.var_is_memory && x.memory_offset == Some(*offset)));

                for (from, after) in in_
                    .zip(out_)
                    .map(|(from, after)| (from.clone(), after.clone()))
                {
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
                        .add_memory_var(function.name.clone(), offset.clone());

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
                            memory_offset: var.memory_offset.clone(),
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
            Instruction::Load(dest, _offset, src) => {
                let in_ = ctx.state.get_facts_at(&function.name, pc)?;
                let out_ = ctx
                    .state
                    .get_facts_at(&function.name, pc + 1)?
                    .filter(|x| &x.belongs_to_var != dest);

                for (from, after) in in_
                    .zip(out_)
                    .map(|(from, after)| (from.clone(), after.clone()))
                {
                    ctx.graph.add_normal(from.clone(), after.clone())?;
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

                for (from, after) in in_
                    .zip(out_)
                    .map(|(from, after)| (from.clone(), after.clone()))
                {
                    ctx.graph.add_normal(from.clone(), after.clone())?;
                }
            }
            _ => {
                panic!("Instruction {:?} not implemented", instruction);
            }
        }

        Ok(())
    }
}
