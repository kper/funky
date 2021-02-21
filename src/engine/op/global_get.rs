use crate::engine::stack::Frame;
use crate::engine::Engine;
use anyhow::{Result};
use wasm_parser::core::GlobalIdx;
use crate::engine::stack::StackContent;

impl Engine {
    pub(crate) fn global_get(&mut self, idx: &GlobalIdx, _fr: &mut Frame) -> Result<()> {
        self.store
            .stack
            .push(StackContent::Value(self.store.globals[*idx as usize].val));

        debug!("globals {:#?}", self.store.globals);

        Ok(())
    }
}
