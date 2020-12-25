use wasm_parser::core::{FunctionBody, FunctionSignature};

#[derive(Debug, Clone)]
pub struct FuncInstance {
    //FIXME Add HostFunc
    pub ty: FunctionSignature,
    pub code: FunctionBody,
}
