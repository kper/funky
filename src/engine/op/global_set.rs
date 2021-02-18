use crate::engine::stack::Frame;
use crate::engine::Engine;
use anyhow::{anyhow, Result};
use wasm_parser::core::GlobalIdx;

impl Engine {
    pub(crate) fn global_set(&mut self, idx: &GlobalIdx, _fr: &mut Frame) -> Result<()> {
        match self.store.stack.pop() {
            Some(v) => {
                if !self.store.globals[*idx as usize].mutable {
                    return Err(anyhow!("Attempting to modify a immutable global"));
                }
                self.store.globals[*idx as usize].val = v;
                debug!("globals {:#?}", self.store.globals);

                Ok(())
            }
            Some(x) => {
                Err(anyhow!("Expected value but found {:?}", x))
            }
            None => {
                Err(anyhow!("Empty stack during local.tee"))
            }
        }
    }
}
