use crate::icfg::tabulation::sparse::defuse::DefUseChain;
use crate::icfg::{flowfuncs::*, tabulation::sparse::Ctx};

pub struct SparseTaintNormalFlowFunction;

impl SparseNormalFlowFunction for SparseTaintNormalFlowFunction {
    fn flow<'a>(
        &self,
        ctx: &mut Ctx<'a>,
        function: &AstFunction,
        pc: usize,
        variable: &String,
        defuse: &mut DefUseChain,
    ) -> Result<Vec<Fact>> {
        debug!(
            "Calling flow for {} with var {} with pc {}",
            function.name, variable, pc
        );

        let mut facts = Vec::new();

        let instructions = &function.instructions;
        let instruction = instructions.get(pc);

        if instruction.is_none() {
            debug!("Instruction is none");
            return Ok(Vec::new());
        }

        let instruction = instruction.unwrap();

        debug!("Next instruction is {:?} for {}", instruction, variable);

        let mut nodes = defuse
            .demand_inclusive(ctx, function, variable, pc)?
            .into_iter()
            .map(|x| x.clone())
            .collect::<Vec<_>>();

        // append all left sides to the nodes
        // %2 = binop %0 %1 -- there %2 is the left side
        let mut append_lhs = |dest: &String| -> Result<()> {
            defuse.force_remove_if_outdated(function, dest, pc)?;
            let x = defuse.demand_inclusive(ctx, function, dest, pc)?;

            nodes.extend(x.into_iter().map(|x| x.clone()));

            Ok(())
        };

        match instruction {
            Instruction::Unop(dest, ..)
            | Instruction::Phi(dest, ..)
            | Instruction::BinOp(dest, ..)
            | Instruction::Assign(dest, ..) => append_lhs(dest)?,
            Instruction::Load(dest, ..) => append_lhs(dest)?,
            Instruction::Call(_, _, dests) => {
                for dest in dests {
                    append_lhs(dest)?;
                }
            }
            Instruction::Store(_src, offset, _i) => {
                let y = ctx
                    .state
                    .add_memory_var(function.name.clone(), *offset as usize);

                let x = defuse.demand_inclusive(ctx, function, &y.name, pc)?;
                nodes.extend(x.into_iter().map(|x| x.clone()));
            }
            _ => {}
        }
        let nodes = nodes
            .into_iter()
            .filter_map(|x| {
                if x.pc != x.next_pc {
                    Some(x.apply_bound())
                } else {
                    Some(x)
                }
            })
            .collect::<Vec<_>>();

        facts.extend(nodes.into_iter());

        debug!("Out {:#?}", facts);

        Ok(facts)
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::icfg::state::State;
    use crate::ir::ast::Program;
    use pretty_assertions::assert_eq;

    #[test]
    fn test_binop() {
        /*
            %0 = 1
            %1 = 1
            %2 = %0 op %1
            %3 = %1 op %0
        */

        let func_name = "main".to_string();
        let function = AstFunction {
            name: func_name.clone(),
            definitions: vec![
                "%0".to_string(),
                "%1".to_string(),
                "%2".to_string(),
                "%3".to_string(),
            ],
            instructions: vec![
                Instruction::Const("%0".to_string(), 1.0),
                Instruction::Const("%1".to_string(), 1.0),
                Instruction::BinOp("%2".to_string(), "%0".to_string(), "%1".to_string()),
                Instruction::BinOp("%3".to_string(), "%0".to_string(), "%1".to_string()),
            ],
            ..Default::default()
        };

        let mut graph = Graph::default();
        let mut state = State::default();

        let mut ctx = Ctx {
            graph: &mut graph,
            state: &mut state,
            prog: &Program {
                functions: vec![function.clone()],
            },
            block_resolver: HashMap::default(),
        };

        let pc = 0;

        // fullfilling precondition of `chain.cache()`
        ctx.state.init_function(&function, pc).unwrap();

        let mut defuse = DefUseChain::default();

        let sparse = SparseTaintNormalFlowFunction;

        let mut facts = sparse
            .flow(&mut ctx, &function, 2, &"%0".to_string(), &mut defuse)
            .unwrap();

        facts.dedup();

        debug!("facts {:#?}", facts);
        assert_eq!(4, facts.len());

        let cmp_facts: Vec<Fact> = vec![
            Fact {
                belongs_to_var: "%0".to_string(),
                var_is_global: false,
                var_is_taut: false,
                var_is_memory: false,
                pc: 3,
                next_pc: 3,
                track: 1,
                function: "main".to_string(),
                memory_offset: None,
            },
            Fact {
                belongs_to_var: "%0".to_string(),
                var_is_global: false,
                var_is_taut: false,
                var_is_memory: false,
                pc: 4,
                next_pc: 4,
                track: 1,
                function: "main".to_string(),
                memory_offset: None,
            },
            Fact {
                belongs_to_var: "%2".to_string(),
                var_is_global: false,
                var_is_taut: false,
                var_is_memory: false,
                pc: 3,
                next_pc: 4,
                track: 3,
                function: "main".to_string(),
                memory_offset: None,
            },
            Fact {
                belongs_to_var: "%2".to_string(),
                var_is_global: false,
                var_is_taut: false,
                var_is_memory: false,
                pc: 4,
                next_pc: 4,
                track: 3,
                function: "main".to_string(),
                memory_offset: None,
            },
        ];
        assert_eq!(cmp_facts, facts);
    }
}
