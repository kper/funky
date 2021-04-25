use crate::icfg::flowfuncs::*;
use anyhow::bail;

pub struct SparseTaintInitialFlowFunction;

impl SparseInitialFlowFunction for SparseTaintInitialFlowFunction {
    fn flow(
        &self,
        function: &AstFunction,
        pc: usize,
        init_facts: &Vec<Fact>,
        normal_flows_debug: &mut Vec<Edge>,
        state: &mut State,
        defuse: &mut DefUseChain,
    ) -> Result<Vec<Edge>> {
        todo!()
    }
}