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
}

impl Convert {
    pub fn new() -> Self {
        Self {
            block_counter: Counter::default(),
            functions: HashMap::new(),
        }
    }

    fn add_ctrl_flow(
        &self,
        graph: &mut Graph,
        in_: &Vec<&Fact>,
        out_: &Vec<&Fact>,
        except: Option<&String>,
    ) -> Result<()> {
        if let Some(except) = except {
            for (from, after) in in_
                .iter()
                .zip(out_)
                .filter(|(from, to)| &from.belongs_to_var != except && &to.belongs_to_var != except)
                .map(|(from, after)| (from.clone(), after.clone()))
            {
                graph.add_normal(from.clone(), after.clone())?;
            }
        } else {
            for (from, after) in in_
                .iter()
                .zip(out_)
                .map(|(from, after)| (from.clone(), after.clone()))
            {
                graph.add_normal(from.clone(), after.clone())?;
            }
        }

        Ok(())
    }

    fn add_call_to_return(
        &self,
        graph: &mut Graph,
        in_: &Vec<&Fact>,
        out_: &Vec<&Fact>,
        except: &Vec<String>,
    ) -> Result<()> {
        for (from, after) in in_
            .iter()
            .zip(out_)
            .filter(|(from, to)| {
                !except.contains(&from.belongs_to_var) && !except.contains(&to.belongs_to_var)
            })
            .map(|(from, after)| (from.clone(), after.clone()))
        {
            graph.add_call_to_return_edge(from.clone(), after.clone())?;
        }

        Ok(())
    }

    fn generate_all_facts(
        graph: &mut Graph,
        function: &crate::ssa::ast::Function,
        block_saver: &mut HashMap<String, usize>,
    ) -> Result<()> {
        let mut iterator = function.instructions.iter();

        let mut pc = 0;

        debug!("///////");
        debug!("/////// ALL");
        debug!("///////");

        //Generating all facts
        for instruction in &mut iterator {
            match instruction {
                Instruction::Conditional(_, nested) => {
                    graph.add_statement(function, instruction)?;
                    //Self::generate_all_facts(graph, function, nested.iter().collect())?;
                }
                Instruction::Block(num) => {
                    block_saver.insert(num.clone(), pc);
                    graph.add_statement(function, instruction)?;
                }
                _ => graph.add_statement(function, instruction)?,
            }

            pc += 1;
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

            let mut block_saver = HashMap::new(); // key = Block, Value = pc
            Self::generate_all_facts(&mut graph, function, &mut block_saver)?;

            graph.pc_counter.set(1); // Set to the first instruction

            let mut iterator = function.instructions.iter();
            //InstructionIterator::new(function.instructions.iter().collect::<Vec<_>>());

            debug!("///////");
            debug!("/////// FLOW");
            debug!("///////");
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
                    Instruction::Block(num) => {
                        self.add_ctrl_flow(&mut graph, &in_, &out_, None)?;
                    }
                    Instruction::Jump(num) => {
                        if let Some(jump_pc) = block_saver.get(num) {
                            let facts = graph.facts.clone();
                            let to_facts = facts
                                .iter()
                                .filter(|x| x.function == function.name && x.pc == *jump_pc)
                                .collect::<Vec<_>>();

                            for (from, to) in in_.iter().zip(to_facts) {
                                graph.add_normal_curved(from.clone().clone(), to.clone())?;
                            }
                        } else {
                            bail!("Block {} was not found", num);
                        }
                    }
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

                        self.add_ctrl_flow(&mut graph, &in_, &out_, Some(reg))?;
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

                        self.add_ctrl_flow(&mut graph, &in_, &out_, Some(dest))?;
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

                        self.add_ctrl_flow(&mut graph, &in_, &out_, Some(dest))?;
                    }
                    Instruction::Kill(dest) => {
                        self.add_ctrl_flow(&mut graph, &in_, &out_, Some(dest))?;
                    }
                    Instruction::Call(_callee_name, _params, regs) => {
                        self.add_call_to_return(&mut graph, &in_, &out_, regs)?;
                    }
                    Instruction::Conditional(reg, instruction_count) => {
                        let facts = graph.facts.clone();
                        // Set else path
                        let facts_before_block = facts
                            .iter()
                            .filter(|x| x.function == function.name && x.pc == pc);

                        let facts_after_block = facts.iter().filter(|x| {
                            x.function == function.name && x.pc == pc + instruction_count
                        });

                        for (from, to) in facts_before_block.zip(facts_after_block) {
                            graph.add_normal_curved(from.clone(), to.clone())?;
                        }

                        // Set normal entering
                        self.add_ctrl_flow(&mut graph, &in_, &out_, None)?;
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

            let mut iterator = function.instructions.iter();

            debug!("///////");
            debug!("/////// HANDLING CALLS");
            debug!("///////");
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
                            .get_vars(&callee_name)
                            .context("Cannot get variables of called function")?
                            .iter()
                            .take(params.len() + 1)
                            .collect();

                        let callee_all_vars: Vec<_> = graph
                            .get_vars(&callee_name)
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