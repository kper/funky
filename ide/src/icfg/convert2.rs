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
        let mut graph = Graph::new();

        self.tabulate(&mut graph, prog, req)?;

        Ok((graph))
    }

    fn tabulate(&mut self, graph: &mut Graph, prog: &Program, req: &Request) -> Result<Vec<Fact>> {
        debug!("Convert intermediate repr to graph");
        //let mut path_edges = Vec::new();
        //let mut worklist = Vec::new();

        let function = prog
            .functions
            .iter()
            .find(|x| x.name == req.function)
            .context("Cannot find function")?;

        if graph.is_function_defined(&function.name) {
            debug!("==> Function was already summarised.");
            return Ok(vec![]);
        }

        graph.init_function_def(&function)?;
        graph.add_var(Variable {
            function: req.function.clone(),
            is_taut: true,
            is_global: false,
            name: "taut".to_string(),
        });

        for def in function.params.iter() {
            graph.add_var(Variable {
                function: req.function.clone(),
                is_taut: false,
                is_global: false,
                name: def.clone(),
            });
        }

        let init = graph.new_facts(&req.function, String::new())?;
        let init_fact = init.get(0).unwrap().clone();

        graph.facts.extend_from_slice(&init);

        for fact in init.into_iter() {
            graph.add_path_edge(init_fact.clone(), fact)?;
        }

        let _ = graph.pc_counter.set(req.pc + 1);
        let mut offset_pc = 0;

        // init

        /*
        for instruction in function.instructions.iter().skip(req.pc - 1) {
            let mut do_break = false;
            match instruction {
                Instruction::Const(reg, _) => {
                    graph.add_var(Variable {
                        function: function.name.clone(),
                        is_global: false,
                        is_taut: false,
                        name: reg.clone(),
                    });
                    do_break = true;
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
                    do_break = true;
                }
                _ => {
                    offset_pc += 1;
                    //graph.pc_counter.get();
                }
            }

            let out_ = graph
                .new_facts(&function.name, format!("{:?}", instruction))
                .context("Cannot create a new fact")?;

            graph.facts.extend_from_slice(&out_); //required for tikz
            for fact in out_.into_iter() {
                graph.add_path_edge(init_fact.clone(), fact)?;
            }

            if do_break {
                break;
            }
        }*/

        let mut skipping = false;
        for instruction in function.instructions.iter().skip(req.pc - 1 + offset_pc) {
            debug!("Instrution {:?}", instruction);

            if skipping {
                debug!("Last instruction declared to skip. Therefore skipping");
                skipping = false;
                break;
            }

            match instruction {
                Instruction::Block(_num) | Instruction::Jump(_num) => {}
                Instruction::Const(reg, _) => {
                    if graph.get_var(&function.name, reg).is_some() {
                        graph.remove_var(&function.name, reg)?;
                    } else {
                        if reg == &req.variable {
                            // init
                            graph.add_var(Variable {
                                function: function.name.clone(),
                                is_global: false,
                                is_taut: false,
                                name: reg.clone(),
                            });
                        }
                    }
                }
                Instruction::Assign(dest, src) | Instruction::Unop(dest, src) => {
                    debug!("Assign");

                    if graph.get_var(&function.name, dest).is_some() {
                        debug!("Assigned dest is relevant");
                        // Relevant

                        graph.remove_var(&function.name, &dest)?;
                    } else {
                        if dest == &req.variable {
                            // init
                            graph.add_var(Variable {
                                function: function.name.clone(),
                                is_global: false,
                                is_taut: false,
                                name: dest.clone(),
                            });
                        }
                    }

                    if graph.get_var(&function.name, src).is_some() {
                        graph.add_var(Variable {
                            function: function.name.clone(),
                            is_global: false,
                            is_taut: false,
                            name: dest.clone(),
                        });
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
                }
                Instruction::Kill(dest) => {
                    if graph.get_var(&function.name, dest).is_some() {
                        graph.remove_var(&function.name, &dest)?;
                    }
                }
                Instruction::Unknown(dest) => {}
                Instruction::Store => {}
                Instruction::Return(regs) => {
                    skipping = true;

                    let vars: Vec<Variable> = graph
                        .get_vars(&function.name)
                        .context("Cannot get vars")?
                        .iter()
                        .filter(|x| !x.is_taut)
                        .cloned()
                        .collect();

                    for var in vars {
                        if !regs.contains(&var.name) {
                            graph.remove_var(&function.name, &var.name)?;
                        }
                    }
                }
                Instruction::Conditional(_reg, jumps) => {
                    assert!(jumps.len() >= 1, "Conditional must have at least one jump");
                }
                Instruction::Table(jumps) => {
                    assert!(jumps.len() >= 1, "Conditional must have at least one jump");
                }
                Instruction::Call(callee, _params, dests) => {
                    let req = Request {
                        function: callee.clone(),
                        pc: 1,
                        variable: "temp".to_string(), //TODO remove variable, because doesnt matter
                    };

                    let old_pc = graph.pc_counter.peek();
                    graph.pc_counter.set(1);

                    let _summary: Vec<Fact> = self
                        .tabulate(graph, prog, &req)
                        .context("Fail occured in nested call")?;

                    graph.pc_counter.set(old_pc);

                    for dest in dests.iter() {
                        // Overwrite
                        if graph.get_var(&function.name, dest).is_some() {
                            graph.remove_var(&function.name, dest)?;
                        }
                    }

                    for (i, _summ) in _summary.into_iter().enumerate().skip(1) {
                        let name = dests.get(i - 1).unwrap().clone();
                        graph.add_var(Variable {
                            function: function.name.clone(),
                            is_global: false,
                            is_taut: false,
                            name,
                        });
                    }
                }
                Instruction::CallIndirect(callees, _params, dests) => {
                    for callee in callees.iter() {
                        let req = Request {
                            function: callee.clone(),
                            pc: 1,
                            variable: "temp".to_string(), //TODO remove variable, because doesnt matter
                        };

                        let old_pc = graph.pc_counter.peek();
                        graph.pc_counter.set(1);

                        let _summary: Vec<Fact> = self
                            .tabulate(graph, prog, &req)
                            .context("Fail occured in nested call")?;

                        graph.pc_counter.set(old_pc);

                        for dest in dests.iter() {
                            // Overwrite
                            if graph.get_var(&function.name, dest).is_some() {
                                graph.remove_var(&function.name, dest)?;
                            }
                        }

                        for (i, _summ) in _summary.into_iter().enumerate().skip(1) {
                            let name = dests.get(i - 1).unwrap().clone();
                            graph.add_var(Variable {
                                function: function.name.clone(),
                                is_global: false,
                                is_taut: false,
                                name,
                            });
                        }
                    }
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

        let out_ = graph.get_last_facts(&function.name);
        debug!("Summary {:?}", out_);
        for fact in out_.iter() {
            graph.add_summary_edge(init_fact.clone(), fact.clone())?;
        }

        Ok(out_)
    }
}
