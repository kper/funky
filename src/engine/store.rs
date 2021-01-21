use crate::engine::memory::MemoryInstance;
use crate::engine::stack::StackContent;

use crate::engine::stack::Frame;
use crate::engine::Variable;
use crate::engine::{FuncInstance, TableInstance};
use wasm_parser::core::{FuncAddr, FunctionBody, FunctionSignature};

use crate::PAGE_SIZE;
use anyhow::{anyhow, Result};

#[derive(Debug, Default)]
pub struct Store {
    funcs: Vec<FuncInstance>,
    pub tables: Vec<TableInstance>,
    pub memory: Vec<MemoryInstance>,
    pub stack: Vec<StackContent>,
    pub globals: Vec<Variable>, //=GlobalInstance
}

impl Store {
    /// Create an empty store with a frame
    pub(crate) fn default_with_frame() -> Self {
        let mut store = Store::default();

        store.stack.push(StackContent::Frame(Frame {
            arity: 0,
            locals: Vec::new(),
        }));

        store
    }

    /// Initializes `n` pages in memory with [0; n * PAGE_SIZE]
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

    /// Get the function instance by address
    pub(crate) fn get_func_instance(&self, func_addr: &FuncAddr) -> Result<&FuncInstance> {
        debug!("Get function's instance by addr {:?}", func_addr);

        self.funcs
            .get(func_addr.get())
            .ok_or_else(|| anyhow!("Cannot find function by addr {:?}", func_addr))
    }

    pub(crate) fn allocate_func_instance(
        &mut self,
        signature: FunctionSignature,
        code: FunctionBody,
    ) -> Result<()> {
        debug!("Allocation function {:?}", signature);

        let instance = FuncInstance {
            ty: signature,
            code,
        };

        self.funcs.push(instance);

        Ok(())
    }

    pub(crate) fn count_functions(&self) -> usize {
        self.funcs.len()
    }
}
