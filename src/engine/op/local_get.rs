use crate::engine::Engine;
use crate::engine::stack::Frame;
use anyhow::{anyhow, Result};
use wasm_parser::core::LocalIdx;
use crate::engine::stack::StackContent::Value;

impl Engine {
    pub(crate) fn local_get(&mut self, idx: &LocalIdx, fr: &mut Frame) -> Result<()> {
        if let Some(val) = fr.locals.get(*idx as usize) {
            self.store.stack.push(Value(*val));
            debug!("LOCAL_GET at {} is {:?}", idx, fr.locals[*idx as usize]);
            debug!("locals {:#?}", fr.locals);

            Ok(())
        } else {
            Err(anyhow!(
                "Trying to access locals ({}), but out of bounds (length {})",
                idx,
                fr.locals.len()
            ))
        }
    }
}
