use crate::icfg::graph2::*;
use crate::ir::ast::Instruction;
/// This module is responsible to parse
/// the webassembly AST to a graph
use crate::{counter::Counter, solver::Request};
use anyhow::{bail, Context, Result};

use log::debug;

use std::collections::HashMap;

use crate::ir::ast::Program;

use crate::icfg::convert::CallHandler;

use crate::icfg::graph2::Edge;

#[derive(Debug)]
pub struct ConvertSummary {
    block_counter: Counter,
    call_handler: CallHandler,
}

impl ConvertSummary {
    pub fn new() -> Self {
        Self {
            block_counter: Counter::default(),
            call_handler: CallHandler::default(),
        }
    }

    pub fn visit(&mut self, prog: &Program, req: &Request) -> Result<Graph> {
        debug!("Convert intermediate repr to graph");

        let mut graph = Graph::new();

        //let mut path_edges = Vec::new();
        //let mut worklist = Vec::new();

        let function = prog
            .functions
            .iter()
            .find(|x| x.name == req.function)
            .context("Cannot find function")?;
        graph.init_function_def(&function)?;
        graph.add_var(Variable {
            function: req.function.clone(),
            is_taut: true,
            is_global: false,
            name: "taut".to_string(),
        });

        //TODO start from req
        let init_fact = graph
            .init_function_fact(req.function.clone(), req.pc)
            .clone();
        graph.add_path_edge(init_fact.clone(), init_fact.clone())?;

        let _ = graph.pc_counter.set(req.pc + 1);

        // init

        for instruction in function.instructions.iter().skip(req.pc - 1).take(1) {
            match instruction {
                Instruction::Block(_num) => {}
                Instruction::Const(reg, _) => {
                    graph.add_var(Variable {
                        function: function.name.clone(),
                        is_global: false,
                        is_taut: false,
                        name: reg.clone(),
                    });
                }
                Instruction::Assign(dest, src) => {
                    graph.add_var(Variable {
                        function: function.name.clone(),
                        is_global: false,
                        is_taut: false,
                        name: dest.clone(),
                    });

                    graph.add_var(Variable {
                        function: function.name.clone(),
                        is_global: false,
                        is_taut: false,
                        name: src.clone(),
                    });
                }
                _ => {}
            }

            let out_ = graph
                .new_facts(&function.name, format!("{:?}", instruction))
                .context("Cannot create a new fact")?;

            graph.facts.extend_from_slice(&out_); //required for tikz
            for fact in out_.into_iter() {
                graph.add_path_edge(init_fact.clone(), fact)?;
            }
        }

        for instruction in function.instructions.iter().skip(req.pc) {
            /*
            let in_ = graph
                .facts
                .iter()
                .filter(|x| x.pc == pc - 1 && x.function == function.name)
                .collect::<Vec<_>>();
            */

            /*
            let out_ = graph
                .facts
                .iter()
                .filter(|x| x.pc == pc && x.function == function.name)
                .collect::<Vec<_>>();*/

            //debug_assert!(in_.len() > 0);

            match instruction {
                Instruction::Block(_num) | Instruction::Jump(_num) => {
                    let out_ = graph
                        .new_facts(&function.name, format!("{:?}", instruction))
                        .context("Cannot create a new fact")?;

                    graph.facts.extend_from_slice(&out_); //required for tikz

                    for fact in out_.into_iter() {
                        graph.add_path_edge(init_fact.clone(), fact)?;
                    }
                }
                Instruction::Const(_reg, _) => {
                    let out_ = graph
                        .new_facts(&function.name, format!("{:?}", instruction))
                        .context("Cannot create a new fact")?;

                    graph.facts.extend_from_slice(&out_); //required for tikz

                    // Ignoring, because not reachable from init
                    for fact in out_.into_iter() {
                        graph.add_path_edge(init_fact.clone(), fact)?;
                    }
                }
                Instruction::Assign(dest, src) | Instruction::Unop(dest, src) => {
                    if graph.get_var(&function.name, dest).is_some() {
                        // Relevant

                        if graph.get_var(&function.name, src).is_none() {
                            // remove the old var if it exists
                            let old_var = graph
                                .get_vars(&function.name)
                                .context("Canot find variables")?
                                .iter()
                                .find(|x| !x.is_taut && &x.name != dest)
                                .context("Cannot find variable")?
                                .name
                                .clone();

                            graph.remove_var(&function.name, &old_var)?;

                            //graph.remove_var(&function.name, src)?;
                            graph.add_var(Variable {
                                function: function.name.clone(),
                                is_global: false,
                                is_taut: false,
                                name: src.clone(),
                            });
                        }
                    }

                    let out_ = graph
                        .new_facts(&function.name, format!("{:?}", instruction))
                        .context("Cannot create a new fact")?;

                    graph.facts.extend_from_slice(&out_); //required for tikz

                    for fact in out_.into_iter() {
                        graph.add_path_edge(init_fact.clone(), fact)?;
                    }
                }
                Instruction::BinOp(dest, src1, src2) => {
                    debug!("Bin {} = {} op {}", dest, src1, src2);

                    if graph.get_var(&function.name, dest).is_some() {
                        let mut ok = false;
                        if graph.get_var(&function.name, src1).is_none() {
                            graph.add_var(Variable {
                                function: function.name.clone(),
                                is_global: false,
                                is_taut: false,
                                name: src1.clone(),
                            });
                            ok = true;
                        }

                        if graph.get_var(&function.name, src2).is_none() {
                            graph.add_var(Variable {
                                function: function.name.clone(),
                                is_global: false,
                                is_taut: false,
                                name: src2.clone(),
                            });
                            ok = true;
                        }

                        if ok {
                            let old_var = graph
                                .get_vars(&function.name)
                                .context("Canot find variables")?
                                .iter()
                                .find(|x| !x.is_taut && &x.name != dest)
                                .context("Cannot find variable")?
                                .name
                                .clone();

                            graph.remove_var(&function.name, &old_var)?;
                        }
                    }

                    let out_ = graph
                        .new_facts(&function.name, format!("{:?}", instruction))
                        .context("Cannot create a new fact")?;

                    graph.facts.extend_from_slice(&out_); //required for tikz

                    for fact in out_.into_iter() {
                        graph.add_path_edge(init_fact.clone(), fact)?;
                    }
                }
                Instruction::Kill(dest) => {
                    if graph.get_var(&function.name, dest).is_some() {
                        graph.remove_var(&function.name, &dest)?;
                    }

                    let out_ = graph
                        .new_facts(&function.name, format!("{:?}", instruction))
                        .context("Cannot create a new fact")?;

                    graph.facts.extend_from_slice(&out_); //required for tikz

                    for fact in out_.into_iter() {
                        graph.add_path_edge(init_fact.clone(), fact)?;
                    }
                }
                _ => {}
            }
        }

        Ok(graph)
    }
}
