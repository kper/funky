use crate::icfg::flowfuncs::*;
use anyhow::bail;

pub struct TaintInitialFlowFunction;

impl InitialFlowFunction for TaintInitialFlowFunction {
    fn flow(
        &self,
        function: &AstFunction,
        pc: usize,
        init_facts: &Vec<Fact>,
        normal_flows_debug: &mut Vec<Edge>,
        state: &mut State,
    ) -> Result<Vec<Edge>> {
        debug!("Calling init flow for {} with pc {}", function.name, pc);

        let mut edges = Vec::new();

        // We need the offset, because not every
        // instruction is taintable. For example BLOCK and JUMP
        // have no registers. That's why skip to the next one.
        let mut offset = 0;
        let instructions = &function.instructions;

        let mut init_fact = init_facts.get(0).context("Cannot find taut")?.clone();

        loop {
            let pc = pc + offset;
            let instruction = instructions.get(pc).context("Cannot find instr")?;
            debug!("Next instruction is {:?}", instruction);

            state.add_statement(
                function,
                format!("{:?}", instruction),
                pc,
                &"taut".to_string(),
            )?;

            let _after = state
                .get_facts_at(&function.name, pc)?
                .filter(|x| x.var_is_taut)
                .collect::<Vec<_>>();

            let after = _after.get(0).unwrap();

            edges.push(Edge::Path {
                from: init_fact.clone().clone(),
                to: after.clone().clone(),
            });

            match instruction {
                Instruction::Const(dest, _) |
                Instruction::Assign(dest, _)
                | Instruction::Unop(dest, _)
                | Instruction::BinOp(dest, _, _)
                | Instruction::Conditional(dest, _)
                | Instruction::Phi(dest, _, _) |
                 Instruction::Load(dest, _, _) 
                => {
                    let before2 = vec![init_fact.clone()];

                    state.add_statement(function, format!("{:?}", instruction), pc + 1, dest)?;

                    for b in before2.into_iter() {
                        let after2 = state
                            .get_facts_at(&function.name, pc + 1)?
                            .filter(|x| &x.belongs_to_var == dest)
                            .cloned();

                        for a in after2 {
                            normal_flows_debug.push(Edge::Normal {
                                from: b.clone(),
                                to: a.clone(),
                                curved: false,
                            });
                            edges.push(Edge::Path {
                                from: init_fact.clone().clone(),
                                to: a.clone(),
                            });
                        }
                    }
                }
                Instruction::Block(_) | Instruction::Jump(_) => {
                    let before2 = vec![init_fact.clone()];

                    state.add_statement(
                        function,
                        format!("{:?}", instruction),
                        pc + 1,
                        &"taut".to_string(),
                    )?;

                    // Replace the init_fact for the next iteration.
                    // Because, we would skip one row if not.
                    let after3 = state
                        .get_facts_at(&function.name, pc + 1)?
                        .filter(|x| x.var_is_taut)
                        .cloned();

                    init_fact = after3.collect::<Vec<_>>().get(0).unwrap().clone();

                    for b in before2.into_iter() {
                        let after2 = state
                            .get_facts_at(&function.name, pc + 1)?
                            .filter(|x| x.var_is_taut)
                            .cloned();

                        for a in after2 {
                            normal_flows_debug.push(Edge::Normal {
                                from: b.clone(),
                                to: a.clone(),
                                curved: false,
                            });
                            edges.push(Edge::Path {
                                from: init_fact.clone().clone(),
                                to: a.clone(),
                            });
                        }
                    }

                    offset += 1;

                    continue;
                }
                Instruction::Call(_callee, _params, dest) => {
                    for dest_var in dest.iter() {
                        let before2 = vec![init_fact.clone()];

                        state.add_statement(
                            function,
                            format!("{:?}", instruction),
                            pc + 1,
                            dest_var,
                        )?;

                        for b in before2.into_iter() {
                            let after2 = state
                                .get_facts_at(&function.name, pc + 1)?
                                .filter(|x| &x.belongs_to_var == dest_var)
                                .cloned();

                            for a in after2 {
                                normal_flows_debug.push(Edge::Normal {
                                    from: b.clone(),
                                    to: a.clone(),
                                    curved: false,
                                });
                                edges.push(Edge::Path {
                                    from: init_fact.clone().clone(),
                                    to: a.clone(),
                                });
                            }
                        }
                    }
                }
                Instruction::Store(_src, offset, _i) => {
                    let before2 = vec![init_fact.clone()];

                    let mem = state.add_memory_var(function.name.clone(), *offset as usize);
                    state.add_statement(
                        function,
                        format!("{:?}", instruction),
                        pc + 1,
                        &mem.name,
                    )?;

                    for b in before2.into_iter() {
                        let after2 = state
                            .get_facts_at(&function.name, pc + 1)?
                            .filter(|x| &x.belongs_to_var == &mem.name)
                            .cloned();

                        for a in after2 {
                            normal_flows_debug.push(Edge::Normal {
                                from: b.clone(),
                                to: a.clone(),
                                curved: false,
                            });
                            edges.push(Edge::Path {
                                from: init_fact.clone().clone(),
                                to: a.clone(),
                            });
                        }
                    }
                }
                Instruction::Return(_dests) => {
                    let before2 = vec![init_fact.clone()];

                    state.add_statement(
                        function,
                        format!("{:?}", instruction),
                        pc + 1,
                        &"taut".to_string(),
                    )?;

                    for b in before2.into_iter() {
                        let after2 = state
                            .get_facts_at(&function.name, pc + 1)?
                            .filter(|x| x.var_is_taut)
                            .cloned();

                        for a in after2 {
                            normal_flows_debug.push(Edge::Normal {
                                from: b.clone(),
                                to: a.clone(),
                                curved: false,
                            });
                            edges.push(Edge::Path {
                                from: init_fact.clone().clone(),
                                to: a.clone(),
                            });
                        }
                    }
                }
                _ => {
                    bail!(
                        "Selected instruction {:?} is not supported. Please choose the next one.",
                        instruction
                    )
                }
            }

            break;
        }

        Ok(edges)
    }
}
