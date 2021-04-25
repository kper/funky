use crate::icfg::{flowfuncs::*, tabulation::sparse::Ctx};
use anyhow::bail;

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
        let mut offset = 0;
        let instructions = &function.instructions;

        let mut init_fact = init_facts.get(0).context("Cannot find taut")?.clone();

        loop {
            let pc = pc + offset;
            let instruction = instructions.get(pc).context("Cannot find instr")?;
            debug!("Next instruction is {:?}", instruction);

            ctx.state.add_statement(
                function,
                format!("{:?}", instruction),
                pc,
                &"taut".to_string(),
            )?;

            let _after = ctx
                .state
                .get_facts_at(&function.name, pc)?
                .filter(|x| x.var_is_taut)
                .collect::<Vec<_>>();

            let after = _after.get(0).unwrap();

            edges.push(Edge::Path {
                from: init_fact.clone().clone(),
                to: after.clone().clone(),
            });

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

                        let after_taut = ctx
                            .state
                            .get_facts_at(&function.name, pc + 1)
                            .context("Cannot find taut's fact")?;

                        for taut in after_taut.into_iter().take(1) {
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
                            .cache(ctx, &function, dest, pc)
                            .context("Cannot find var's fact")?;

                        for var in after_var.into_iter().take(1) {
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
                Instruction::Call(_callee, _, dests) => {
                    for dest in dests {
                        let before = vec![init_fact.clone()];

                        for b in before.into_iter() {
                            // The tautological fact was built by the `pacemaker`
                            // and will not be sparsely propagated.

                            let after_taut = ctx
                                .state
                                .get_facts_at(&function.name, pc + 1)
                                .context("Cannot find taut's fact")?;

                            for taut in after_taut.into_iter().take(1) {
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
                                .cache(ctx, &function, dest, pc)
                                .context("Cannot find var's fact")?;

                            for var in after_var.into_iter().take(1) {
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
                }
                Instruction::Store(_src, offset, _) => {
                    let before = vec![init_fact.clone()];

                    for b in before.into_iter() {
                        // The tautological fact was built by the `pacemaker`
                        // and will not be sparsely propagated.

                        let after_taut = ctx
                            .state
                            .get_facts_at(&function.name, pc + 1)
                            .context("Cannot find taut's fact")?;

                        for taut in after_taut.into_iter().take(1) {
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
                            .add_memory_var(function.name.clone(), offset.clone());

                        let after_var = defuse
                            .cache(ctx, &function, &mem.name, pc)
                            .context("Cannot find var's fact")?;

                        for var in after_var.into_iter().take(1) {
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
                Instruction::Return(_dests) => {
                    let before = vec![init_fact.clone()];

                    for b in before.into_iter() {
                        // The tautological fact was built by the `pacemaker`
                        // and will not be sparsely propagated.

                        let after_taut = ctx
                            .state
                            .get_facts_at(&function.name, pc + 1)
                            .context("Cannot find taut's fact")?;

                        for taut in after_taut.into_iter().take(1) {
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
                            .cache(ctx, &function, &"taut".to_string(), pc)
                            .context("Cannot find var's fact")?;

                        for var in after_var.into_iter().take(1) {
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
                    unimplemented!()
                }
                _ => {}
            }

            break;
        }

        Ok(edges)
    }
}
