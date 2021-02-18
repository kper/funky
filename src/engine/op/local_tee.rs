use crate::engine::stack::Frame;
use crate::engine::Engine;
use anyhow::{anyhow, Result};
use wasm_parser::core::LocalIdx;

impl Engine {
    pub(crate) fn local_tee(&mut self, idx: &LocalIdx, fr: &mut Frame) -> Result<()> {
        debug!("OP_LOCAL_TEE {:?}", idx);

        let value = match self.store.stack.pop() {
            Some(v) => v,
            None => {
                return Err(anyhow!("Empty stack during local.tee"));
            }
        };

        self.store.stack.push(value);
        self.store.stack.push(value);

        self.local_set(idx, fr)?;

        debug!("stack {:?}", self.store.stack);
        debug!("locals {:#?}", fr.locals);

        Ok(())
    }
}
