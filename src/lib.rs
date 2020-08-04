#[macro_use]
extern crate log;
extern crate wasm_parser;
pub mod allocation;
pub mod engine;
pub mod instantiation;

pub use validation::validate;
pub use wasm_parser::parse;
pub use wasm_parser::read_wasm;

#[cfg(test)]
mod tests;

use engine::*;
use std::cell::RefCell;
use std::rc::Rc;

#[allow(dead_code)]
pub(crate) fn empty_engine() -> Engine {
    let mi = Rc::new(RefCell::new(ModuleInstance {
        start: 0,
        code: Vec::new(),
        fn_types: Vec::new(),
        funcaddrs: Vec::new(),
        tableaddrs: Vec::new(),
        memaddrs: Vec::new(),
        globaladdrs: Vec::new(),
        exports: Vec::new(),
    }));
    Engine {
        started: true,
        store: Store {
            funcs: Vec::new(),
            tables: Vec::new(),
            globals: Vec::new(),
            memory: Vec::new(),
            stack: vec![StackContent::Frame(Frame {
                arity: 0,
                locals: Vec::new(),
            })],
        },
        module: mi,
    }
}

#[allow(unused_macros)]
macro_rules! construct_engine {
    ($body:expr, $params:expr, $return:expr) => {{
        let mut e = empty_engine();

        let body = FunctionBody {
            locals: vec![],
            code: $body,
        };

        // We have 2 parameters, but supply 3
        e.store.funcs = vec![FuncInstance {
            ty: FunctionSignature {
                param_types: $params,
                return_types: $returns,
            },
            code: body.clone(),
        }];

        e.module.borrow_mut().code = vec![body.clone()];

        e
    }}
}
