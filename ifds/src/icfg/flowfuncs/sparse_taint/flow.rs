use crate::icfg::flowfuncs::*;
use crate::icfg::state::State;

pub struct SparseTaintNormalFlowFunction;

impl SparseNormalFlowFunction for SparseTaintNormalFlowFunction {
    fn flow(
        &self,
        function: &AstFunction,
        pc: usize,
        variable: &String,
        block_resolver: &BlockResolver,
        state: &mut State,
        defuse: &mut DefUseChain,
    ) -> Result<Vec<Edge>> {
        todo!()
    }
}