use wasm_parser::core::FuncIdx;
use anyhow::Result;
use crate::engine::Engine;
use crate::engine::map_stackcontent_to_value;

impl Engine {
    pub(crate) fn call_function(&mut self, idx: &FuncIdx) -> Result<()> {
        debug!("OP_CALL {:?}", idx);

        trace!("fn_types: {:#?}", self.module.fn_types);
        let t = self.store.funcs[*idx as usize].ty.clone();

        let args = self
            .store
            .stack
            .split_off(self.store.stack.len() - t.param_types.len())
            .into_iter()
            .map(map_stackcontent_to_value)
            .collect::<Result<_>>()?;

        self.invoke_function(*idx, args)?;

        Ok(())
    }
}
