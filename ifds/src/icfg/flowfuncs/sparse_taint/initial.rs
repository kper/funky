use crate::icfg::tabulation::sparse::defuse::DefUseChain;
use crate::icfg::{flowfuncs::*, tabulation::sparse::Ctx};

pub struct SparseTaintInitialFlowFunction;

impl SparseInitialFlowFunction for SparseTaintInitialFlowFunction {
    fn flow<'a>(
        &self,
        ctx: &mut Ctx<'a>,
        function: &AstFunction,
        pc: usize,
        init_facts: &Vec<Fact>,
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
                    self.generate_normal_flows(
                        ctx,
                        function,
                        defuse,
                        &mut edges,
                        &mut init_fact,
                        pc,
                        &vec![dest.to_string()],
                    )?;
                }
                Instruction::Call(_callee, _, dests) => {
                    self.generate_normal_flows(
                        ctx,
                        function,
                        defuse,
                        &mut edges,
                        &mut init_fact,
                        pc,
                        &dests,
                    )?;
                }
                Instruction::Store(_src, offset, _) => {
                    let mem = {
                        let mem = ctx
                            .state
                            .add_memory_var(function.name.clone(), offset.clone() as usize);

                        mem.name.clone()
                    };

                    self.generate_normal_flows(
                        ctx,
                        function,
                        defuse,
                        &mut edges,
                        &mut init_fact,
                        pc,
                        &vec![mem],
                    )?;
                }
                Instruction::Return(_dests) => {
                    self.generate_normal_flows(
                        ctx,
                        function,
                        defuse,
                        &mut edges,
                        &mut init_fact,
                        pc,
                        &vec![],
                    )?;
                }
                Instruction::Block(_) | Instruction::Jump(_) => {
                    panic!("Block or Jump as first instruction is not supported");

                    /*log::warn!(
                        "Statement is a block or jump instruction. Therefore skipping to next one."
                    );*/

                    /*let next_facts =
                        defuse.get_facts_at(ctx, function, &init_fact.belongs_to_var, pc)?;

                    init_fact = next_facts
                        .first()
                        .context("Cannot find next initial fact")?
                        .clone()
                        .clone();

                    offset += 1;*/
                    continue;
                }
                _ => {}
            }

            break;
        }

        Ok(edges)
    }
}

impl SparseTaintInitialFlowFunction {
    /// Helper function for generating normal flows for the initial statement.
    /// ## Parameters
    ///  - `init_fact` is the first fact of the procedure.
    ///  - `dests` is a list of Registers which are on lhs side.
    pub(crate) fn generate_normal_flows(
        &self,
        ctx: &mut Ctx,
        function: &AstFunction,
        defuse: &mut DefUseChain,
        edges: &mut Vec<Edge>,
        init_fact: &mut Fact,
        pc: usize,
        dests: &Vec<String>,
    ) -> Result<()> {
        let after_taut = defuse
            .demand(ctx, &function, &"taut".to_string(), init_fact.pc)
            .context("Cannot find taut's fact")?;

        debug!("Next taut is {:#?}", after_taut);

        for taut in after_taut.into_iter() {
            edges.push(Edge::Path {
                from: init_fact.clone().clone(),
                to: taut,
            });
        }

        for dest in dests {
            let after_var = defuse
                .demand_inclusive(ctx, &function, dest, pc)
                .context("Cannot find var's fact")?
                .into_iter()
                .map(|x| x.clone())
                .collect::<Vec<_>>();

            // append all left sides to the nodes
            // %2 = binop %0 %1 -- there %2 is the left side
            let mut appended = Vec::new();
            let mut append_lhs = |dest: &String| -> Result<()> {
                defuse.force_remove_if_outdated(function, dest, pc)?;
                let x = defuse.demand_inclusive(ctx, function, dest, pc)?;
                appended.extend(x.into_iter().map(|x| x.clone()));

                Ok(())
            };

            for var in after_var.clone().iter() {
                append_lhs(&var.belongs_to_var)?;
            }

            for var in after_var.into_iter().chain(appended) {
                let applied = var.apply_bound();

                // after
                edges.push(Edge::Path {
                    from: init_fact.clone().clone(),
                    to: applied.clone(),
                });
            }
        }

        Ok(())
    }
}
