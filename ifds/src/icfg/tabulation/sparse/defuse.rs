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
    ConditionalSingle(PC, Instruction, PC, Vec<SCFG>),
    ConditionalJump(PC, Instruction, PC),
    Jump(PC, PC), //unconditional
    Table(PC, Instruction, Vec<PC>),
    FunctionEnd(PC),
}

impl SCFG {
    pub fn get_pc(&self) -> PC {
        match self {
            SCFG::Conditional(pc, ..) => *pc,
            SCFG::Instruction(pc, ..) => *pc,
            SCFG::ConditionalSingle(pc, ..) => *pc,
            SCFG::ConditionalJump(pc, ..) => *pc,
            SCFG::Jump(pc, _jump_to_pc) => *pc,
            SCFG::Table(pc, ..) => *pc,
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
    pub fn get_facts_at<'a>(
        &mut self,
        ctx: &mut Ctx<'a>,
        function: &AstFunction,
        var: &String,
        pc: usize,
    ) -> Result<Vec<&Fact>> {
        debug!("Get facts for {} ({}) at {}", function.name, var, pc);
        let graph = self.cache(ctx, function, var, pc).with_context(|| {
            format!(
                "Cannot find graph for {} (func {}) at {}",
                var, function.name, pc
            )
        })?;
        let all_facts = graph.flatten().into_iter().collect::<Vec<_>>();

        let facts = all_facts
            .into_iter()
            .filter(|x| x.pc == pc)
            .collect::<Vec<_>>();

        Ok(facts)
    }

    /// Get the facts in the graph.
    pub fn get_entry_fact<'a>(
        &mut self,
        ctx: &mut Ctx<'a>,
        function: &AstFunction,
        var: &String,
        start_pc: usize,
    ) -> Result<Fact> {
        let start = self.get_start_pc(function, var).unwrap_or(start_pc);

        debug!("Get facts for {} ({}) at {}", function.name, var, start);
        let graph = self.cache(ctx, function, var, start).with_context(|| {
            format!(
                "Cannot find graph for {} (func {}) at {}",
                var, function.name, start
            )
        })?;
        let all_facts = graph.flatten().into_iter().collect::<Vec<_>>();

        let facts = all_facts
            .into_iter()
            .filter(|x| x.pc == start && x.next_pc == start)
            .collect::<Vec<_>>();

        if let Some(fact) = facts.first() {
            return Ok(fact.clone().clone());
        } else {
            let var = ctx
                .state
                .get_var(&function.name, var)
                .context("Cannot find var")?;
            let track = ctx
                .state
                .get_track(&function.name, &var.name)
                .context("Cannot find track")?;

            let first = Fact::from_var(var, start_pc, start_pc, track);

            return Ok(first);
        }
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

    /// Works like `demand`, but also includes the current instruction.
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

        // entry fact has a loop
        let is_entry = |x: &&Fact| x.pc == x.next_pc && x.pc == old_pc && x.next_pc == old_pc;

        let all_facts = graph.flatten().collect::<Vec<_>>();

        let next_pcs = all_facts
            .iter()
            .filter(|x| x.pc == old_pc && !is_entry(x))
            .map(|x| x.next_pc);

        let mut facts = Vec::new();

        for next_pc in next_pcs {
            facts.extend(
                all_facts
                    .iter()
                    .filter(|x| x.pc == next_pc && !is_entry(x))
                    .map(|x| x.clone().clone()),
            )
        }

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

    /// If the `start_pc` is not the same as `pc`, then remove the cache entry.
    /// If the cache entry was removed, then return `true`.
    pub fn force_remove_if_outdated(
        &mut self,
        function: &AstFunction,
        var: &String,
        pc: usize,
    ) -> Result<bool> {
        debug!(
            "Checking if `start_pc` is the same as {} for {} ({})",
            pc, var, function.name
        );
        if let Some(start_pc) = self.get_start_pc(function, var) {
            if start_pc != pc {
                log::warn!(
                    "Force removal of outdated cache entry for {} ({}) at {}",
                    var,
                    function.name,
                    start_pc
                );

                self.inner.remove(&(function.name.clone(), var.clone()));

                return Ok(true);
            }
        }

        debug!("Cache is not outdated");

        Ok(false)
    }

    /// Build the defuse chain for the function, var and pc.
    /// Also, the `var` is not initialized in the parameters. So it assumes,
    /// that the first assignment is ok.
    /// The precondition is that the function must be already initialized.
    /// Because we need the track of the given variable `var`.
    pub fn cache<'a>(
        &mut self,
        ctx: &mut Ctx<'a>,
        function: &AstFunction,
        var: &String,
        pc: usize,
    ) -> Result<&Graph> {
        self.inner_cache(ctx, function, var, pc, false, false)
            .context("Caching failed")
    }

    /// Build the defuse chain for the function, var and pc.
    /// Also, the `var` was initialized in the parameters. Therefore,
    /// any following assignment will kill it.
    /// The precondition is that the function must be already initialized.
    /// Because we need the track of the given variable `var`.
    pub fn cache_when_already_defined<'a>(
        &mut self,
        ctx: &mut Ctx<'a>,
        function: &AstFunction,
        var: &String,
        pc: usize,
    ) -> Result<&Graph> {
        self.inner_cache(ctx, function, var, pc, true, true)
            .context("Caching when variable is already defined failed")
    }

    /// Build the defuse chain for the function, var and pc.
    /// The precondition is that the function must be already initialized.
    /// Because we need the track of the given variable `var`.
    fn inner_cache<'a>(
        &mut self,
        ctx: &mut Ctx<'a>,
        function: &AstFunction,
        var: &String,
        pc: usize,
        is_defined: bool,
        was_called_in_param: bool, //handles an edge case when `var` was a parameter
    ) -> Result<&Graph> {
        debug!(
            "=> Cache: func {} for {} at {} (is_defined {}, was_called_in param {})",
            function.name, var, pc, is_defined, was_called_in_param
        );
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
            if pc >= start_pc {
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
        let scfg = self
            .build_next2(function, instructions, &ctx.block_resolver)
            .context("Building the controlflow graph failed")?;

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
            is_defined,
            true,
            &mut graph,
            was_called_in_param,
        )
        .context("Building the graph failed")?;

        debug!("graph {:#?}", graph.flatten().collect::<Vec<_>>());

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
        was_called_as_param: bool,
    ) -> Result<Fact> {
        let get_relevant_instructions = |instructions, mut is_defined: bool, max_level: usize| {
            let mut relevant_instructions = Vec::new();

            for instruction in instructions {
                match instruction {
                    &SCFG::Instruction(_pc, ref inner_instruction) => {
                        debug!("Checking {:?}", inner_instruction);

                        let is_lhs = self.is_lhs_used(&var, &inner_instruction);
                        let is_rhs = self.is_rhs_used(&var, &inner_instruction);

                        debug!("is_lhs {}", is_lhs);
                        debug!("is_rhs {}", is_rhs);

                        // Edge case for return
                        // do not propagate if not in return
                        if !var.is_taut {
                            //except when var is taut, then ok
                            match inner_instruction {
                                Instruction::Call(..) if var.is_global && is_lhs => {
                                    // Edge case when variable is global, `is_lhs` is ok and on call
                                    // then add instruction, but stop there
                                    relevant_instructions.push(instruction.clone());
                                }
                                Instruction::CallIndirect(..) if var.is_global && is_lhs => {
                                    // Edge case when variable is global, `is_lhs` is ok and on call
                                    // then add instruction, but stop there
                                    relevant_instructions.push(instruction.clone());
                                }
                                _ => {}
                            }
                        }

                        if is_lhs {
                            if !is_defined {
                                is_defined = true;
                                relevant_instructions.push(instruction.clone());
                                debug!("Instruction is now defined.");
                            } else {
                                log::warn!("Instruction is overwritten. Therefore stopping.");
                                break;
                            }
                        } else {
                            if is_rhs {
                                debug!("Instruction is used on the rhs.");
                                relevant_instructions.push(instruction.clone());
                            }
                        }
                    }
                    &SCFG::Jump(_pc, _jump_to_pc) => {
                        relevant_instructions.push(instruction.clone());
                        break;
                    }
                    _ => {
                        relevant_instructions.push(instruction.clone());
                    }
                }
            }

            //if is_top_level && (!overwritten || var.is_taut) {
            if is_top_level {
                if !was_called_as_param || var.is_taut {
                    relevant_instructions.push(SCFG::FunctionEnd(max_level));
                } else if relevant_instructions.len() > 0 {
                    relevant_instructions.push(SCFG::FunctionEnd(max_level));
                }
            }

            (is_defined, relevant_instructions)
        };

        let (is_defined, relevant_instructions) =
            get_relevant_instructions(instructions.iter().skip(start_pc), is_defined, max_len);

        debug!("relevant scfg {} {:#?}", var.name, relevant_instructions);

        let next_pc = {
            if relevant_instructions.len() == 0 && was_called_as_param {
                start_pc
            } else {
                relevant_instructions
                    .first()
                    .map(|x| x.get_pc())
                    .unwrap_or(max_len)
            }
        };
        let first = Fact::from_var(var, start_pc, next_pc, track);
        debug!("first fact {:#?}", first);
        assert!(first.pc <= first.next_pc);
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
                SCFG::ConditionalSingle(pc, _instruction, _jump_to, _block) => {
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
                    //log::error!("Jump to pc is {}", jump_to_pc);
                    let next = jump_to_pc;
                    debug!("Edge from {} to {} for {}", pc, next, var.name);
                    let x = Fact::from_var(var, *pc, *next, track);
                    graph.add_normal(node.clone(), x.clone())?;
                    node = x;
                }
                SCFG::ConditionalJump(pc, _instruction, jump_to_pc) => {
                    // One edge back
                    //log::error!("Jump to pc is {}", jump_to_pc);
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
                SCFG::Table(pc, _instruction, jumps) => {
                    for jump_to_pc in jumps {
                        //log::error!("Jump to pc is {}", jump_to_pc);
                        let next = jump_to_pc;
                        debug!("Edge from {} to {} for {}", pc, next, var.name);
                        let x = Fact::from_var(var, *pc, *next, track);
                        graph.add_normal(node.clone(), x.clone())?;
                        node = x;
                    }
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
                        false,
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
                        false,
                    )?;

                    let x = Fact::from_var(var, *pc, next, track);
                    graph.add_normal(node.clone(), x.clone())?;
                    debug!("Edge from {} to {} for {}", pc, next, var.name);

                    Ok(x)
                }
                SCFG::ConditionalSingle(_pc, _instruction, _jump_pc, block) => {
                    // We have to look a step further
                    let next = relevant_instructions
                        .get(i + 2)
                        .map(|x| x.get_pc())
                        .unwrap_or(max_len);

                    let _res1 = self.build_graph(
                        function,
                        block,
                        block_resolver,
                        next,
                        var,
                        track,
                        0,
                        is_defined,
                        false,
                        graph,
                        false,
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
                    let first = jumps
                        .first()
                        .unwrap()
                        .parse::<usize>()
                        .context("Jump is not a number")?;

                    let first_pc = block_resolver
                        .get(&(function.name.clone(), format!("{}", first)))
                        .with_context(|| {
                            format!("Cannot find the beginning of the conditional {}", first)
                        })?;

                    let after_first_block = jumps
                        .get(1)
                        .unwrap()
                        .parse::<usize>()
                        .context("Jump is not a number")?;

                    let last_pc_first_block = block_resolver
                        .get(&(function.name.clone(), format!("{}", jumps.last().unwrap())))
                        .context("Cannot find the end of the first block")?
                        - 1;

                    let first_branch = self
                        .take_branch(function, *first_pc, last_pc_first_block - pc)
                        .collect::<Vec<_>>();
                    debug!("first {:#?}", first_branch);
                    assert_eq!(last_pc_first_block - pc, first_branch.len());

                    let first_branch = self.build_next2(function, first_branch, block_resolver)?;

                    main.push(SCFG::ConditionalSingle(
                        *pc,
                        inner_instruction.clone().clone(),
                        *first_pc + 1,
                        first_branch,
                    ));

                    i = last_pc_first_block + 1; //skip all conditionals
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
                    panic!(
                        "This is conditional is not supported. There must be an error with the IR"
                    );
                }
                Instruction::Table(jumps) => {
                    let mut jumps_pc = Vec::new();
                    for jump_to_block in jumps {
                        let jump_to_pc = block_resolver
                            .get(&(function.name.clone(), jump_to_block.clone()))
                            .context("Cannot find the block")?;

                        //jumps_pc.push(jump_to_pc.checked_sub(1).unwrap_or(*jump_to_pc));
                        jumps_pc.push(*jump_to_pc);
                    }

                    main.push(SCFG::Table(
                        *pc,
                        inner_instruction.clone().clone(),
                        jumps_pc,
                    ));

                    i += 1;
                    debug!("Setting i to {}", i);
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
            Instruction::Const(dest, ..) if dest == var => true,
            Instruction::Assign(dest, ..) if dest == var => true,
            Instruction::BinOp(dest, ..) if dest == var => true,
            Instruction::Phi(dest, ..) if dest == var => true,
            Instruction::Unop(dest, ..) if dest == var => true,
            Instruction::Kill(dest) if dest == var => true,
            Instruction::Unknown(dest) if dest == var => true,
            Instruction::Unop(dest, ..) if dest == var => true,
            Instruction::Call(..) if variable.is_global => true,
            Instruction::CallIndirect(..) if variable.is_global => true,
            Instruction::Call(_, _, dest) if dest.contains(var) => true,
            Instruction::CallIndirect(_, _, dest) if dest.contains(var) => true,
            Instruction::Load(dest, ..) if dest == var => true,
            _ => false,
        }
    }

    fn is_rhs_used(&self, variable: &Variable, instruction: &Instruction) -> bool {
        let var = &variable.name;

        match instruction {
            Instruction::Assign(_dest, src) if src == var => true,
            Instruction::BinOp(_, src1, src2) if src1 == var || src2 == var => true,
            Instruction::Phi(_, src1, src2) if src1 == var || src2 == var => true,
            Instruction::Unop(_dest, src) if src == var => true,
            Instruction::Call(..) if variable.is_taut => true,
            Instruction::Call(_, params, _) if params.contains(var) => true,
            Instruction::Call(..) if variable.is_memory => true,
            Instruction::Return(params) if params.contains(var) => true,
            Instruction::Return(..) if variable.is_global => true,
            Instruction::Return(..) if variable.is_memory => true,
            Instruction::Call(_, _, _) if variable.is_global => true,
            Instruction::CallIndirect(_, _, _) if variable.is_global => true,
            Instruction::CallIndirect(_, _, _) if variable.is_memory => true,
            Instruction::Store(src, _, _) if src == var => true,
            Instruction::Store(_src, ..) if variable.is_memory => true, //always true for all occurrences
            Instruction::Load(_dest, ..) if variable.is_memory => true, //always true for all occurrences
            Instruction::Load(dest, ..) if dest == var => true,
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
    fn test_building_mem_scfg() {
        let func_name = "main".to_string();
        let function = AstFunction {
            name: func_name.clone(),
            definitions: vec![
                "%0".to_string(),
                "%1".to_string(),
                "%2".to_string(),
                "%3".to_string(),
            ],
            instructions: vec![
                Instruction::Const("%0".to_string(), 1.0),
                Instruction::Block("0".to_string()),
                Instruction::Const("%2".to_string(), 1.0),
                Instruction::Store("%2".to_string(), 0.0, "%0".to_string()),
                Instruction::BinOp("3".to_string(), "%0".to_string(), "%2".to_string()),
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
        ctx.state.add_memory_var("main".to_string(), 0);

        let mut chain = DefUseChain::default();
        let facts = chain
            .cache(&mut ctx, &function, &"mem@0".to_string(), pc)
            .unwrap()
            .flatten()
            .collect::<Vec<_>>();

        assert_snapshot!("building_memory_defuse_mem_0_scfg", facts);
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
    fn test_building_table_scfg() {
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
                Instruction::Table(vec!["0".to_string(), "2".to_string(), "3".to_string()]),
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

        assert_snapshot!("building_table_defuse_reg_0_scfg", facts);
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

        assert_eq!(3, facts.len());
        assert_eq!(0, facts.get(0).unwrap().next_pc);

        let before = chain
            .points_to(&mut ctx, &function, &"%0".to_string(), 3)
            .unwrap();
        assert_eq!(1, before.len());
        assert_eq!(0, before.get(0).unwrap().next_pc);

        let after = chain
            .demand(&mut ctx, &function, &"%0".to_string(), 0)
            .unwrap();
        assert_eq!(1, after.len());
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

        assert_eq!(3, facts.len());
        assert_eq!(5, facts.get(1).unwrap().next_pc);

        let before = chain
            .points_to(&mut ctx, &function, &"%1".to_string(), 1)
            .unwrap();
        assert_eq!(1, before.len());
        assert_eq!(1, before.get(0).unwrap().next_pc);

        let after = chain
            .demand(&mut ctx, &function, &"%1".to_string(), 1)
            .unwrap();
        assert_eq!(1, after.len());
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
        assert_eq!(3, facts.len());

        let facts = chain
            .cache(&mut ctx, &function, &"%0".to_string(), 0)
            .unwrap()
            .flatten()
            .collect::<Vec<_>>();
        assert_eq!(3, facts.len());
    }
}
