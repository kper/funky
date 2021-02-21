use crate::engine::memory::MemoryInstance;
use crate::engine::stack::StackContent;

use crate::engine::stack::Frame;
use crate::engine::Variable;
use crate::engine::{FuncInstance, TableInstance};
use wasm_parser::core::{FuncAddr, GlobalAddr, FunctionBody, FunctionSignature};

use crate::PAGE_SIZE;
use anyhow::{anyhow, Result, bail};

pub type GlobalInstance = Variable;

#[derive(Debug, Default)]
pub struct Store {
    pub funcs: Vec<FuncInstance>,
    pub tables: Vec<TableInstance>,
    pub memory: Vec<MemoryInstance>,
    pub stack: Vec<StackContent>,
    pub globals: Vec<GlobalInstance>,
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

    /// Get the global instance by address
    pub(crate) fn get_global_instance(&self, global_addr: &GlobalAddr) -> Result<&GlobalInstance> {
        debug!("Get global's instance by addr {:?}", global_addr);

        self.globals
            .get(global_addr.get())
            .ok_or_else(|| anyhow!("Cannot find global by addr {:?}", global_addr))
    }

    pub(crate) fn get_func_instances(&self) -> &[FuncInstance] {
        debug!("Getting func instances");

        &self.funcs
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

    /// Pop off `n` elements of the stack and return it.
    pub fn pop_off_stack(&mut self, mut n: usize) -> Result<Vec<StackContent>> {
        debug!("pop_off_stack; stack {}; n {}", self.stack.len(), n);
        let mut result = Vec::with_capacity(n);

        while n > 0 {
            if let Some(val) = self.stack.pop() {
                debug!("Popping off {:?}", val);
                result.push(val);
                n -= 1;
            }
            else {
                bail!("The stack was unexpectedly empty while popping {} elements of the stack", n);
            }
        }

        Ok(result.into_iter().rev().collect())
    }
}