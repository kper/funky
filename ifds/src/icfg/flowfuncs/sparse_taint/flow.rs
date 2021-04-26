use crate::icfg::state::State;
use crate::icfg::{flowfuncs::*, tabulation::sparse::Ctx};

pub struct SparseTaintNormalFlowFunction;

impl SparseTaintNormalFlowFunction {
    pub fn identity<'a>(
        &self,
        ctx: &mut Ctx<'a>,
        function: &AstFunction,
        fact: &Fact,
        pc: usize,
        defuse: &mut DefUseChain,
        edges: &mut Vec<Edge>,
    ) -> Result<()> {
        let after = defuse.get_next(ctx, function, &fact.belongs_to_var, pc)?;

        for to in after {
            edges.push(Edge::Normal {
                from: fact.apply(),
                to: to,
                curved: false,
            });
        }

        Ok(())
    }
}

impl SparseNormalFlowFunction for SparseTaintNormalFlowFunction {
    fn flow<'a>(
        &self,
        ctx: &mut Ctx<'a>,
        function: &AstFunction,
        pc: usize,
        variable: &String,
        block_resolver: &BlockResolver,
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
        let is_taut = variable == &"taut".to_string();

        let nodes = defuse.demand(ctx, function, variable, pc)?;

        // Apply here function instead

        facts.extend(nodes.into_iter());

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
        };

        let pc = 0;

        // fullfilling precondition of `chain.cache()`
        ctx.state.init_function(&function, pc).unwrap();

        let mut defuse = DefUseChain::default();

        let block_resolver = BlockResolver::default();
        let sparse = SparseTaintNormalFlowFunction;

        let edges = sparse
            .flow(
                &mut ctx,
                &function,
                2,
                &"%0".to_string(),
                &block_resolver,
                &mut defuse,
            )
            .unwrap();

        assert_eq!(2, edges.len());

        let cmp_facts: Vec<Fact> = vec![
            Fact {
                belongs_to_var: "%0".to_string(),
                var_is_global: false,
                var_is_taut: false,
                var_is_memory: false,
                pc: 3,
                next_pc: 4,
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
        ];
        assert_eq!(cmp_facts, edges);
    }
}
