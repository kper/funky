use crate::engine::{Engine};
use crate::engine::stack::StackContent;
use crate::value::Value;
use anyhow::{bail, Context, Result};
use wasm_parser::core::FuncAddr;

impl Engine {
    pub(crate) fn call_function(&mut self, func_addr: FuncAddr) -> Result<()> {
        debug!("OP_CALL {:?}", func_addr);

        let param_count = &self
            .store
            .get_func_instance(&func_addr)?
            .ty
            .param_types
            .len();

        debug!("=> Function with {:?} found", func_addr);
        debug!("=> Stack is {:#?}", self.store.stack);

        let args = self.extract_args_of_stack(*param_count).with_context(|| {
            format!(
                "Cannot extract args out of stack for function {:?}",
                func_addr
            )
        })?;

        //debug!("=> Resetting stack");
        //let mut stack: Vec<_> = self.store.stack.drain(0..).collect();

        let func_addr_inner = func_addr.get();
        self.invoke_function(func_addr, args)
            .with_context(|| format!("Invoking function {:?} failed", func_addr_inner))?;

        //debug!("=> Restoring stack");
        // Insert `stack` before the values of `self.store.stack`
        //let mut new_stack: Vec<_> = self.store.stack.drain(0..).collect();
        //self.store.stack = stack.drain(0..).collect();
        //self.store.stack.append(&mut new_stack);

        Ok(())
    }

    /// Drops the `param_count` off the stack and returns it
    /// so it can be used as arguments for a web assembly function.
    /// However, we are ignoring labels and frames.
    pub(crate) fn extract_args_of_stack(&mut self, mut param_count: usize) -> Result<Vec<Value>> {
        if param_count == 0 {
            return Ok(vec![]);
        }

        let mut args =  Vec::new();

        while param_count > 0 {
            if let Some(StackContent::Value(val)) = self.store.stack.pop() {
                args.push(val);
            } 
            else {
                bail!("No value left at the stack.");
            }

            param_count -= 1;
        }

        let args  = args.into_iter().rev().collect::<Vec<_>>();

        debug!("=> Stack is {:#?}", self.store.stack);
        debug!("=> args {:#?}", args);

        Ok(args)
    }
}
