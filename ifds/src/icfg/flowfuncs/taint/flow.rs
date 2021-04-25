use crate::icfg::flowfuncs::*;
use crate::icfg::state::State;

pub struct TaintNormalFlowFunction;

impl NormalFlowFunction for TaintNormalFlowFunction {
    fn flow(
        &self,
        function: &AstFunction,
        pc: usize,
        variable: &String,
        block_resolver: &BlockResolver,
        state: &mut State,
    ) -> Result<Vec<Edge>> {
        debug!(
            "Calling flow for {} with var {} with pc {}",
            function.name, variable, pc
        );

        let mut edges = Vec::new();

        let instructions = &function.instructions;

        let instruction = instructions
            .get(pc)
            .context("Cannot find instruction when calculating normal flows")?;
        debug!("Next instruction is {:?} for {}", instruction, variable);

        let is_taut = variable == &"taut".to_string();

        match instruction {
            Instruction::Const(reg, _) if reg == variable => {
                //kill
            }
            Instruction::Const(reg, _) if reg != variable && !is_taut => {
                state.add_statement(function, format!("{:?}", instruction), pc + 1, variable)?;

                let before = state
                    .get_facts_at(&function.name, pc)?
                    .filter(|x| &x.belongs_to_var == variable);

                let after = state
                    .get_facts_at(&function.name, pc + 1)?
                    .filter(|x| &x.belongs_to_var == variable);

                for (b, a) in before.zip(after) {
                    edges.push(Edge::Normal {
                        from: b.clone(),
                        to: a.clone(),
                        curved: false,
                    });
                }
            }
            Instruction::Assign(dest, src) if src == variable => {
                state.add_statement(function, format!("{:?}", instruction), pc + 1, dest)?;
                state.add_statement(function, format!("{:?}", instruction), pc + 1, src)?;

                let before = state
                    .get_facts_at(&function.name, pc)?
                    .filter(|x| &x.belongs_to_var == src)
                    .cloned();

                let copy_before = state
                    .get_facts_at(&function.name, pc)?
                    .filter(|x| &x.belongs_to_var == src)
                    .cloned();

                let after = state
                    .get_facts_at(&function.name, pc + 1)?
                    .filter(|x| &x.belongs_to_var == src || &x.belongs_to_var == dest)
                    .cloned();

                for (b, a) in (before.chain(copy_before)).zip(after) {
                    edges.push(Edge::Normal {
                        from: b,
                        to: a,
                        curved: false,
                    });
                }
            }
            Instruction::Assign(_dest, _src) if _dest == variable => {
                //kill
            }
            Instruction::Assign(_dest, _src) => {
                state.add_statement(function, format!("{:?}", instruction), pc + 1, variable)?;

                let before = state
                    .get_facts_at(&function.name, pc)?
                    .filter(|x| &x.belongs_to_var == variable)
                    .cloned();

                let after = state
                    .get_facts_at(&function.name, pc + 1)?
                    .filter(|x| &x.belongs_to_var == variable)
                    .cloned();

                for (b, a) in before.zip(after) {
                    edges.push(Edge::Normal {
                        from: b,
                        to: a,
                        curved: false,
                    });
                }
            }
            Instruction::Unop(dest, src) if src == variable => {
                state.add_statement(function, format!("{:?}", instruction), pc + 1, dest)?;
                state.add_statement(function, format!("{:?}", instruction), pc + 1, src)?;

                let before = state
                    .get_facts_at(&function.name, pc)?
                    .filter(|x| &x.belongs_to_var == src)
                    .cloned();

                let copy_before = state
                    .get_facts_at(&function.name, pc)?
                    .filter(|x| &x.belongs_to_var == src)
                    .cloned();

                let after = state
                    .get_facts_at(&function.name, pc + 1)?
                    .filter(|x| &x.belongs_to_var == src || &x.belongs_to_var == dest)
                    .cloned();

                for (b, a) in (before.chain(copy_before)).zip(after) {
                    edges.push(Edge::Normal {
                        from: b,
                        to: a,
                        curved: false,
                    });
                }
            }
            Instruction::Unop(dest, src) if dest != variable && src != variable => {
                state.add_statement(function, format!("{:?}", instruction), pc + 1, variable)?;

                let before = state
                    .get_facts_at(&function.name, pc)?
                    .filter(|x| &x.belongs_to_var == variable)
                    .cloned();

                let after = state
                    .get_facts_at(&function.name, pc + 1)?
                    .filter(|x| &x.belongs_to_var == variable)
                    .cloned();

                for (b, a) in before.zip(after) {
                    edges.push(Edge::Normal {
                        from: b,
                        to: a,
                        curved: false,
                    });
                }
            }
            Instruction::Unop(_dest, _src) if _dest == variable => {
                //kill
            }
            Instruction::BinOp(dest, src1, _src2) if src1 == variable => {
                state.add_statement(function, format!("{:?}", instruction), pc + 1, dest)?;
                state.add_statement(function, format!("{:?}", instruction), pc + 1, src1)?;

                let before = state
                    .get_facts_at(&function.name, pc)?
                    .filter(|x| &x.belongs_to_var == src1)
                    .cloned();

                let copy_before = state
                    .get_facts_at(&function.name, pc)?
                    .filter(|x| &x.belongs_to_var == src1)
                    .cloned();

                let after = state
                    .get_facts_at(&function.name, pc + 1)?
                    .filter(|x| &x.belongs_to_var == src1 || &x.belongs_to_var == dest)
                    .cloned();

                for (b, a) in (before.chain(copy_before)).zip(after) {
                    edges.push(Edge::Normal {
                        from: b,
                        to: a,
                        curved: false,
                    });
                }
            }
            Instruction::BinOp(dest, _src1, src2) if src2 == variable => {
                state.add_statement(function, format!("{:?}", instruction), pc + 1, dest)?;

                state.add_statement(function, format!("{:?}", instruction), pc + 1, src2)?;

                let before = state
                    .get_facts_at(&function.name, pc)?
                    .filter(|x| &x.belongs_to_var == src2)
                    .cloned();

                let copy_before = state
                    .get_facts_at(&function.name, pc)?
                    .filter(|x| &x.belongs_to_var == src2)
                    .cloned();

                let after = state
                    .get_facts_at(&function.name, pc + 1)?
                    .filter(|x| &x.belongs_to_var == src2 || &x.belongs_to_var == dest)
                    .cloned();

                for (b, a) in (before.chain(copy_before)).zip(after) {
                    edges.push(Edge::Normal {
                        from: b,
                        to: a,
                        curved: false,
                    });
                }
            }
            Instruction::BinOp(_dest, _src1, _src2) if _dest == variable => {
                //kill
            }
            Instruction::Kill(reg) if variable == reg => {
                // kill
            }
            Instruction::Jump(block) => {
                let jump_to_pc = block_resolver
                    .get(&(function.name.clone(), block.clone()))
                    .with_context(|| format!("Cannot find block to jump to {}", block))?;

                state.add_statement(
                    function,
                    format!("{:?}", instruction),
                    *jump_to_pc,
                    variable,
                )?;

                let before = state
                    .get_facts_at(&function.name, pc)?
                    .filter(|x| &x.belongs_to_var == variable)
                    .cloned();

                let after = state
                    .get_facts_at(&function.name, *jump_to_pc)?
                    .filter(|x| &x.belongs_to_var == variable)
                    .cloned();

                for (b, a) in before.zip(after) {
                    edges.push(Edge::Normal {
                        from: b,
                        to: a,
                        curved: true,
                    });
                }
            }
            Instruction::Conditional(_reg, jumps) if jumps.len() == 1 => {
                // edge case for an `if`
                // which continues if the condition is not successful
                for block in jumps.iter() {
                    let jump_to_pc = block_resolver
                        .get(&(function.name.clone(), block.clone()))
                        .with_context(|| format!("Cannot find block to jump to {}", block))?;

                    state.add_statement(
                        function,
                        format!("{:?}", instruction),
                        *jump_to_pc,
                        variable,
                    )?;

                    let before = state
                        .get_facts_at(&function.name, pc)?
                        .filter(|x| &x.belongs_to_var == variable)
                        .cloned();

                    let after = state
                        .get_facts_at(&function.name, *jump_to_pc)?
                        .filter(|x| &x.belongs_to_var == variable)
                        .cloned();

                    for (b, a) in before.zip(after) {
                        edges.push(Edge::Normal {
                            from: b,
                            to: a,
                            curved: true,
                        });
                    }
                }

                state.add_statement(function, format!("{:?}", instruction), pc + 1, variable)?;

                let before = state
                    .get_facts_at(&function.name, pc)?
                    .filter(|x| &x.belongs_to_var == variable)
                    .cloned();

                let after = state
                    .get_facts_at(&function.name, pc + 1)?
                    .filter(|x| &x.belongs_to_var == variable)
                    .cloned();

                for (b, a) in before.zip(after) {
                    edges.push(Edge::Normal {
                        from: b,
                        to: a,
                        curved: true,
                    });
                }
            }
            Instruction::Conditional(_reg, jumps) => {
                for block in jumps.iter() {
                    let jump_to_pc = block_resolver
                        .get(&(function.name.clone(), block.clone()))
                        .with_context(|| format!("Cannot find block to jump to {}", block))?;

                    state.add_statement(
                        function,
                        format!("{:?}", instruction),
                        *jump_to_pc,
                        variable,
                    )?;

                    let before = state
                        .get_facts_at(&function.name, pc)?
                        .filter(|x| &x.belongs_to_var == variable)
                        .cloned();

                    let after = state
                        .get_facts_at(&function.name, *jump_to_pc)?
                        .filter(|x| &x.belongs_to_var == variable)
                        .cloned();

                    for (b, a) in before.zip(after) {
                        edges.push(Edge::Normal {
                            from: b,
                            to: a,
                            curved: true,
                        });
                    }
                }
            }
            Instruction::Phi(dest, src1, _src2) if src1 == variable => {
                state.add_statement(function, format!("{:?}", instruction), pc + 1, dest)?;

                state.add_statement(function, format!("{:?}", instruction), pc + 1, src1)?;

                let before = state
                    .get_facts_at(&function.name, pc)?
                    .filter(|x| &x.belongs_to_var == src1)
                    .cloned();

                let copy_before = state
                    .get_facts_at(&function.name, pc)?
                    .filter(|x| &x.belongs_to_var == src1)
                    .cloned();

                let after = state
                    .get_facts_at(&function.name, pc + 1)?
                    .filter(|x| &x.belongs_to_var == src1 || &x.belongs_to_var == dest)
                    .cloned();

                for (b, a) in (before.chain(copy_before)).zip(after) {
                    edges.push(Edge::Normal {
                        from: b,
                        to: a,
                        curved: false,
                    });
                }
            }
            Instruction::Phi(dest, _src1, src2) if src2 == variable => {
                state.add_statement(function, format!("{:?}", instruction), pc + 1, dest)?;

                state.add_statement(function, format!("{:?}", instruction), pc + 1, src2)?;

                let before = state
                    .get_facts_at(&function.name, pc)?
                    .filter(|x| &x.belongs_to_var == src2)
                    .cloned();

                let copy_before = state
                    .get_facts_at(&function.name, pc)?
                    .filter(|x| &x.belongs_to_var == src2)
                    .cloned();

                let after = state
                    .get_facts_at(&function.name, pc + 1)?
                    .filter(|x| &x.belongs_to_var == src2 || &x.belongs_to_var == dest)
                    .cloned();

                for (b, a) in (before.chain(copy_before)).zip(after) {
                    edges.push(Edge::Normal {
                        from: b,
                        to: a,
                        curved: false,
                    });
                }
            }
            Instruction::Phi(dest, src1, src2)
                if dest != variable && src1 != variable && src2 != variable =>
            {
                state.add_statement(function, format!("{:?}", instruction), pc + 1, variable)?;

                // Identity
                let before = state
                    .get_facts_at(&function.name, pc)?
                    .into_iter()
                    .filter(|x| &x.belongs_to_var == variable)
                    .cloned();

                let after = state
                    .get_facts_at(&function.name, pc + 1)?
                    .filter(|x| &x.belongs_to_var == variable)
                    .cloned();

                for (b, a) in before.zip(after) {
                    edges.push(Edge::Normal {
                        from: b,
                        to: a,
                        curved: false,
                    });
                }
            }
            Instruction::Phi(_dest, _src1, _src2) => {
                //kill
            }
            Instruction::Table(jumps) => {
                for block in jumps.iter() {
                    let jump_to_pc = block_resolver
                        .get(&(function.name.clone(), block.clone()))
                        .with_context(|| format!("Cannot find block to jump to {}", block))?;

                    state.add_statement(
                        function,
                        format!("{:?}", instruction),
                        *jump_to_pc,
                        variable,
                    )?;

                    let before = state
                        .get_facts_at(&function.name, pc)?
                        .into_iter()
                        .filter(|x| &x.belongs_to_var == variable)
                        .cloned();

                    let after = state
                        .get_facts_at(&function.name, *jump_to_pc)?
                        .filter(|x| &x.belongs_to_var == variable)
                        .cloned();

                    for (b, a) in before.zip(after) {
                        edges.push(Edge::Normal {
                            from: b,
                            to: a,
                            curved: true,
                        });
                    }
                }
            }
            Instruction::Return(dest) if dest.contains(variable) => {
                state.add_statement(function, format!("{:?}", instruction), pc + 1, variable)?;

                // Identity
                let before = state
                    .get_facts_at(&function.name, pc)?
                    .into_iter()
                    .filter(|x| &x.belongs_to_var == variable)
                    .cloned();

                let after = state
                    .get_facts_at(&function.name, pc + 1)?
                    .filter(|x| &x.belongs_to_var == variable)
                    .cloned();

                for (b, a) in before.zip(after) {
                    edges.push(Edge::Normal {
                        from: b,
                        to: a,
                        curved: false,
                    });
                }
            }
            Instruction::Store(src, offset, i) if variable == src || variable == i => {
                let mem_var = state.add_memory_var(function.name.clone(), *offset as usize);

                state.add_statement(
                    function,
                    format!("{:?}", instruction),
                    pc + 1,
                    &mem_var.name,
                )?;

                state.add_statement(function, format!("{:?}", instruction), pc + 1, &variable)?;

                let before = state
                    .get_facts_at(&function.name, pc)?
                    .filter(|x| &x.belongs_to_var == src)
                    .cloned();

                let after = state
                    .get_facts_at(&function.name, pc + 1)?
                    .filter(|x| x.belongs_to_var == mem_var.name || &x.belongs_to_var == variable)
                    .cloned()
                    .collect::<Vec<_>>();

                for b in before {
                    for a in after.iter() {
                        edges.push(Edge::Normal {
                            from: b.clone(),
                            to: a.clone(),
                            curved: false,
                        });
                    }
                }
            }
            Instruction::Store(src, offset, i)
                if variable != src
                    && variable != i
                    && variable != &"taut".to_string()
                    && variable != &format!("mem@{}", offset) =>
            {
                state.add_statement(function, format!("{:?}", instruction), pc + 1, variable)?;

                // Identity
                let before = state
                    .get_facts_at(&function.name, pc)?
                    .filter(|x| &x.belongs_to_var == variable)
                    .cloned();

                let after = state
                    .get_facts_at(&function.name, pc + 1)?
                    .filter(|x| &x.belongs_to_var == variable)
                    .cloned();

                for (b, a) in before.zip(after) {
                    edges.push(Edge::Normal {
                        from: b,
                        to: a,
                        curved: false,
                    });
                }
            }
            Instruction::Store(_src, _offset, _i) => {
                // kill
            }
            Instruction::Load(dest, _offset, _i)
                if variable != dest && variable != _i && !variable.starts_with("mem") =>
            {
                state.add_statement(function, format!("{:?}", instruction), pc + 1, variable)?;

                // Identity
                let before = state
                    .get_facts_at(&function.name, pc)?
                    .into_iter()
                    .filter(|x| &x.belongs_to_var == variable)
                    .cloned();

                let after = state
                    .get_facts_at(&function.name, pc + 1)?
                    .filter(|x| &x.belongs_to_var == variable)
                    .cloned();

                for (b, a) in before.zip(after) {
                    edges.push(Edge::Normal {
                        from: b,
                        to: a,
                        curved: false,
                    });
                }
            }
            Instruction::Load(dest, _offset, _i) => {
                state.add_statement(function, format!("{:?}", instruction), pc + 1, &dest)?;
                state.add_statement(function, format!("{:?}", instruction), pc + 1, &variable)?;

                // Here happens the overtainting, because we take all
                // memory variables
                let before = state
                    .get_facts_at(&function.name, pc)?
                    .into_iter()
                    .filter(|x| x.memory_offset.is_some() || _i == variable);

                let after = state
                    .get_facts_at(&function.name, pc + 1)?
                    .filter(|x| &x.belongs_to_var == variable || &x.belongs_to_var == dest)
                    .cloned()
                    .collect::<Vec<_>>();

                for b in before {
                    for a in after.iter() {
                        edges.push(Edge::Normal {
                            from: b.clone(),
                            to: a.clone(),
                            curved: false,
                        });
                    }
                }
            }
            _ => {
                state.add_statement(function, format!("{:?}", instruction), pc + 1, variable)?;

                // Identity
                let before = state
                    .get_facts_at(&function.name, pc)?
                    .filter(|x| &x.belongs_to_var == variable);

                let after = state
                    .get_facts_at(&function.name, pc + 1)?
                    .filter(|x| &x.belongs_to_var == variable);

                for (b, a) in before.zip(after) {
                    edges.push(Edge::Normal {
                        from: b.clone(),
                        to: a.clone(),
                        curved: false,
                    });
                }
            }
        }

        Ok(edges)
    }
}
