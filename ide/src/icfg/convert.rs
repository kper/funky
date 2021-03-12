use std::ops::Sub;

use crate::counter::Counter;
use crate::icfg::graph::Graph;
use crate::ssa::ast::Instruction;
use crate::ssa::ast::Instruction::*;
use crate::symbol_table::SymbolTable;
use anyhow::{bail, Context, Result};
use funky::engine::Engine;
/// This module is responsible to parse
/// the webassembly AST to a graph
use funky::engine::{func::FuncInstance, FunctionBody, InstructionWrapper};
use log::debug;
use wasm_parser::core::*;

use crate::grammar::*;
use crate::icfg::graph::Fact;
use std::collections::HashMap;

#[derive(Debug)]
pub struct Convert {
    block_counter: Counter,
    functions: HashMap<String, usize>, // function name to fact id
    registration_returns: HashMap<String, Vec<Fact>>, // function name to facts
}

impl Convert {
    pub fn new() -> Self {
        Self {
            block_counter: Counter::default(),
            functions: HashMap::new(),
            registration_returns: HashMap::new(),
        }
    }

    pub fn visit(&mut self, ir: &'static str) -> Result<Graph> {
        debug!("Convert intermediate repr to graph");

        let prog = ProgramParser::new()
            .parse(ir)
            .context("Parsing IR failed")?;

        let mut graph = Graph::new();

        for function in prog.functions.iter() {
            debug!("Init function {}", function.name);
            graph.init_function(function);
            //graph.add_row(&function.name, "init".to_string(), &mut Vec::new())?;
        }

        //debug!("graph {:#?}", graph);

        for function in prog.functions.iter() {
            debug!("Creating graph from function {}", function.name);

            let mut iterator =
                InstructionIterator::new(function.instructions.iter().collect::<Vec<_>>());
            let mut killing_set = Vec::new();

            for instruction in &mut iterator {
                match instruction {
                    Instruction::Const(reg, _val) => {
                        debug!("Adding const");
                        graph.add_var(&function.name, &reg, &mut killing_set)?;

                        graph.add_row(
                            &function.name,
                            format!("after {:?}", instruction),
                            &mut killing_set,
                        )?;
                    }
                    Instruction::Assign(dest, src) => {
                        debug!("Assignment");
                        graph.add_assignment(&function.name, &dest, &src, &mut killing_set)?;

                        graph.add_row(
                            &function.name,
                            format!("after {:?}", instruction),
                            &mut killing_set,
                        )?;
                    }
                    Instruction::Unop(dest, src) => {
                        debug!("Unop");
                        graph.add_unop(&function.name, &dest, &src, &mut killing_set)?;
                        graph.add_row(
                            &function.name,
                            format!("after {:?}", instruction),
                            &mut killing_set,
                        )?;
                    }
                    Instruction::BinOp(dest, src1, src2) => {
                        debug!("Binop");
                        graph.add_binop(&function.name, &dest, &src1, &src2, &mut killing_set)?;
                        graph.add_row(
                            &function.name,
                            format!("after {:?}", instruction),
                            &mut killing_set,
                        )?;
                    }
                    Instruction::Kill(dest) => {
                        debug!("Kill");
                        graph.kill_var(&function.name, &dest, &mut killing_set)?;
                        graph.add_row(
                            &function.name,
                            format!("after {:?}", instruction),
                            &mut killing_set,
                        )?;
                    }
                    Instruction::Call(name, _regs) => {
                        graph.add_row(
                            &function.name,
                            format!("before {:?}", instruction),
                            &mut killing_set,
                        )?;
                        let meeting_facts = graph.add_call_to_return(
                            &function.name,
                            format!("Return from {}", name),
                            &mut killing_set,
                        )?;

                        // Expect a return edge from `name` with function.name to the meeting facts
                        self.registration_returns.insert(name.clone(), meeting_facts);
                    }
                    _ => {}
                }

                killing_set.clear();
            }
        }

        for function in prog.functions.iter() {
            let mut iterator =
                InstructionIterator::new(function.instructions.iter().collect::<Vec<_>>());

            for instruction in &mut iterator {
                match instruction {
                    Instruction::Call(name, regs) => {
                        debug!("Call {}", name);

                        let lookup_function = prog
                            .functions
                            .iter()
                            .find(|x| &x.name == name)
                            .expect("Function not found");

                        graph.add_call(&function.name, &lookup_function, name, regs)?;
                    }
                    _ => {}
                }
            }
        }

        for (goal_function, meeting_facts) in self.registration_returns.drain() {
            if let Some(goal_function_ref) = graph.functions.get(&goal_function) {
                graph.add_return(&goal_function_ref.last_facts.clone(), meeting_facts)?;
            }
        }

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
