use crate::engine::Engine;
use crate::fetch_binop;
use crate::value::Value::I32;
use anyhow::{anyhow, Result};

impl Engine {
    pub(crate) fn select(&mut self) -> Result<()> {
        debug!("OP_SELECT");
        debug!("Popping {:?}", self.store.stack.last());
        let c = match self.store.stack.pop() {
            Some(I32(x)) => x,
            _ => return Err(anyhow!("Expected I32 on top of stack")),
        };
        let (v1, v2) = fetch_binop!(self.store.stack);
        if c != 0 {
            debug!("C is not 0 therefore, pushing {:?}", v2);
            self.store.stack.push(v2)
        } else {
            debug!("C is not 0 therefore, pushing {:?}", v1);
            self.store.stack.push(v1)
        }

        Ok(())
    }
}
