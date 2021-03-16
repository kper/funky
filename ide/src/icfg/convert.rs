/// This module is responsible to parse
/// the webassembly AST to a graph
use crate::counter::Counter;
use crate::icfg::graph2::Graph;
use crate::ssa::ast::Instruction;
use anyhow::{bail, Context, Result};

use log::debug;

use crate::icfg::graph::Fact;
use std::collections::HashMap;

use crate::ssa::ast::Program;

#[derive(Debug)]
pub struct Convert {
    block_counter: Counter,
    functions: HashMap<String, usize>, // function name to fact id
    registration_returns: HashMap<String, (Vec<Fact>, Vec<String>)>, // function name to (all facts + dest regs)
}

impl Convert {
    pub fn new() -> Self {
        Self {
            block_counter: Counter::default(),
            functions: HashMap::new(),
            registration_returns: HashMap::new(),
        }
    }

    pub fn visit(&mut self, prog: Program) -> Result<Graph> {
        debug!("Convert intermediate repr to graph");

        let mut graph = Graph::new();

        for function in prog.functions.iter() {
            debug!("Init function {}", function.name);
            graph.init_function(function)?;
        }

        graph.pc_counter.set(1); // Set to the first instruction

        for function in prog.functions.iter() {
            debug!("Creating graph from function {}", function.name);

            let mut iterator =
                InstructionIterator::new(function.instructions.iter().collect::<Vec<_>>());

            //Generating all facts
            for instruction in &mut iterator {
                debug!("Instruction {:?}", instruction);
                graph.add_statement(function, instruction)?;
            }

            graph.pc_counter.set(1); // Set to the first instruction

            let mut iterator =
                InstructionIterator::new(function.instructions.iter().collect::<Vec<_>>());

            debug!("Setting flow functions");
            for instruction in &mut iterator {
                let pc = graph.pc_counter.get();

                let in_ = graph
                    .facts
                    .iter()
                    .filter(|x| x.pc == pc - 1 && x.function == function.name)
                    .collect::<Vec<_>>();
                let out_ = graph
                    .facts
                    .iter()
                    .filter(|x| x.pc == pc && x.function == function.name)
                    .collect::<Vec<_>>();

                if in_.len() == 0 {
                    bail!("Cannot find `in` set");
                }

                if out_.len() == 0 {
                    bail!("Cannot find `out` set");
                }

                debug!("Instruction {:?}", instruction);
                match instruction {
                    Instruction::Const(reg, _val) => {
                        let before = in_
                            .iter()
                            .find(|x| x.belongs_to_var == "taut".to_string())
                            .map(|x| *x)
                            .context("Cannot get `before` fact")?
                            .clone();
                        let after = out_
                            .iter()
                            .find(|x| &x.belongs_to_var == reg)
                            .map(|x| *x)
                            .context("Cannot get `before` fact")?
                            .clone();

                        graph.add_normal(before, after)?;
                    }
                    _ => {}
                }
            }

            graph.pc_counter.set(1); // Set to the first instruction

            /*
            for instruction in &mut iterator {
                match instruction {
                    Instruction::Const(reg, _val) => {
                        debug!("Adding const");

                        graph.add_row(
                            &function.name,
                            format!("before {:?}", instruction),
                        )?;

                        graph.add_var(&function.name, &reg)?;

                        graph.add_row(
                            &function.name,
                            format!("after {:?}", instruction),
                        )?;
                    }
                    Instruction::Assign(dest, src) => {
                        debug!("Assignment");

                        graph.add_row(
                            &function.name,
                            format!("{:?}", instruction),
                        )?;

                        graph.add_assignment(&function.name, &dest, &src)?;

                        graph.add_row(
                            &function.name,
                            format!("{:?}", instruction),
                        )?;
                    }
                    Instruction::Unop(dest, src) => {
                        debug!("Unop");
                        graph.add_row(
                            &function.name,
                            format!("{:?}", instruction),
                        )?;
                        graph.add_unop(&function.name, &dest, &src)?;
                        graph.add_row(
                            &function.name,
                            format!("{:?}", instruction),
                        )?;
                    }
                    Instruction::BinOp(dest, src1, src2) => {
                        debug!("Binop");
                        graph.add_row(
                            &function.name,
                            format!("{:?}", instruction),
                        )?;

                        graph.add_binop(&function.name, &dest, &src1, &src2)?;
                        graph.add_row(
                            &function.name,
                            format!("{:?}", instruction),
                        )?;
                    }
                    Instruction::Kill(dest) => {
                        debug!("Kill");
                        graph.add_row(
                            &function.name,
                            format!("{:?}", instruction),
                        )?;
                        graph.kill_var(&function.name, &dest)?;
                        graph.add_row(
                            &function.name,
                            format!("{:?}", instruction),
                        )?;
                    }
                    Instruction::Call(name, _params, dest_regs) => {
                        graph.add_empty_vars(&function.name, &dest_regs)?;
                        graph.add_row(
                            &function.name,
                            format!("{:?}", instruction),
                        )?;
                        let meeting_facts = graph.add_call_to_return(
                            &function.name,
                            format!("Return from {}", name),
                        )?;

                        // Expect a return edge from `name` with function.name to the meeting facts
                        self.registration_returns
                            .insert(name.clone(), (meeting_facts, dest_regs.clone()));
                    }
                    _ => {}
                }

                graph.pc_counter.get();
            }
                */
        }

        /*
        for function in prog.functions.iter() {
            let mut iterator =
                InstructionIterator::new(function.instructions.iter().collect::<Vec<_>>());

            for instruction in &mut iterator {
                match instruction {
                    Instruction::Call(name, params, _dest_regs) => {
                        debug!("Call {}", name);

                        let lookup_function = prog
                            .functions
                            .iter()
                            .find(|x| &x.name == name)
                            .context("Function not found")?;

                        graph.add_call(&function.name, &lookup_function, name, params)?;
                    }
                    _ => {}
                }
            }
        }

        for (goal_function, (meeting_facts, dest_regs)) in self.registration_returns.drain() {
            if let Some((goal_function_name, goal_function_results_len, goal_function_last_facts)) =
                graph.functions.get(&goal_function).map(|x| (&x.name, x.results_len, x.last_facts.clone())) {

                if goal_function_results_len != dest_regs.len() {
                    bail!("Mismatch results with call of {}.\nExpected results: {}\nActual results: {}",
                        goal_function_name,
                        goal_function_results_len,
                        dest_regs.len());
                }

                graph.add_return(
                    &goal_function_last_facts,
                    meeting_facts,
                    &dest_regs,
                )?;
            }
        }
        */

        return Ok(graph);
    }
}

/// Highlevel iterator, that has the
/// ability to jump to blocks
struct InstructionIterator<'a> {
    instructions: Vec<&'a Instruction>,
    current: usize,
    blocks: HashMap<&'a String, usize>, // BlockId to index in instructions
}

impl<'a> InstructionIterator<'a> {
    pub fn new(instructions: Vec<&'a Instruction>) -> Self {
        let mut v = Self {
            instructions,
            current: 0,
            blocks: HashMap::default(),
        };

        v.create_block_jump_list();

        v
    }

    fn create_block_jump_list(&mut self) {
        for (index, block) in self
            .instructions
            .iter()
            .enumerate()
            .filter(|(_id, x)| matches!(x, Instruction::Block(_)))
        {
            match block {
                Instruction::Block(id) => {
                    self.blocks.insert(id, index);
                }
                _ => {
                    panic!("Only expecting blocks");
                }
            }
        }
    }

    pub fn jump_to(&mut self, block_id: &String) -> Result<()> {
        debug!("Jump to block {}", block_id);
        if let Some(index) = self.blocks.get(block_id) {
            self.current = *index;
            return Ok(());
        } else {
            bail!("Block {} does not exist", block_id);
        }
    }

    #[allow(dead_code)]
    pub fn peek(&self) -> Option<&'a Instruction> {
        self.instructions.get(self.current).map(|x| *x)
    }
}

impl<'a> std::iter::Iterator for &mut InstructionIterator<'a> {
    type Item = &'a Instruction;

    fn next(&mut self) -> Option<Self::Item> {
        let item = self.instructions.get(self.current);
        self.current += 1;
        let item = item.map(|x| *x);

        match item {
            Some(Instruction::Block(_)) => {
                return self.next();
            }
            _ => {}
        }

        if let Some(&Instruction::Jump(ref id)) = item {
            if self.jump_to(id).is_ok() {
                self.next()
            } else {
                debug!("Block not found, therefore ending");
                None
            }
        } else {
            item
        }
    }
}
