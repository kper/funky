use crate::icfg::flowfuncs::*;
use crate::icfg::state::State;

pub struct TaintNormalFlowFunction;

impl NormalFlowFunction for TaintNormalFlowFunction {
    fn flow(
        &self,
        function: &AstFunction,
        graph: &mut Graph,
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

        let instruction = instructions.get(pc).context("Cannot find instr")?;
        debug!("Next instruction is {:?} for {}", instruction, variable);

        let is_taut = variable == &"taut".to_string();

        match instruction {
            Instruction::Const(reg, _) if reg == variable => {
                //kill
            }
            Instruction::Const(reg, _) if reg != variable && !is_taut => {
                let after = state.add_statement(
                    function,
                    format!("{:?}", instruction),
                    pc + 1,
                    variable,
                )?;

                let before = state
                    .get_facts_at(&function.name, pc)?
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
            Instruction::Assign(dest, src) if src == variable => {
                let mut after =
                    state.add_statement(function, format!("{:?}", instruction), pc + 1, dest)?;

                let after2 =
                    state.add_statement(function, format!("{:?}", instruction), pc + 1, src)?;

                after.extend(after2);

                let before = state
                    .get_facts_at(&function.name, pc)?
                    .filter(|x| &x.belongs_to_var == src)
                    .cloned();

                let copy_before = state
                    .get_facts_at(&function.name, pc)?
                    .filter(|x| &x.belongs_to_var == src)
                    .cloned();

                for (b, a) in (before.chain(copy_before)).zip(after) {
                    edges.push(Edge::Normal {
                        from: b,
                        to: a,
                        curved: false,
                    });
                }
            }
            Instruction::Assign(dest, src) if dest != variable && src != variable => {
                let after = state.add_statement(
                    function,
                    format!("{:?}", instruction),
                    pc + 1,
                    variable,
                )?;

                let before = state 
                    .get_facts_at(&function.name, pc)?
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
            Instruction::Assign(_dest, _src) if _dest == variable => {
                //kill
            }
            Instruction::Unop(dest, src) if src == variable => {
                let mut after =
                    state.add_statement(function, format!("{:?}", instruction), pc + 1, dest)?;

                let after2 =
                    state.add_statement(function, format!("{:?}", instruction), pc + 1, src)?;

                after.extend(after2);

                let before = state
                    .get_facts_at(&function.name, pc)?
                    .filter(|x| &x.belongs_to_var == src)
                    .cloned();

                let copy_before = state
                    .get_facts_at(&function.name, pc)?
                    .filter(|x| &x.belongs_to_var == src)
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
                let after = state.add_statement(
                    function,
                    format!("{:?}", instruction),
                    pc + 1,
                    variable,
                )?;

                let before = state
                    .get_facts_at(&function.name, pc)?
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
                let mut after =
                    state.add_statement(function, format!("{:?}", instruction), pc + 1, dest)?;

                let after2 =
                    state.add_statement(function, format!("{:?}", instruction), pc + 1, src1)?;

                after.extend(after2);

                let before = state
                    .get_facts_at(&function.name, pc)?
                    .filter(|x| &x.belongs_to_var == src1)
                    .cloned();

                let copy_before = state
                    .get_facts_at(&function.name, pc)?
                    .filter(|x| &x.belongs_to_var == src1)
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
                let mut after =
                    state.add_statement(function, format!("{:?}", instruction), pc + 1, dest)?;

                let after2 =
                    state.add_statement(function, format!("{:?}", instruction), pc + 1, src2)?;

                after.extend(after2);

                let before = state
                    .get_facts_at(&function.name, pc)?
                    .filter(|x| &x.belongs_to_var == src2)
                    .cloned();

                let copy_before = state
                    .get_facts_at(&function.name, pc)?
                    .filter(|x| &x.belongs_to_var == src2)
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

                let after = state.add_statement(
                    function,
                    format!("{:?}", instruction),
                    *jump_to_pc,
                    variable,
                )?;

                let before = state
                    .get_facts_at(&function.name, pc)?
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

                    let after = state.add_statement(
                        function,
                        format!("{:?}", instruction),
                        *jump_to_pc,
                        variable,
                    )?;

                    let before = state
                        .get_facts_at(&function.name, pc)?
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

                let after = state.add_statement(
                    function,
                    format!("{:?}", instruction),
                    pc + 1,
                    variable,
                )?;

                let before = state
                    .get_facts_at(&function.name, pc)?
                    .into_iter()
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

                    let after = state.add_statement(
                        function,
                        format!("{:?}", instruction),
                        *jump_to_pc,
                        variable,
                    )?;

                    let before = state
                        .get_facts_at(&function.name, pc)?
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
                let mut after =
                    state.add_statement(function, format!("{:?}", instruction), pc + 1, dest)?;

                let after2 =
                    state.add_statement(function, format!("{:?}", instruction), pc + 1, src1)?;

                after.extend(after2);

                let before = state
                    .get_facts_at(&function.name, pc)?
                    .filter(|x| &x.belongs_to_var == src1)
                    .cloned();

                let copy_before = state
                    .get_facts_at(&function.name, pc)?
                    .filter(|x| &x.belongs_to_var == src1)
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
                let mut after =
                    state.add_statement(function, format!("{:?}", instruction), pc + 1, dest)?;

                let after2 =
                    state.add_statement(function, format!("{:?}", instruction), pc + 1, src2)?;

                after.extend(after2);

                let before = state
                    .get_facts_at(&function.name, pc)?
                    .filter(|x| &x.belongs_to_var == src2)
                    .cloned();

                let copy_before = state
                    .get_facts_at(&function.name, pc)?
                    .filter(|x| &x.belongs_to_var == src2)
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
                let after = state.add_statement(
                    function,
                    format!("{:?}", instruction),
                    pc + 1,
                    variable,
                )?;

                // Identity
                let before = state
                    .get_facts_at(&function.name, pc)?
                    .into_iter()
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

                    let after = state.add_statement(
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
                let after = state.add_statement(
                    function,
                    format!("{:?}", instruction),
                    pc + 1,
                    variable,
                )?;

                // Identity
                let before = state
                    .get_facts_at(&function.name, pc)?
                    .into_iter()
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
                let mem_var =
                    state.add_memory_var("mem".to_string(), function.name.clone(), *offset);

                let after = state.add_statement(
                    function,
                    format!("{:?}", instruction),
                    pc + 1,
                    &mem_var.name,
                )?;

                let after_var = state.add_statement(
                    function,
                    format!("{:?}", instruction),
                    pc + 1,
                    &variable,
                )?;

                // Identity
                let before = state
                    .get_facts_at(&function.name, pc)?
                    .into_iter()
                    .filter(|x| &x.belongs_to_var == src)
                    .cloned();

                for b in before {
                    for a in after.iter().chain(after_var.iter()) {
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
                let after = state.add_statement(
                    function,
                    format!("{:?}", instruction),
                    pc + 1,
                    variable,
                )?;

                // Identity
                let before = state
                    .get_facts_at(&function.name, pc)?
                    .into_iter()
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
                let after = state.add_statement(
                    function,
                    format!("{:?}", instruction),
                    pc + 1,
                    variable,
                )?;

                // Identity
                let before = state
                    .get_facts_at(&function.name, pc)?
                    .into_iter()
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
            Instruction::Load(dest, offset, _i) => {
                let after =
                    state.add_statement(function, format!("{:?}", instruction), pc + 1, &dest)?;

                let after_var = state.add_statement(
                    function,
                    format!("{:?}", instruction),
                    pc + 1,
                    &variable,
                )?;

                // we cannot know which exact variables, because we only
                // know the `offset`. But, we can eliminate all cases
                // where the offset is higher, so they can't be meant.
                let before = state
                    .get_facts_at(&function.name, pc)?
                    .into_iter()
                    .filter(|x| {
                        (x.memory_offset.is_some() && x.memory_offset <= Some(*offset))
                            || _i == variable
                    });

                for b in before {
                    for a in after.iter().chain(after_var.iter()) {
                        edges.push(Edge::Normal {
                            from: b.clone(),
                            to: a.clone(),
                            curved: false,
                        });
                    }
                }
            }
            _ => {
                let after = state.add_statement(
                    function,
                    format!("{:?}", instruction),
                    pc + 1,
                    variable,
                )?;

                // Identity
                let before = state
                    .get_facts_at(&function.name, pc)?
                    .into_iter()
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
        }

        Ok(edges)
    }
}
