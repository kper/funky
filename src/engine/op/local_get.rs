use crate::engine::stack::{Frame, StackContent};
use crate::engine::Engine;
use anyhow::{bail, Result};
use wasm_parser::core::LocalIdx;

impl Engine {
    pub(crate) fn local_get(&mut self, idx: &LocalIdx, fr: &mut Frame) -> Result<()> {
        debug!("LOCAL_GET at {} is {:?}", idx, fr.locals[*idx as usize]);
        debug!("locals {:#?}", fr.locals);

        if let Some(val) = fr.locals.get(*idx as usize) {
            debug!("Pushing {:?} to the stack", val);
            self.store.stack.push(StackContent::Value(*val));

            debug!("stack {:#?}", self.store.stack);

            Ok(())
        } else {
            bail!(
                "Trying to access locals ({}), but out of bounds (length {})",
                idx,
                fr.locals.len()
            )
        }
    }
}
