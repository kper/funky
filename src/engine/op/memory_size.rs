use crate::engine::Engine;
use crate::value::Value::I32;
use anyhow::{anyhow, Result};
use crate::PAGE_SIZE;
use crate::engine::StackContent;

impl Engine {
    pub(crate) fn memory_size(&mut self) -> Result<()> {
        let module = &self.module;
        let addr = module
            .memaddrs
            .get(0)
            .ok_or_else(|| anyhow!("No memory address found"))?;
        let instance = &self.store.memory[*addr as usize];

        let sz = instance.data.len() / PAGE_SIZE;

        self.store.stack.push(StackContent::Value(I32(sz as i32)));

        Ok(())
    }
}
