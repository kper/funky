use crate::engine::map_stackcontent_to_value;
use crate::engine::Engine;
use anyhow::{anyhow, Result};
use crate::value::Value::I32;
use wasm_parser::core::FuncIdx;
use crate::fetch_unop;
use crate::engine::stack::StackContent::Value;


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
                let f = self
                    .store
                    .funcs
                    .get(a as usize)
                    .expect("No function in store");

                {
                    // Compare types
                    let m = &self.module;
                    let ty = m.fn_types.get(*idx as usize);
                    assert!(&f.ty == ty.expect("No type found"));
                }

                let args = self
                    .store
                    .stack
                    .split_off(self.store.stack.len() - f.ty.param_types.len())
                    .into_iter()
                    .map(map_stackcontent_to_value)
                    .collect::<Result<_>>()?;

                self.invoke_function(a as u32, args)?;
            }
            None => panic!("Table not initilized at index {}", i),
        }

        Ok(())
    }
}
