use crate::engine::stack::{Frame, StackContent};
use crate::engine::Engine;
use anyhow::{bail, Result};
use wasm_parser::core::LocalIdx;

impl Engine {
    pub(crate) fn local_tee(&mut self, idx: &LocalIdx, fr: &mut Frame) -> Result<()> {
        debug!("OP_LOCAL_TEE {:?}", idx);

        let value = match self.store.stack.pop() {
            Some(StackContent::Value(v)) => v,
            Some(_) => {
                bail!("Unexpected stack element at local.tee");
            }
            None => {
                bail!("Empty stack during local.tee");
            }
        };

        self.store.stack.push(StackContent::Value(value));
        self.store.stack.push(StackContent::Value(value));

        self.local_set(idx, fr)?;

        debug!("stack {:?}", self.store.stack);
        debug!("locals {:#?}", fr.locals);

        Ok(())
    }
}
