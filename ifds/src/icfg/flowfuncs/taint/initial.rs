use crate::icfg::flowfuncs::*;

pub struct TaintInitialFlowFunction;

impl InitialFlowFunction for TaintInitialFlowFunction {
    fn flow(
        &self,
        function: &AstFunction,
        graph: &mut Graph,
        pc: usize,
        init_facts: &Vec<Fact>,
        normal_flows_debug: &mut Vec<Edge>,
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

            let _after2 = graph.add_statement(
                function,
                format!("{:?}", instruction),
                pc,
                &"taut".to_string(),
            )?;
            let after2 = _after2.get(0).unwrap();

            edges.push(Edge::Path {
                from: init_fact.clone().clone(),
                to: after2.clone(),
            });

            match instruction {
                Instruction::Const(reg, _) => {
                    let before2 = vec![init_fact.clone()];

                    let after2 =
                        graph.add_statement(function, format!("{:?}", instruction), pc + 1, reg)?;

                    for (b, a) in before2.into_iter().zip(after2) {
                        normal_flows_debug.push(Edge::Normal {
                            from: b.clone(),
                            to: a.clone(),
                            curved: false,
                        });
                        edges.push(Edge::Path {
                            from: init_fact.clone().clone(),
                            to: a,
                        });
                    }
                }
                Instruction::Assign(dest, _src) => {
                    let before2 = vec![init_fact.clone()];

                    let after2 = graph.add_statement(
                        function,
                        format!("{:?}", instruction),
                        pc + 1,
                        dest,
                    )?;

                    for (b, a) in before2.into_iter().zip(after2) {
                        normal_flows_debug.push(Edge::Normal {
                            from: b.clone(),
                            to: a.clone(),
                            curved: false,
                        });
                        edges.push(Edge::Path {
                            from: init_fact.clone().clone(),
                            to: a,
                        });
                    }
                }
                Instruction::Unop(dest, _src) => {
                    let before2 = vec![init_fact.clone()];

                    let after2 = graph.add_statement(
                        function,
                        format!("{:?}", instruction),
                        pc + 1,
                        dest,
                    )?;

                    for (b, a) in before2.into_iter().zip(after2) {
                        normal_flows_debug.push(Edge::Normal {
                            from: b.clone(),
                            to: a.clone(),
                            curved: false,
                        });
                        edges.push(Edge::Path {
                            from: init_fact.clone().clone(),
                            to: a,
                        });
                    }
                }
                Instruction::BinOp(dest, _src, _src2) => {
                    let before2 = vec![init_fact.clone()];

                    let after2 = graph.add_statement(
                        function,
                        format!("{:?}", instruction),
                        pc + 1,
                        dest,
                    )?;

                    for (b, a) in before2.into_iter().zip(after2) {
                        normal_flows_debug.push(Edge::Normal {
                            from: b.clone(),
                            to: a.clone(),
                            curved: false,
                        });
                        edges.push(Edge::Path {
                            from: init_fact.clone().clone(),
                            to: a,
                        });
                    }
                }
                Instruction::Conditional(reg, _) => {
                    let before2 = vec![init_fact.clone()];

                    let after2 =
                        graph.add_statement(function, format!("{:?}", instruction), pc + 1, reg)?;

                    for (b, a) in before2.into_iter().zip(after2) {
                        normal_flows_debug.push(Edge::Normal {
                            from: b.clone(),
                            to: a.clone(),
                            curved: false,
                        });
                        edges.push(Edge::Path {
                            from: init_fact.clone().clone(),
                            to: a,
                        });
                    }
                }
                Instruction::Block(_) | Instruction::Jump(_) => {
                    let before2 = vec![init_fact.clone()];

                    let after2 = graph.add_statement(
                        function,
                        format!("{:?}", instruction),
                        pc + 1,
                        &"taut".to_string(),
                    )?;

                    for (b, a) in before2.into_iter().zip(after2.clone()) {
                        normal_flows_debug.push(Edge::Normal {
                            from: b.clone(),
                            to: a.clone(),
                            curved: false,
                        });
                        edges.push(Edge::Path {
                            from: init_fact.clone().clone(),
                            to: a,
                        });
                    }

                    offset += 1;

                    // Replace the init_fact for the next iteration.
                    // Because, we would skip one row if not.
                    init_fact = after2.get(0).unwrap().clone();

                    continue;
                }
                Instruction::Phi(dest, _src1, _src2) => {
                    let before2 = vec![init_fact.clone()];

                    let after2 = graph.add_statement(
                        function,
                        format!("{:?}", instruction),
                        pc + 1,
                        dest,
                    )?;

                    for (b, a) in before2.into_iter().zip(after2) {
                        normal_flows_debug.push(Edge::Normal {
                            from: b.clone(),
                            to: a.clone(),
                            curved: false,
                        });
                        edges.push(Edge::Path {
                            from: init_fact.clone().clone(),
                            to: a,
                        });
                    }
                }
                _ => {}
            }

            break;
        }

        Ok(edges)
    }
}
