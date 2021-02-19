use crate::engine::Engine;
use crate::engine::memory::grow_memory;
use crate::value::Value::I32;
use crate::engine::stack::StackContent;
use crate::PAGE_SIZE;
use anyhow::{anyhow, Result};
use crate::engine::Page;

impl Engine {
    pub(crate) fn memory_grow(&mut self) -> Result<()> {
        let module = &self.module;
        let addr = module
            .memaddrs
            .get(0)
            .ok_or_else(|| anyhow!("No memory address found"))?;
        let instance = &mut self.store.memory[*addr as usize];
        let _sz = instance.data.len() / PAGE_SIZE;

        if let Some(StackContent::Value(I32(n))) = self.store.stack.pop() {
            if n < 0 {
                return Err(anyhow!("Memory grow expected n > 0, got {}", n));
            }

            match grow_memory(instance, Page::new(n as usize)) {
                Err(()) => {
                    error!("Memory growing failed because paging failed.");
                    self.store.stack.push(StackContent::Value(I32(-1)));
                }
                Ok(_new_sz) => {
                    //debug!("Old memory size {} pages", _new_sz);
                    self.store.stack.push(StackContent::Value(I32(_sz as i32)));
                }
            }
        } else {
            return Err(anyhow!("Unexpected stack element. Expected I32"));
        }

        Ok(())
    }
}
