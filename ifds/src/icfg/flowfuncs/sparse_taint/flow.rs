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
            Instruction::Const(reg, _) if reg == variable => {
                //kill
            }
            Instruction::Const(dest, _) => {
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
                    .demand(ctx, &function, dest, pc)
                    .context("Cannot find var's fact")?;

                for var in after_var.into_iter() {
                    edges.push(Edge::Normal {
                        from: before.clone(),
                        to: var.clone(),
                        curved: false,
                    });
                }
            }
            Instruction::Assign(dest, src) => {
                let before = defuse
                    .src_before(ctx, &function, src, pc)
                    .context("Cannot find var's fact")?
                    .into_iter()
                    .map(|x| x.clone())
                    .collect::<Vec<_>>();

                let after_var = defuse
                    .demand(ctx, &function, dest, pc)
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
            _ => {}
        }

        Ok(edges)
    }
}
