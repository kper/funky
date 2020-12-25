use crate::engine::stack::Frame;
use crate::engine::stack::StackContent::Value;
use crate::engine::Engine;
use anyhow::{anyhow, Result};
use wasm_parser::core::GlobalIdx;

impl Engine {
    pub(crate) fn global_set(&mut self, idx: &GlobalIdx, fr: &mut Frame) -> Result<()> {
        match self.store.stack.pop() {
            Some(Value(v)) => {
                if !self.store.globals[*idx as usize].mutable {
                    return Err(anyhow!("Attempting to modify a immutable global"));
                }
                self.store.globals[*idx as usize].val = v;
                debug!("globals {:#?}", self.store.globals);

                Ok(())
            }
            Some(x) => {
                return Err(anyhow!("Expected value but found {:?}", x));
            }
            None => {
                return Err(anyhow!("Empty stack during local.tee"));
            }
        }
    }
}
