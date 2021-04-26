use crate::icfg::state::State;
use crate::icfg::{flowfuncs::*, tabulation::sparse::Ctx};

pub struct SparseTaintNormalFlowFunction;

impl SparseNormalFlowFunction for SparseTaintNormalFlowFunction {
    fn flow<'a>(
        &self,
        ctx: &mut Ctx<'a>,
        function: &AstFunction,
        pc: usize,
        variable: &String,
        block_resolver: &BlockResolver,
        defuse: &mut DefUseChain,
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
            Instruction::Const(reg, _) if reg == variable || is_taut => {
                //kill
            }
            Instruction::Assign(dest, _) if dest == variable || is_taut => {
                //kill
            }
            Instruction::Unop(dest, _) if dest == variable || is_taut => {
                //kill
            }
            Instruction::BinOp(dest, _, _) if dest == variable || is_taut => {
                //kill
            }
            Instruction::Kill(reg) if reg == variable || is_taut=> {
                //kill
            }
            Instruction::Phi(dest, _, _) if dest == variable || is_taut => {
                //kill
            }
            Instruction::Unknown(reg) if reg == variable || is_taut => {
                //kill
            }
            Instruction::Const(dest, _) | Instruction::Unknown(dest) => {
                let before = ctx
                    .state
                    .get_facts_at(&function.name, pc)
                    .context("Cannot find taut's fact")?
                    .filter(|x| x.var_is_taut)
                    .collect::<Vec<_>>()
                    .get(0)
                    .context("Cannot find taut")?
                    .clone()
                    .clone();

                let after_var = defuse
                    .demand_current(ctx, &function, dest, pc)
                    .context("Cannot find var's fact")?;

                for var in after_var.into_iter() {
                    edges.push(Edge::Normal {
                        from: before.clone(),
                        to: var.clone(),
                        curved: false,
                    });
                }
            }
            Instruction::Assign(dest, src) | Instruction::Unop(dest, src) => {
                let before = defuse
                    .src_before(ctx, &function, src, pc)
                    .context("Cannot find var's fact")?
                    .into_iter()
                    .map(|x| x.clone())
                    .collect::<Vec<_>>();

                let after_var = defuse
                    .demand_current(ctx, &function, dest, pc)
                    .context("Cannot find var's fact")?;

                for b in before {
                    for var in after_var.iter() {
                        edges.push(Edge::Normal {
                            from: b.clone(),
                            to: var.clone().clone(),
                            curved: false,
                        });
                    }
                }
            }
            Instruction::BinOp(dest, src1, src2) | Instruction::Phi(dest, src1, src2) => {
                let before = defuse
                    .src_before(ctx, &function, src1, pc)
                    .context("Cannot find var's fact")?
                    .into_iter()
                    .map(|x| x.clone())
                    .collect::<Vec<_>>();

                debug!("before {:#?}", before);

                let before2 = defuse
                    .src_before(ctx, &function, src2, pc)
                    .context("Cannot find var's fact")?
                    .into_iter()
                    .map(|x| x.clone())
                    .collect::<Vec<_>>();

                debug!("before2 {:#?}", before2);

                let after_var = defuse
                    .demand_current(ctx, &function, dest, pc)
                    .context("Cannot find var's fact")?;

                for b in before.into_iter().chain(before2) {
                    for var in after_var.iter() {
                        edges.push(Edge::Normal {
                            from: b.clone(),
                            to: var.clone().clone(),
                            curved: false,
                        });
                    }
                }
            }
            Instruction::Conditional(_, jumps) => {
                for block in jumps.iter() {
                    let jump_to_pc = block_resolver
                        .get(&(function.name.clone(), block.clone()))
                        .with_context(|| format!("Cannot find block to jump to {}", block))?
                        .clone();

                    let before = defuse
                        .src_before(ctx, &function, variable, pc)
                        .context("Cannot get facts")?
                        .into_iter()
                        .map(|x| x.clone())
                        .collect::<Vec<_>>();

                    let after = defuse
                        .demand_current(ctx, function, variable, jump_to_pc)
                        .context("Cannot get facts")?;

                    for b in before.into_iter() {
                        for var in after.iter() {
                            edges.push(Edge::Normal {
                                from: b.clone(),
                                to: var.clone().clone(),
                                curved: true,
                            });
                        }
                    }
                }
            }
            _ => {}
        }

        Ok(edges)
    }
}
