use crate::engine::memory::MemoryInstance;
use crate::engine::stack::StackContent;

use crate::engine::Variable;
use crate::engine::{FuncInstance, TableInstance};

use crate::PAGE_SIZE;
use anyhow::{anyhow, Result};

#[derive(Debug)]
pub struct Store {
    pub funcs: Vec<FuncInstance>,
    pub tables: Vec<TableInstance>,
    pub memory: Vec<MemoryInstance>,
    pub stack: Vec<StackContent>,
    pub globals: Vec<Variable>, //=GlobalInstance
}

impl Store {
    /// Initializes `n` pages in memory with 0
    pub(crate) fn init_memory(&mut self, n: usize) -> Result<()> {
        if self.memory.last().is_none() {
            let instance = MemoryInstance {
                max: None,
                data: vec![0u8; n * PAGE_SIZE],
            };

            self.memory.push(instance);
            Ok(())
        } else {
            Err(anyhow!("A memory instance is already defined"))
        }
    }
}
