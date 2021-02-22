use crate::engine::stack::Frame;
use crate::engine::stack::StackContent;
use crate::engine::Engine;
use anyhow::{Result, Context};
use wasm_parser::core::GlobalIdx;

impl Engine {
    pub(crate) fn global_get(&mut self, idx: &GlobalIdx, _fr: &mut Frame) -> Result<()> {
        self.store.stack.push(StackContent::Value(
            self.store
                .globals
                .get(*idx as usize)
                .context("Cannot access global")?
                .val,
        ));

        debug!("globals {:#?}", self.store.globals);

        Ok(())
    }
}
