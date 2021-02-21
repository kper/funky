use crate::engine::stack::{Frame, StackContent};
use crate::engine::Engine;
use anyhow::Result;
use wasm_parser::core::LocalIdx;

impl Engine {
    pub(crate) fn local_set(&mut self, idx: &LocalIdx, fr: &mut Frame) -> Result<()> {
        debug!("OP_LOCAL_SET {:?}", idx);
        debug!("locals {:#?}", fr.locals);

        match self.store.stack.pop() {
            Some(StackContent::Value(v)) => {
                match fr.locals.get_mut(*idx as usize) {
                    Some(k) => *k = v, //Exists replace
                    None => {
                        //Does not exists; push
                        fr.locals.push(v)
                    }
                }
            }
            Some(x) => panic!("Expected value but found {:?}", x),
            None => panic!("Empty stack during local.set"),
        }

        Ok(())
    }
}
