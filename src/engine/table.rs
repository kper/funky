use wasm_parser::core::FuncAddr;

#[derive(Debug, Clone)]
pub struct TableInstance {
    pub elem: Vec<Option<FuncAddr>>,
    pub max: Option<u32>,
}
