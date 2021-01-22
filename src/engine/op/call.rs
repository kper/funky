use crate::engine::map_stackcontent_to_value;
use crate::engine::{Engine, StackContent};
use crate::value::Value;
use anyhow::{anyhow, Context, Result};
use wasm_parser::core::FuncAddr;

impl Engine {
    pub(crate) fn call_function(&mut self, func_addr: &FuncAddr) -> Result<()> {
        debug!("OP_CALL {:?}", func_addr);

        let param_count = &self
            .store
            .get_func_instance(func_addr)?
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

        self.invoke_function(func_addr, args)
            .with_context(|| format!("Invoking function {:?} failed", func_addr))?;

        /*
        debug!("=> Restoring stack");
        // Insert `stack` before the values of `self.store.stack`
        let mut new_stack: Vec<_> = self.store.stack.drain(0..).collect();
        self.store.stack = stack.drain(0..).collect();
        self.store.stack.append(&mut new_stack);*/

        Ok(())
    }

    /// Drops the `param_count` off the stack and returns it
    /// so it can be used as arguments for a web assembly function.
    /// However, we are ignoring labels and frames.
    pub(crate) fn extract_args_of_stack(&mut self, param_count: usize) -> Result<Vec<Value>> {
        let mut count = 0;
        let mut value_count = 0;

        if param_count == 0 {
            return Ok(vec![]);
        }

        // Count until we counted at least `function_ty.param_types.len()` values of the stack
        for element in self.store.stack.iter().rev() {
            debug!("Element is {:?}", element);
            count += 1;

            if !matches!(element, StackContent::Value(_)) {
                // If not value, then go to next
                continue;
            } else {
                value_count += 1;
            }

            debug!(
                "value_count {} >= params {} then break",
                value_count, param_count
            );
            if value_count >= param_count {
                break;
            }
        }

        debug!("=> count is {}", count);

        let new_args = self
            .store
            .stack
            .split_off(self.store.stack.len() - count)
            .into_iter();

        let mut non_values = new_args
            .clone()
            .filter(|w| !matches!(w, StackContent::Value(_)))
            .collect();
        // Append labels and frames back
        self.store.stack.append(&mut non_values);

        let args: Vec<_> = new_args
            .filter(|w| matches!(w, StackContent::Value(_)))
            .map(map_stackcontent_to_value)
            .collect::<Result<_>>()
            .context("Cannot map StackContent to Value")?;

        debug!("=> Stack is {:#?}", self.store.stack);
        debug!("=> args {:#?}", args);

        Ok(args)
    }
}
