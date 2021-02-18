use crate::engine::stack::CtrlStackContent;
use crate::engine::Engine;
use crate::fetch_unop;
use crate::value::Value::I32;
use anyhow::{anyhow, Context, Result};
use wasm_parser::core::FuncAddr;

impl Engine {
    pub(crate) fn call_indirect_function(&mut self, function_addr: &FuncAddr) -> Result<()> {
        debug!("OP_CALL_INDIRECT {:?}", function_addr);

        debug!("before ta");
        let ta = self
            .module
            .tableaddrs
            .get(0)
            .context("Cannot find first table addr")?;

        debug!("before tab");

        let tab = &self
            .store
            .tables
            .get(*ta as usize)
            .with_context(|| format!("Cannot access {:?}", ta))?;

        debug!("before i");

        let i = match fetch_unop!(self.store.stack) {
            I32(x) => x,
            x => return Err(anyhow!("invalid index type: {:?}", x)),
        };
        if (i as usize) >= tab.elem.len() {
            return Err(anyhow!(
                "Attempt to perform indirect call to index larger than the table"
            ));
        }

        debug!("after i");

        let indirected_func_addr = tab
            .elem
            .get(i as usize)
            .ok_or_else(|| anyhow!("Cannot access elem at {:?}", i))?
            .as_ref()
            .ok_or_else(|| anyhow!("Accessed element is not defined"))?
            .clone();

        debug!("ii i");

        let func_instance = &self.store.get_func_instance(&indirected_func_addr)?.ty;

        let param_count = func_instance.param_types.len();

        debug!(
            "Indirecting to {:?} with params {}",
            indirected_func_addr, param_count
        );

        let args = &self.extract_args_of_stack(param_count).with_context(|| {
            format!(
                "Cannot extract args out of stack for function {:?}",
                indirected_func_addr
            )
        })?;

        debug!("=> Invoking {:?} with {:?}", indirected_func_addr, args);

        self.invoke_function(&indirected_func_addr, args.to_vec())?;

        Ok(())
    }
}
