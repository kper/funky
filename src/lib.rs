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

#[allow(dead_code)]
pub(crate) fn empty_engine() -> Engine {
    let mi = ModuleInstance {
        start: 0,
        code: Vec::new(),
        fn_types: Vec::new(),
        funcaddrs: Vec::new(),
        tableaddrs: Vec::new(),
        memaddrs: Vec::new(),
        globaladdrs: Vec::new(),
        exports: Vec::new(),
    };

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
#[macro_export]
macro_rules! construct_engine {
    ($body:expr, $params:expr, $returns:expr) => {{
        use wasm_parser::core::*;
        #[allow(unused_imports)]
        use wasm_parser::core::CtrlInstructions::*;
        #[allow(unused_imports)]
        use wasm_parser::core::Instruction::*;
        #[allow(unused_imports)]
        use wasm_parser::core::MemoryInstructions::*;
        #[allow(unused_imports)]
        use wasm_parser::core::NumericInstructions::*;
        #[allow(unused_imports)]
        use wasm_parser::core::ParamInstructions::*;
        #[allow(unused_imports)]
        use wasm_parser::core::VarInstructions::*;

        let mut e = crate::empty_engine();

        let body = FunctionBody {
            locals: vec![],
            code: $body,
        };

        e.store.funcs = vec![FuncInstance {
            ty: FunctionSignature {
                param_types: $params,
                return_types: $returns,
            },
            code: body.clone(),
        }];


        // Set the code section 
        e.module.code = vec![body.clone()];

        // Export the function
        e.module.funcaddrs.push(0);
        e.module.exports = vec![ExportInstance {
            name: "test".to_string(),
            value: ExternalKindType::Function { ty : 0 }
        }];

        e
    }};
}
