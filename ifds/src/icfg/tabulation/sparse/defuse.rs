#![allow(dead_code)]

use crate::icfg::{flowfuncs::BlockResolver, graph::*};
use crate::ir::ast::Function as AstFunction;
use crate::ir::ast::Instruction;
use anyhow::{Context, Result};
use log::debug;
use std::collections::HashMap;
use std::collections::VecDeque;

use crate::icfg::tabulation::sparse::Ctx;

type Function = String;
type Var = String;
type StartPC = usize;

#[derive(Debug, Default)]
pub struct DefUseChain {
    inner: HashMap<(Function, Var), (StartPC, Graph)>,
}

type PC = usize;
#[derive(Debug, Clone)]
enum SCFG {
    Instruction(PC, Instruction),
    Conditional(PC, Instruction, Vec<SCFG>, Vec<SCFG>),
    ConditionalJump(PC, Instruction, PC),
    Jump(PC, PC),
    FunctionEnd(PC),
}

impl SCFG {
    pub fn get_pc(&self) -> PC {
        match self {
            SCFG::Conditional(pc, ..) => *pc,
            SCFG::Instruction(pc, ..) => *pc,
            SCFG::ConditionalJump(pc, ..) => *pc,
            SCFG::Jump(pc, _jump_to_pc) => *pc,
            SCFG::FunctionEnd(pc, ..) => *pc,
        }
    }
}

impl DefUseChain {
    /// Get the DefUseChain for function and variable
    pub fn get_graph(&self, function: &String, var: &String) -> Option<&Graph> {
        self.inner
            .get(&(function.clone(), var.clone()))
            .map(|(_, x)| x)
    }

    /// Get the facts in the graph.
    pub fn get_facts_at(&self, function: &String, var: &String, pc: usize) -> Result<Vec<&Fact>> {
        let graph = self.get_graph(function, var).context("Cannot find graph")?;
        let facts = graph
            .flatten()
            .into_iter()
            .filter(|x| x.pc == pc)
            .collect::<Vec<_>>();

        Ok(facts)
    }

    /// Cache and get next
    pub fn demand<'a>(
        &mut self,
        ctx: &mut Ctx<'a>,
        function: &AstFunction,
        var: &String,
        pc: usize,
    ) -> Result<Vec<Fact>> {
        let graph = self.cache(ctx, function, var, pc)?;

        let x = graph
            .flatten()
            .into_iter()
            .filter(|x| x.pc > pc)
            .map(|x| x.clone())
            .collect::<Vec<_>>();

        Ok(x)
    }

    /// Cache and get next
    pub fn demand_inclusive<'a>(
        &mut self,
        ctx: &mut Ctx<'a>,
        function: &AstFunction,
        var: &String,
        pc: usize,
    ) -> Result<Vec<&Fact>> {
        debug!("Querying demand_inclusive for {} at {}", var, pc);
        let graph = self.cache(ctx, function, var, pc)?;

        let xx = graph.flatten().collect::<Vec<_>>();
        debug!("xx (all) for {} at {} {:#?}", var, pc, xx);

        let mut queue: VecDeque<&Fact> = VecDeque::new();
        let mut seen = Vec::new();

        // entry fact has a loop
        let is_entry = |x: &Fact| x.pc == x.next_pc && x.pc == pc && x.next_pc == pc;

        if let Some(start) = graph
            .edges
            .iter()
            .map(|x| x.get_from())
            .chain(graph.edges.iter().map(|x| x.to()))
            .find(|x| x.pc == pc && !is_entry(x))
        {
            debug!("Adding start {:#?}", start);
            queue.push_back(start);
        } else {
            log::warn!(
                "No start fact found. Therefore skipping for {} at {}",
                var,
                pc
            );
            return Ok(Vec::new());
        }

        while let Some(node) = queue.pop_front() {
            debug!("Popping node {:?}", node);
            seen.push(node);
            for child in graph
                .edges
                .iter()
                .filter(|x| x.get_from() == node && !is_entry(x.get_from()))
                .map(|x| x.to())
            {
                if !seen.contains(&child) {
                    debug!("queue child {:?}", child);
                    queue.push_back(child);
                }
            }
        }

        debug!("xx (filtered) for {} at {} {:#?}", var, pc, seen);

        Ok(seen)
    }

    pub fn get_next<'a>(
        &mut self,
        ctx: &mut Ctx<'a>,
        function: &AstFunction,
        var: &String,
        old_pc: usize,
    ) -> Result<Vec<Fact>> {
        let graph = self.cache(ctx, function, var, old_pc)?;

        let facts = graph
            .flatten()
            .into_iter()
            .filter(|x| x.pc == old_pc)
            .map(|x| x.clone())
            .collect::<Vec<_>>();

        Ok(facts)
    }

    // nodes which point to (var, pc)
    pub fn points_to<'a>(
        &mut self,
        ctx: &mut Ctx<'a>,
        function: &AstFunction,
        var: &String,
        pc: usize,
    ) -> Result<Vec<Fact>> {
        let graph = self.cache(ctx, function, var, pc)?;

        let all_facts = graph.flatten().collect::<Vec<_>>();

        let facts = all_facts
            .into_iter()
            .filter(|x| x.next_pc <= pc)
            .collect::<Vec<_>>();

        let next_pc = facts.iter().map(|x| x.next_pc).max().unwrap_or(0);

        // Get all next nodes, because there might be multiple
        let x: Vec<_> = facts
            .into_iter()
            .filter(|x| x.next_pc == next_pc)
            .map(|x| x.clone())
            .collect();

        Ok(x)
    }

    pub fn get_start_pc(&self, function: &AstFunction, var: &String) -> Option<usize> {
        self.get_start_pc_by_name(&function.name, var)
    }

    pub fn get_start_pc_by_name(&self, function: &String, var: &String) -> Option<usize> {
        self.inner
            .get(&(function.clone(), var.clone()))
            .map(|(pc, _)| *pc)
    }

    /// Build the defuse chain for the instruction
    /// The precondition is that the function must be already initialized.
    /// Because we need the track of the given variable `var`.
    pub(crate) fn cache<'a>(
        &mut self,
        ctx: &mut Ctx<'a>,
        function: &AstFunction,
        var: &String,
        pc: usize,
    ) -> Result<&Graph> {
        let var = ctx
            .state
            .get_var(&function.name, var)
            .context("Variable is not defined in the state")?
            .clone();

        // already exists
        if self
            .inner
            .contains_key(&(function.name.clone(), var.name.clone()))
        {
            let start_pc = self
                .get_start_pc(function, &var.name)
                .context("Cannot get start_pc")?;

            // If `pc` is lower than `start_pc`, then delete and continue
            if pc > start_pc {
                debug!("Cached.");
                let x = self
                    .inner
                    .get(&(function.name.clone(), var.name.clone()))
                    .map(|(_, x)| x)
                    .context("Cannot get graph")?;

                return Ok(x);
            } else {
                debug!("Cache is old. Removing");
                self.inner
                    .remove(&(function.name.clone(), var.name.clone()));
            }
        }

        let track = ctx
            .state
            .get_track(&function.name, &var.name)
            .context("Cannot find track of var")?;

        let instructions: Vec<_> = function.instructions.iter().enumerate().collect();

        let max_len = instructions.len();
        let scfg = self.build_next2(function, instructions, &ctx.block_resolver)?;

        debug!("scfg {:#?}", scfg);

        let mut graph = Graph::default();
        self.build_graph(
            function,
            &scfg,
            &ctx.block_resolver,
            max_len,
            &var,
            track,
            pc,
            false,
            true,
            &mut graph,
        )?;

        {
            self.inner
                .insert((function.name.clone(), var.name.clone()), (pc, graph));
        }

        let (_, ref graph) = self
            .inner
            .get(&(function.name.clone(), var.name.clone()))
            .context("Cannot find chained facts")?;

        Ok(graph)
    }

    fn take_branch<'a>(
        &self,
        function: &'a AstFunction,
        skip: usize,
        take: usize,
    ) -> impl Iterator<Item = (usize, &'a Instruction)> {
        let mut instructions = function.instructions.iter().enumerate();

        for _ in 0..skip {
            instructions.next();
        }

        instructions.take(take)
    }

    fn build_graph<'a>(
        &'a self,
        function: &AstFunction,
        instructions: &Vec<SCFG>,
        block_resolver: &BlockResolver,
        max_len: usize,
        var: &Variable,
        track: usize,
        start_pc: usize,
        is_defined: bool,
        is_top_level: bool,
        graph: &mut Graph,
    ) -> Result<Fact> {
        let get_relevant_instructions =
            |instructions: Vec<SCFG>, mut is_defined: bool, max_level: usize| {
                let mut relevant_instructions = Vec::new();
                let mut overwritten = false;

                for instruction in instructions.into_iter() {
                    match instruction {
                        SCFG::Instruction(_pc, ref inner_instruction) => {
                            let is_lhs = self.is_lhs_used(&var, &inner_instruction);
                            let is_rhs = self.is_rhs_used(&var, &inner_instruction);

                            if is_lhs {
                                if !is_defined {
                                    is_defined = true;
                                    relevant_instructions.push(instruction.clone());
                                } else {
                                    overwritten = true;
                                    break;
                                }
                            } else {
                                if is_rhs {
                                    relevant_instructions.push(instruction.clone());
                                }
                            }
                        }
                        SCFG::Jump(_pc, _jump_to_pc) => {
                            relevant_instructions.push(instruction.clone());
                            break;
                        }
                        _ => {
                            relevant_instructions.push(instruction.clone());
                        }
                    }
                }

                if is_top_level && !overwritten {
                    relevant_instructions.push(SCFG::FunctionEnd(max_level));
                }

                (is_defined, relevant_instructions)
            };

        let (is_defined, relevant_instructions) =
            get_relevant_instructions(instructions.clone(), is_defined, max_len);

        //debug!("rel {} {:#?}", var.name, relevant_instructions);

        let first = Fact::from_var(
            var,
            start_pc,
            relevant_instructions
                .first()
                .map(|x| x.get_pc())
                .unwrap_or(max_len),
            track,
        );
        let mut node = first.clone();
        let mut i = 0;

        // Cannot simply add next instruction, we have to check to look
        // if it is a conditional
        // if yes, then look for the next inner instruction
        // if not, then ok

        for instruction in relevant_instructions.iter() {
            match instruction {
                SCFG::Instruction(pc, _instruction) => {
                    let x = self
                        .add_next_instruction(
                            &relevant_instructions,
                            i,
                            function,
                            block_resolver,
                            var,
                            track,
                            is_defined,
                            graph,
                            pc,
                            &node,
                            max_len,
                        )
                        .context("Adding next instruction failed")?;

                    node = x;
                }
                SCFG::Conditional(pc, _instruction, _block1, _block2) => {
                    let x = self
                        .add_next_instruction(
                            &relevant_instructions,
                            i,
                            function,
                            block_resolver,
                            var,
                            track,
                            is_defined,
                            graph,
                            pc,
                            &node,
                            max_len,
                        )
                        .context("Adding next instruction failed")?;

                    node = x;
                }
                SCFG::Jump(pc, jump_to_pc) => {
                    log::error!("Jump to pc is {}", jump_to_pc);
                    let next = jump_to_pc;
                    debug!("Edge from {} to {} for {}", pc, next, var.name);
                    let x = Fact::from_var(var, *pc, *next, track);
                    graph.add_normal(node.clone(), x.clone())?;
                    node = x;
                }
                SCFG::ConditionalJump(pc, _instruction, jump_to_pc) => {
                    // One edge back
                    log::error!("Jump to pc is {}", jump_to_pc);
                    let next = jump_to_pc;
                    debug!("Edge from {} to {} for {}", pc, next, var.name);
                    let x = Fact::from_var(var, *pc, *next, track);
                    graph.add_normal(node.clone(), x.clone())?;

                    // one edge goes next
                    let x = self
                        .add_next_instruction(
                            &relevant_instructions,
                            i,
                            function,
                            block_resolver,
                            var,
                            track,
                            is_defined,
                            graph,
                            pc,
                            &node,
                            max_len,
                        )
                        .context("Adding next instruction failed")?;

                    node = x;
                }
                SCFG::FunctionEnd(pc) => {
                    let next = pc;
                    debug!("Edge from {} to {} for {}", pc, next, var.name);
                    let x = Fact::from_var(var, *pc, *next, track);
                    graph.add_normal(node.clone(), x.clone())?;
                    node = x;
                }
            }

            i += 1;
        }

        Ok(node)
    }

    fn add_next_instruction<'a>(
        &'a self,
        relevant_instructions: &Vec<SCFG>,
        i: usize,
        function: &AstFunction,
        block_resolver: &HashMap<(String, String), usize>,
        var: &Variable,
        track: usize,
        is_defined: bool,
        graph: &mut Graph,
        pc: &usize,
        node: &Fact,
        max_len: usize,
    ) -> Result<Fact> {
        if let Some(next_instruction) = relevant_instructions.get(i + 1) {
            // Check if conditional
            match next_instruction {
                SCFG::Conditional(_pc, _instruction, block1, block2) => {
                    // We have to look a step further
                    let next = relevant_instructions
                        .get(i + 2)
                        .map(|x| x.get_pc())
                        .unwrap_or(max_len);

                    log::warn!(
                        "Before building next graph: meet pc {} {:?}",
                        next,
                        _instruction
                    );

                    let _res1 = self.build_graph(
                        function,
                        block1,
                        block_resolver,
                        next,
                        var,
                        track,
                        0,
                        is_defined,
                        false,
                        graph,
                    )?;

                    let _res2 = self.build_graph(
                        function,
                        block2,
                        block_resolver,
                        next,
                        var,
                        track,
                        0,
                        is_defined,
                        false,
                        graph,
                    )?;

                    let x = Fact::from_var(var, *pc, next, track);
                    graph.add_normal(node.clone(), x.clone())?;
                    debug!("Edge from {} to {} for {}", pc, next, var.name);

                    Ok(x)
                }
                _ => {
                    let next = next_instruction.get_pc();
                    let x = Fact::from_var(var, *pc, next, track);
                    graph.add_normal(node.clone(), x.clone())?;
                    debug!("Edge from {} to {} for {}", pc, next, var.name);

                    Ok(x)
                }
            }
        } else {
            let next = max_len;

            let x = Fact::from_var(var, *pc, next, track);
            graph.add_normal(node.clone(), x.clone())?;

            Ok(x)
        }
    }

    fn build_next2<'a>(
        &'a self,
        function: &AstFunction,
        instructions: Vec<(usize, &Instruction)>,
        block_resolver: &BlockResolver,
    ) -> Result<Vec<SCFG>> {
        let mut main = Vec::new();
        let mut i = 0;
        while i < instructions.len() {
            let ref_instruction = instructions.get(i).context("Cannot find instruction")?;
            let (pc, inner_instruction) = ref_instruction;
            debug!("Instruction {:?}", inner_instruction);
            match inner_instruction {
                Instruction::Conditional(_, jumps) if jumps.len() == 2 => {
                    let after_cond = jumps
                        .last()
                        .unwrap()
                        .parse::<usize>()
                        .context("Jump is not a number")?
                        + 1;
                    let end_cond_pc = block_resolver
                        .get(&(function.name.clone(), format!("{}", after_cond)))
                        .context("Cannot find the end of the conditional")?;

                    let after_first_block = jumps
                        .get(1)
                        .unwrap()
                        .parse::<usize>()
                        .context("Jump is not a number")?
                        - 1; //last instruction of the first_block
                    let last_pc_first_block = block_resolver
                        .get(&(function.name.clone(), format!("{}", after_first_block)))
                        .context("Cannot find the end of the first block")?;

                    let second_block_id = jumps
                        .get(1)
                        .unwrap()
                        .parse::<usize>()
                        .context("Jump is not a number")?;

                    let first_pc_second_block = block_resolver
                        .get(&(function.name.clone(), format!("{}", second_block_id)))
                        .context("Cannot find the end of the second block")?;

                    let first_branch = self
                        .take_branch(function, pc + 2, last_pc_first_block - pc)
                        .collect::<Vec<_>>();
                    debug!("first {:#?}", first_branch);
                    assert_eq!(last_pc_first_block - pc, first_branch.len());
                    let second_branch = self
                        .take_branch(
                            function,
                            first_pc_second_block + 1,
                            end_cond_pc - first_pc_second_block - 1,
                        )
                        .collect::<Vec<_>>();
                    debug!("second {:#?}", second_branch);
                    assert_eq!(end_cond_pc - first_pc_second_block - 1, second_branch.len());

                    let first_branch = self.build_next2(function, first_branch, block_resolver)?;
                    let second_branch =
                        self.build_next2(function, second_branch, block_resolver)?;

                    main.push(SCFG::Conditional(
                        *pc,
                        inner_instruction.clone().clone(),
                        first_branch,
                        second_branch,
                    ));

                    i = *end_cond_pc + 1; //skip all conditionals
                    debug!("Setting i to {}", i);
                }
                Instruction::Conditional(_, jumps) if jumps.len() == 1 => {
                    let jump_to_block = jumps.first().context("Cannot get label")?;
                    let jump_to_pc = block_resolver
                        .get(&(function.name.clone(), jump_to_block.clone()))
                        .context("Cannot find the block")?;

                    main.push(SCFG::ConditionalJump(
                        *pc,
                        inner_instruction.clone().clone(),
                        jump_to_pc.checked_sub(1).unwrap_or(*jump_to_pc),
                    ));
                    i += 1;
                    debug!("Setting i to {}", i);
                }
                Instruction::Conditional(_, _) => {
                    unimplemented!()
                }
                Instruction::Jump(block) => {
                    let jump_to_pc = block_resolver
                        .get(&(function.name.clone(), block.clone()))
                        .context("Cannot find the block")?;
                    main.push(SCFG::Jump(
                        *pc,
                        jump_to_pc.checked_sub(1).unwrap_or(*jump_to_pc),
                    ));
                    i += 1;
                    debug!("Setting i to {}", i);
                }
                _ => {
                    main.push(SCFG::Instruction(*pc, inner_instruction.clone().clone()));
                    i += 1;
                    debug!("Setting i to {}", i);
                }
            }
        }

        Ok(main)
    }

    fn is_lhs_used(&self, variable: &Variable, instruction: &Instruction) -> bool {
        let var = &variable.name;

        match instruction {
            Instruction::Const(dest, _) if dest == var => true,
            Instruction::Assign(dest, _src) if dest == var => true,
            Instruction::BinOp(dest, _, _) if dest == var => true,
            _ => false,
        }
    }

    fn is_rhs_used(&self, variable: &Variable, instruction: &Instruction) -> bool {
        let var = &variable.name;

        match instruction {
            Instruction::Assign(_dest, src) if src == var => true,
            Instruction::BinOp(_, src1, src2) if src1 == var || src2 == var => true,
            _ => false,
        }
    }

    fn is_used_mem(&self, _variable: &Variable, instruction: &Instruction) -> bool {
        macro_rules! is_mem {
            ($e:ident) => {
                $e.starts_with("mem")
            };
        }

        match instruction {
            Instruction::Unop(dest, src) => is_mem!(dest) || is_mem!(src),
            Instruction::BinOp(dest, src1, src2) => is_mem!(dest) || is_mem!(src1) || is_mem!(src2),
            Instruction::Const(dest, _) => is_mem!(dest),
            Instruction::Assign(dest, src) => is_mem!(dest) || is_mem!(src),
            Instruction::Call(_callee, params, dest) => {
                params.iter().any(|x| is_mem!(x)) || dest.iter().any(|x| is_mem!(x))
            }
            Instruction::CallIndirect(_callee, params, dest) => {
                params.iter().any(|x| is_mem!(x)) || dest.iter().any(|x| is_mem!(x))
            }
            Instruction::Kill(dest) => is_mem!(dest),
            Instruction::Conditional(dest, _) => is_mem!(dest),
            Instruction::Return(dest) => dest.iter().any(|x| is_mem!(x)),
            Instruction::Phi(dest, src1, src2) => is_mem!(dest) || is_mem!(src1) || is_mem!(src2),
            Instruction::Store(src1, _, src2) => is_mem!(src1) || is_mem!(src2),
            Instruction::Load(dest, _, src) => is_mem!(dest) || is_mem!(src),
            _ => false,
        }
    }

    fn is_used_taut(&self, _variable: &Variable, instruction: &Instruction) -> bool {
        match instruction {
            Instruction::Const(_dest, _) => true,
            _ => false,
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::icfg::state::State;
    use crate::ir::ast::Program;
    use insta::assert_debug_snapshot as assert_snapshot;

    fn resolve_block_ids<'a>(
        ctx: &mut Ctx<'a>,
        function: &AstFunction,
        start_pc: usize,
    ) -> Result<()> {
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
                    panic!("")
                }
            }
        }

        Ok(())
    }

    #[test]
    fn test_building_loop_scfg() {
        let func_name = "main".to_string();
        let function = AstFunction {
            name: func_name.clone(),
            definitions: vec!["%0".to_string(), "%1".to_string(), "%2".to_string()],
            instructions: vec![
                Instruction::Const("%0".to_string(), 1.0),
                Instruction::Block("0".to_string()),
                Instruction::BinOp("%2".to_string(), "%1".to_string(), "%0".to_string()),
                Instruction::BinOp("%2".to_string(), "%1".to_string(), "%0".to_string()),
                Instruction::Block("2".to_string()),
                Instruction::Jump("0".to_string()),
                Instruction::Block("3".to_string()),
            ],
            ..Default::default()
        };

        let mut graph = Graph::default();
        let mut state = State::default();

        let mut ctx = Ctx {
            graph: &mut graph,
            state: &mut state,
            prog: &Program {
                functions: vec![function.clone()],
            },
            block_resolver: HashMap::default(),
        };

        let pc = 0;

        // fullfilling precondition of `chain.cache()`
        ctx.state.init_function(&function, pc).unwrap();
        resolve_block_ids(&mut ctx, &function, pc).unwrap();

        let mut chain = DefUseChain::default();
        let facts = chain
            .cache(&mut ctx, &function, &"%0".to_string(), pc)
            .unwrap()
            .flatten()
            .collect::<Vec<_>>();

        assert_snapshot!("building_loop_defuse_reg_0_scfg", facts);
    }

    #[test]
    fn test_building_conditional_if_scfg() {
        let func_name = "main".to_string();
        let function = AstFunction {
            name: func_name.clone(),
            definitions: vec!["%0".to_string(), "%1".to_string(), "%2".to_string()],
            instructions: vec![
                Instruction::Const("%0".to_string(), 1.0),
                Instruction::Block("0".to_string()),
                Instruction::BinOp("%2".to_string(), "%1".to_string(), "%0".to_string()),
                Instruction::BinOp("%2".to_string(), "%1".to_string(), "%0".to_string()),
                Instruction::Block("2".to_string()),
                Instruction::Conditional("%0".to_string(), vec!["0".to_string()]),
                Instruction::Block("3".to_string()),
            ],
            ..Default::default()
        };

        let mut graph = Graph::default();
        let mut state = State::default();

        let mut ctx = Ctx {
            graph: &mut graph,
            state: &mut state,
            prog: &Program {
                functions: vec![function.clone()],
            },
            block_resolver: HashMap::default(),
        };

        let pc = 0;

        // fullfilling precondition of `chain.cache()`
        ctx.state.init_function(&function, pc).unwrap();
        resolve_block_ids(&mut ctx, &function, pc).unwrap();

        let mut chain = DefUseChain::default();
        let facts = chain
            .cache(&mut ctx, &function, &"%0".to_string(), pc)
            .unwrap()
            .flatten()
            .collect::<Vec<_>>();

        assert_snapshot!("building_conditional_jump_defuse_reg_0_scfg", facts);
    }

    #[test]
    fn test_building_conditional_scfg() {
        let func_name = "main".to_string();
        let function = AstFunction {
            name: func_name.clone(),
            definitions: vec!["%0".to_string(), "%1".to_string(), "%2".to_string()],
            instructions: vec![
                Instruction::Const("%0".to_string(), 1.0),
                Instruction::Const("%1".to_string(), 1.0),
                Instruction::Conditional("%1".to_string(), vec!["0".to_string(), "1".to_string()]),
                Instruction::Block("0".to_string()),
                Instruction::BinOp("%2".to_string(), "%0".to_string(), "%1".to_string()),
                Instruction::Block("1".to_string()),
                Instruction::BinOp("%2".to_string(), "%1".to_string(), "%0".to_string()),
                Instruction::BinOp("%2".to_string(), "%1".to_string(), "%0".to_string()),
                Instruction::Block("2".to_string()),
                Instruction::BinOp("%2".to_string(), "%1".to_string(), "%0".to_string()),
                Instruction::Block("3".to_string()),
            ],
            ..Default::default()
        };

        let mut graph = Graph::default();
        let mut state = State::default();

        let mut ctx = Ctx {
            graph: &mut graph,
            state: &mut state,
            prog: &Program {
                functions: vec![function.clone()],
            },
            block_resolver: HashMap::default(),
        };

        let pc = 0;

        // fullfilling precondition of `chain.cache()`
        ctx.state.init_function(&function, pc).unwrap();
        resolve_block_ids(&mut ctx, &function, pc).unwrap();

        let mut chain = DefUseChain::default();
        let facts = chain
            .cache(&mut ctx, &function, &"%0".to_string(), pc)
            .unwrap()
            .flatten()
            .collect::<Vec<_>>();

        assert_snapshot!("building_conditional_defuse_reg_0_scfg", facts);
    }

    #[test]
    fn test_building_scfg() {
        /*
            0 - %0 = 1
            1 - %1 = 1
            2 - %2 = %0 op %1
            3 - %2 = %1 op %0

            %0:
                0 -> 2
                2 -> 3

            %1:
                0 -> 1
                1 -> 2
                2 -> 3
        */
        let func_name = "main".to_string();
        let function = AstFunction {
            name: func_name.clone(),
            definitions: vec!["%0".to_string(), "%1".to_string(), "%2".to_string()],
            instructions: vec![
                Instruction::Const("%0".to_string(), 1.0),
                Instruction::Const("%1".to_string(), 1.0),
                Instruction::BinOp("%2".to_string(), "%0".to_string(), "%1".to_string()),
                Instruction::BinOp("%2".to_string(), "%1".to_string(), "%0".to_string()),
            ],
            ..Default::default()
        };

        let mut graph = Graph::default();
        let mut state = State::default();

        let mut ctx = Ctx {
            graph: &mut graph,
            state: &mut state,
            prog: &Program {
                functions: vec![function.clone()],
            },
            block_resolver: HashMap::default(),
        };

        let pc = 0;

        // fullfilling precondition of `chain.cache()`
        ctx.state.init_function(&function, pc).unwrap();

        let mut chain = DefUseChain::default();
        let facts = chain
            .cache(&mut ctx, &function, &"%0".to_string(), pc)
            .unwrap()
            .flatten()
            .collect::<Vec<_>>();

        assert_snapshot!("building_defuse_reg_0_scfg", facts);

        let facts = chain
            .cache(&mut ctx, &function, &"%1".to_string(), pc)
            .unwrap()
            .flatten()
            .collect::<Vec<_>>();

        assert_snapshot!("building_defuse_reg_1_scfg", facts);

        let facts = chain
            .cache(&mut ctx, &function, &"%2".to_string(), pc)
            .unwrap()
            .flatten()
            .collect::<Vec<_>>();

        assert_snapshot!("building_defuse_reg_2_scfg", facts);
    }

    #[test]
    fn test_building_scfg2() {
        let func_name = "main".to_string();
        let function = AstFunction {
            name: func_name.clone(),
            definitions: vec!["%0".to_string(), "%1".to_string(), "%2".to_string()],
            instructions: vec![
                Instruction::Const("%0".to_string(), 1.0),
                Instruction::Const("%1".to_string(), 1.0),
                Instruction::BinOp("%2".to_string(), "%0".to_string(), "%1".to_string()),
                Instruction::BinOp("%3".to_string(), "%1".to_string(), "%0".to_string()),
            ],
            ..Default::default()
        };

        let mut graph = Graph::default();
        let mut state = State::default();

        let mut ctx = Ctx {
            graph: &mut graph,
            state: &mut state,
            prog: &Program {
                functions: vec![function.clone()],
            },
            block_resolver: HashMap::default(),
        };

        let pc = 0;

        // fullfilling precondition of `chain.cache()`
        ctx.state.init_function(&function, pc).unwrap();

        let mut chain = DefUseChain::default();

        let facts = chain
            .cache(&mut ctx, &function, &"%2".to_string(), 2)
            .unwrap()
            .flatten()
            .collect::<Vec<_>>();

        assert_snapshot!("building_defuse2_reg_2_scfg_pc_2", facts);
    }

    #[test]
    fn test_building_scfg3() {
        let func_name = "main".to_string();
        let function = AstFunction {
            name: func_name.clone(),
            definitions: vec!["%0".to_string(), "%1".to_string(), "%2".to_string()],
            instructions: vec![
                Instruction::Const("%0".to_string(), 1.0),
                Instruction::Const("%1".to_string(), 1.0),
                Instruction::BinOp("%2".to_string(), "%0".to_string(), "%1".to_string()),
                Instruction::BinOp("%3".to_string(), "%1".to_string(), "%0".to_string()),
            ],
            ..Default::default()
        };

        let mut graph = Graph::default();
        let mut state = State::default();

        let mut ctx = Ctx {
            graph: &mut graph,
            state: &mut state,
            prog: &Program {
                functions: vec![function.clone()],
            },
            block_resolver: HashMap::default(),
        };

        let pc = 0;

        // fullfilling precondition of `chain.cache()`
        ctx.state.init_function(&function, pc).unwrap();

        let mut chain = DefUseChain::default();

        let facts = chain
            .cache(&mut ctx, &function, &"%2".to_string(), pc)
            .unwrap()
            .flatten()
            .collect::<Vec<_>>();

        assert_snapshot!("building_defuse3_reg_2_scfg_pc_0", facts);
    }

    #[test]
    fn testing_caching_first_var() {
        let func_name = "main".to_string();
        let function = AstFunction {
            name: func_name.clone(),
            definitions: vec!["%0".to_string(), "%1".to_string(), "%2".to_string()],
            instructions: vec![
                Instruction::Const("%0".to_string(), 1.0),
                Instruction::Const("%1".to_string(), 1.0),
                Instruction::Const("%2".to_string(), 1.0),
                Instruction::Const("%0".to_string(), 1.0),
                Instruction::Const("%1".to_string(), 1.0),
            ],
            ..Default::default()
        };

        let mut graph = Graph::default();
        let mut state = State::default();

        let mut ctx = Ctx {
            graph: &mut graph,
            state: &mut state,
            prog: &Program {
                functions: vec![function.clone()],
            },
            block_resolver: HashMap::default(),
        };

        let pc = 0;

        // fullfilling precondition of `chain.cache()`
        ctx.state.init_function(&function, pc).unwrap();

        let mut chain = DefUseChain::default();
        let facts = chain
            .cache(&mut ctx, &function, &"%0".to_string(), pc)
            .unwrap()
            .flatten()
            .collect::<Vec<_>>();

        assert_snapshot!("defuse_reg_0_scfg", facts);

        assert_eq!(2, facts.len());
        assert_eq!(0, facts.get(0).unwrap().next_pc);

        let before = chain
            .points_to(&mut ctx, &function, &"%0".to_string(), 3)
            .unwrap();
        assert_eq!(1, before.len());
        assert_eq!(0, before.get(0).unwrap().next_pc);

        let after = chain
            .demand(&mut ctx, &function, &"%0".to_string(), 0)
            .unwrap();
        assert_eq!(0, after.len());
    }

    #[test]
    fn testing_caching_second_var() {
        let func_name = "main".to_string();
        let function = AstFunction {
            name: func_name.clone(),
            definitions: vec!["%0".to_string(), "%1".to_string(), "%2".to_string()],
            instructions: vec![
                Instruction::Const("%0".to_string(), 1.0),
                Instruction::Const("%1".to_string(), 1.0),
                Instruction::Const("%2".to_string(), 1.0),
                Instruction::Const("%0".to_string(), 1.0),
                Instruction::Const("%1".to_string(), 1.0),
            ],
            ..Default::default()
        };

        let mut graph = Graph::default();
        let mut state = State::default();

        let mut ctx = Ctx {
            graph: &mut graph,
            state: &mut state,
            prog: &Program {
                functions: vec![function.clone()],
            },
            block_resolver: HashMap::default(),
        };

        let pc = 0;

        // fullfilling precondition of `chain.cache()`
        ctx.state.init_function(&function, pc).unwrap();

        let mut chain = DefUseChain::default();
        let facts = chain
            .cache(&mut ctx, &function, &"%1".to_string(), pc)
            .unwrap()
            .flatten()
            .collect::<Vec<_>>();

        assert_snapshot!("defuse_reg_1_scfg", facts);

        assert_eq!(2, facts.len());
        assert_eq!(5, facts.get(1).unwrap().next_pc);

        let before = chain
            .points_to(&mut ctx, &function, &"%1".to_string(), 1)
            .unwrap();
        assert_eq!(1, before.len());
        assert_eq!(1, before.get(0).unwrap().next_pc);

        let after = chain
            .demand(&mut ctx, &function, &"%1".to_string(), 1)
            .unwrap();
        assert_eq!(0, after.len());
    }

    #[test]
    fn testing_caching_recursion() {
        let func_name = "main".to_string();
        let function = AstFunction {
            name: func_name.clone(),
            definitions: vec!["%0".to_string(), "%1".to_string(), "%2".to_string()],
            instructions: vec![
                Instruction::Const("%0".to_string(), 1.0),
                Instruction::Const("%1".to_string(), 1.0),
                Instruction::Const("%2".to_string(), 1.0),
                Instruction::Const("%0".to_string(), 1.0),
                Instruction::Const("%1".to_string(), 1.0),
            ],
            ..Default::default()
        };

        let mut graph = Graph::default();
        let mut state = State::default();

        let mut ctx = Ctx {
            graph: &mut graph,
            state: &mut state,
            prog: &Program {
                functions: vec![function.clone()],
            },
            block_resolver: HashMap::default(),
        };

        let pc = 2;

        // fullfilling precondition of `chain.cache()`
        ctx.state.init_function(&function, pc).unwrap();

        let mut chain = DefUseChain::default();
        let facts = chain
            .cache(&mut ctx, &function, &"%0".to_string(), pc)
            .unwrap()
            .flatten()
            .collect::<Vec<_>>();
        assert_eq!(2, facts.len());

        let facts = chain
            .cache(&mut ctx, &function, &"%0".to_string(), 0)
            .unwrap()
            .flatten()
            .collect::<Vec<_>>();
        assert_eq!(2, facts.len());
    }
}
