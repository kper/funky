/// This module is responsible to parse
/// the webassembly AST to a graph
use crate::counter::Counter;
use crate::icfg::graph2::*;
use crate::ssa::ast::Instruction;
use anyhow::{bail, Context, Result};

use log::debug;

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

    fn add_ctrl_flow(
        &self,
        graph: &mut Graph,
        in_: &Vec<&Fact>,
        out_: &Vec<&Fact>,
        except: &String,
    ) -> Result<()> {
        for (from, after) in in_
            .iter()
            .zip(out_)
            .filter(|(from, to)| &from.belongs_to_var != except && &to.belongs_to_var != except)
            .map(|(from, after)| (from.clone(), after.clone()))
        {
            graph.add_normal(from.clone(), after.clone())?;
        }

        Ok(())
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

                let facts = graph.facts.clone();
                let in_ = facts
                    .iter()
                    .filter(|x| x.pc == pc - 1 && x.function == function.name)
                    .collect::<Vec<_>>();

                let out_ = facts
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

                        self.add_ctrl_flow(&mut graph, &in_, &out_, reg)?;
                    }
                    Instruction::Assign(dest, src) | Instruction::Unop(dest, src) => {
                        let before = in_
                            .iter()
                            .find(|x| &x.belongs_to_var == src)
                            .map(|x| *x)
                            .context("Cannot get `before` fact")?
                            .clone();
                        let after = out_
                            .iter()
                            .find(|x| &x.belongs_to_var == dest)
                            .map(|x| *x)
                            .context("Cannot get `before` fact")?
                            .clone();

                        graph.add_normal(before, after)?;

                        self.add_ctrl_flow(&mut graph, &in_, &out_, dest)?;
                    }
                    Instruction::BinOp(dest, src1, src2) => {
                        let before = in_
                            .iter()
                            .filter(|x| &x.belongs_to_var == src1 || &x.belongs_to_var == src2)
                            .map(|x| *x)
                            .collect::<Vec<_>>();

                        let after = out_
                            .iter()
                            .find(|x| &x.belongs_to_var == dest)
                            .map(|x| *x)
                            .context("Cannot get `before` fact")?
                            .clone();

                        for from in before.into_iter() {
                            graph.add_normal(from.clone(), after.clone())?;
                        }

                        self.add_ctrl_flow(&mut graph, &in_, &out_, dest)?;
                    }
                    Instruction::Kill(dest) => {
                        self.add_ctrl_flow(&mut graph, &in_, &out_, dest)?;
                    }
                    _ => {}
                }
            }

            graph.pc_counter.set(1); // Set to the first instruction
        }

        self.handle_calls(prog, &mut graph)?;

        return Ok(graph);
    }

    pub fn handle_calls(&mut self, prog: Program, graph: &mut Graph) -> Result<()> {
        for function in prog.functions.iter() {
            graph.pc_counter.set(1); // Set to the first instruction

            let mut iterator =
                InstructionIterator::new(function.instructions.iter().collect::<Vec<_>>());

            debug!("Setting flow functions");
            for instruction in &mut iterator {
                let pc = graph.pc_counter.get();

                let facts = graph.facts.clone();
                let in_ = facts
                    .iter()
                    .filter(|x| x.pc == pc - 1 && x.function == function.name)
                    .collect::<Vec<_>>();

                let out_ = facts
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
                    Instruction::Call(callee_name, params, dest) => {
                        let before = in_
                            .iter()
                            .filter(|x| {
                                params.contains(&x.belongs_to_var)
                                    || x.belongs_to_var == "taut".to_string()
                            })
                            .map(|x| *x)
                            .collect::<Vec<_>>();

                        let callee_params_vars: Vec<_> = graph
                            .get_vars(callee_name)
                            .context("Cannot get variables of called function")?
                            .iter()
                            .take(params.len() + 1)
                            .collect();

                        let callee_all_vars: Vec<_> = graph
                            .get_vars(callee_name)
                            .context("Cannot get variables of called function")?
                            .iter()
                            .collect();

                        let mut callee_first_facts = Vec::new();
                        let mut callee_last_facts = Vec::new();

                        for var in callee_params_vars {
                            let first_fact = graph
                                .get_first_fact_of_var(var)
                                .context("Cannot get first fact of variable")?;
                            callee_first_facts.push(first_fact.clone());
                        }

                        for var in callee_all_vars {
                            let last_fact = graph
                                .get_last_fact_of_var(var)
                                .context("Cannot get last fact of variable")?;

                            callee_last_facts.push(last_fact.clone());
                        }

                        for (from, to) in before.iter().zip(callee_first_facts) {
                            graph.add_call_edge(from.clone().clone(), to)?;
                        }

                        // After the return
                        let after_caller = out_
                            .iter()
                            .filter(|x| {
                                dest.contains(&x.belongs_to_var)
                                    || x.belongs_to_var == "taut".to_string()
                            })
                            .map(|x| *x)
                            .collect::<Vec<_>>();


                        // Shrink to result of callee

                        let expected_return = graph
                            .functions
                            .get(callee_name)
                            .context("Cannot find function")?
                            .return_count;

                        let taut = callee_last_facts.get(0).unwrap().clone();

                        callee_last_facts.reverse();

                        let mut callee_last_facts: Vec<_> = callee_last_facts
                            .into_iter()
                            .take(expected_return)
                            .collect();

                        callee_last_facts.reverse();

                        callee_last_facts.insert(0, taut);

                        assert_eq!(callee_last_facts.len(), after_caller.len());
                        for (from, to) in callee_last_facts.iter().zip(after_caller) {
                            graph.add_return_edge(from.clone(), to.clone())?;
                        }
                    }
                    _ => {}
                }
            }
        }

        Ok(())
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
