use crate::engine::map_stackcontent_to_value;
use crate::engine::Engine;
use anyhow::{anyhow, Context, Result};
use wasm_parser::core::FuncIdx;

impl Engine {
    pub(crate) fn call_function(&mut self, idx: &FuncIdx) -> Result<()> {
        debug!("OP_CALL {:?}", idx);

        //let function_ty = self.store.funcs[*idx as usize].ty.clone();
        let function_ty = &self
            .store
            .funcs
            .get(*idx as usize)
            .ok_or_else(|| anyhow!("Cannot access function with addr {}", idx))?
            .ty;

        debug!("=> Function with addr {} found", idx);
        debug!("=> Stack is {:#?}", self.store.stack);
        debug!("=> Function ty is {:#?}", function_ty);

        let args = self
            .store
            .stack
            .split_off(self.store.stack.len() - function_ty.param_types.len())
            .into_iter()
            .map(map_stackcontent_to_value)
            .collect::<Result<_>>()
            .with_context(|| {
                format!("Cannot map StackContent to Value for function addr {}", idx)
            })?;

        self.invoke_function(*idx, args)
            .with_context(|| format!("Invoking function addr {} failed", idx))?;

        Ok(())
    }
}
