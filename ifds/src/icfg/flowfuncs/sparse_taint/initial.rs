use crate::icfg::{flowfuncs::*, tabulation::sparse::Ctx};

pub struct SparseTaintInitialFlowFunction;

impl SparseInitialFlowFunction for SparseTaintInitialFlowFunction {
    fn flow<'a>(
        &self,
        ctx: &mut Ctx<'a>,
        function: &AstFunction,
        pc: usize,
        init_facts: &Vec<Fact>,
        normal_flows_debug: &mut Vec<Edge>,
        defuse: &mut DefUseChain,
    ) -> Result<Vec<Edge>> {
        debug!("Calling init flow for {} with pc {}", function.name, pc);

        let mut edges = Vec::new();

        // We need the offset, because not every
        // instruction is taintable. For example BLOCK and JUMP
        // have no registers. That's why skip to the next one.
        let offset = 0;
        let instructions = &function.instructions;

        let init_fact = init_facts.get(0).context("Cannot find taut")?.clone();

        debug!("init fact is {:#?}", init_fact);

        loop {
            let pc = pc + offset;
            let instruction = instructions.get(pc).context("Cannot find instr")?;
            debug!("Next instruction is {:?}", instruction);

            match instruction {
                Instruction::Const(dest, _)
                | Instruction::Unknown(dest)
                | Instruction::Assign(dest, _)
                | Instruction::Unop(dest, _)
                | Instruction::BinOp(dest, _, _)
                | Instruction::Phi(dest, _, _)
                | Instruction::Conditional(dest, _)
                | Instruction::Load(dest, _, _) => {
                    let before = vec![init_fact.clone()];

                    for b in before.into_iter() {
                        // The tautological fact was built by the `pacemaker`
                        // and will not be sparsely propagated.

                        let after_taut = defuse
                            .demand(ctx, &function, &"taut".to_string(), b.pc)
                            .context("Cannot find taut's fact")?;

                        debug!("Next taut is {:#?}", after_taut);

                        for taut in after_taut.into_iter() {
                            normal_flows_debug.push(Edge::Normal {
                                from: b.clone(),
                                to: taut.clone(),
                                curved: false,
                            });

                            edges.push(Edge::Path {
                                from: init_fact.clone().clone(),
                                to: taut.clone(),
                            });
                        }

                        let after_var = defuse
                            .get_next(ctx, &function, dest, pc)
                            .context("Cannot find var's fact")?;

                        for var in after_var.into_iter() {
                            let applied = var.apply();

                            normal_flows_debug.push(Edge::Normal {
                                from: b.clone(),
                                to: applied.clone(),
                                curved: false,
                            });

                            edges.push(Edge::Path {
                                from: init_fact.clone().clone(),
                                to: applied.clone(),
                            });
                        }
                    }
                }
                Instruction::Call(_callee, _, dests) => {
                    for dest in dests {
                        let before = vec![init_fact.clone()];

                        for b in before.into_iter() {
                            // The tautological fact was built by the `pacemaker`
                            // and will not be sparsely propagated.

                            let after_taut = defuse
                                .demand(ctx, &function, &"taut".to_string(), pc)
                                .context("Cannot find taut's fact")?;

                            for taut in after_taut.into_iter() {
                                normal_flows_debug.push(Edge::Normal {
                                    from: b.clone(),
                                    to: taut.clone(),
                                    curved: false,
                                });

                                edges.push(Edge::Path {
                                    from: init_fact.clone().clone(),
                                    to: taut.clone(),
                                });
                            }

                            let after_var = defuse
                                .demand(ctx, &function, dest, pc)
                                .context("Cannot find var's fact")?;

                            for var in after_var.into_iter() {
                                let mut applied = var.clone();
                                applied.pc += 1;

                                assert!(applied.pc <= applied.next_pc);

                                normal_flows_debug.push(Edge::Normal {
                                    from: b.clone(),
                                    to: applied.clone(),
                                    curved: false,
                                });

                                edges.push(Edge::Path {
                                    from: init_fact.clone().clone(),
                                    to: applied.clone(),
                                });
                            }
                        }
                    }
                }
                Instruction::Store(_src, offset, _) => {
                    let before = vec![init_fact.clone()];

                    for b in before.into_iter() {
                        // The tautological fact was built by the `pacemaker`
                        // and will not be sparsely propagated.

                        let after_taut = defuse
                            .demand(ctx, &function, &"taut".to_string(), pc)
                            .context("Cannot find taut's fact")?;

                        for taut in after_taut.into_iter() {
                            normal_flows_debug.push(Edge::Normal {
                                from: b.clone(),
                                to: taut.clone(),
                                curved: false,
                            });

                            edges.push(Edge::Path {
                                from: init_fact.clone().clone(),
                                to: taut.clone(),
                            });
                        }

                        let mem = ctx
                            .state
                            .add_memory_var(function.name.clone(), offset.clone() as usize);

                        let after_var = defuse
                            .demand(ctx, &function, &mem.name, pc)
                            .context("Cannot find var's fact")?;

                        for var in after_var.into_iter() {
                            let mut applied = var.clone();
                            applied.pc += 1;

                            assert!(applied.pc <= applied.next_pc);
                            normal_flows_debug.push(Edge::Normal {
                                from: b.clone(),
                                to: applied.clone(),
                                curved: false,
                            });

                            edges.push(Edge::Path {
                                from: init_fact.clone().clone(),
                                to: applied.clone(),
                            });
                        }
                    }
                }
                Instruction::Return(_dests) => {
                    let before = vec![init_fact.clone()];

                    for b in before.into_iter() {
                        // The tautological fact was built by the `pacemaker`
                        // and will not be sparsely propagated.

                        let after_taut = defuse
                            .demand(ctx, &function, &"taut".to_string(), pc)
                            .context("Cannot find taut's fact")?;

                        for taut in after_taut.into_iter() {
                            normal_flows_debug.push(Edge::Normal {
                                from: b.clone(),
                                to: taut.clone(),
                                curved: false,
                            });

                            edges.push(Edge::Path {
                                from: init_fact.clone().clone(),
                                to: taut.clone(),
                            });
                        }

                        let after_var = defuse
                            .demand(ctx, &function, &"taut".to_string(), pc)
                            .context("Cannot find var's fact")?;

                        for var in after_var.into_iter() {
                            let applied = var.apply(); 

                            assert!(applied.pc <= applied.next_pc);
                            normal_flows_debug.push(Edge::Normal {
                                from: b.clone(),
                                to: var.clone(),
                                curved: false,
                            });

                            edges.push(Edge::Path {
                                from: init_fact.clone().clone(),
                                to: var.clone(),
                            });
                        }
                    }
                }
                Instruction::Block(_) | Instruction::Jump(_) => {
                    panic!("Block or Jump as first instruction is not supported");
                }
                _ => {}
            }

            break;
        }

        Ok(edges)
    }
}
