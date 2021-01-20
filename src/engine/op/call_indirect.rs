use crate::engine::map_stackcontent_to_value;
use crate::engine::stack::StackContent::Value;
use crate::engine::Engine;
use crate::fetch_unop;
use crate::value::Value::I32;
use anyhow::{anyhow, Context, Result};
use wasm_parser::core::FuncIdx;

impl Engine {
    pub(crate) fn call_indirect_function(&mut self, idx: &FuncIdx) -> Result<()> {
        debug!("OP_CALL_INDIRECT {:?}", idx);
        let ta = self.module.tableaddrs[0];
        let tab = &self.store.tables[ta as usize];

        let i = match fetch_unop!(self.store.stack) {
            I32(x) => x,
            x => return Err(anyhow!("invalid index type: {:?}", x)),
        };
        if (i as usize) >= tab.elem.len() {
            return Err(anyhow!(
                "Attempt to perform indirect call to index larger than the table"
            ));
        }
        trace!("Table: {:?}", tab.elem);

        match tab.elem[i as usize] {
            Some(a) => {
                let func_instance = &self
                    .store
                    .funcs
                    .get(*idx as usize)
                    .ok_or_else(|| anyhow!("Cannot access function with addr {}", idx))?
                    .ty;

                let param_count = func_instance.param_types.len();

                {
                    // Compare types
                    let m = &self.module;
                    let ty = m.fn_types.get(*idx as usize);
                    assert!(func_instance == ty.expect("No type found"));
                }

                let args = self
                    .extract_args_of_stack(param_count)
                    .with_context(|| {
                        format!("Cannot extract args out of stack for function addr {}", idx)
                    })?;

                self.invoke_function(a as u32, args)?;
            }
            None => panic!("Table not initialized at index {}", i),
        }

        Ok(())
    }
}
