use wasm_parser::core::FuncAddr;

#[derive(Debug, Clone)]
pub struct TableInstance {
    pub elem: Vec<Option<FuncAddr>>,
    pub max: Option<u32>,
}

impl TableInstance {
    pub fn new(n: u32, max: Option<u32>) -> Self {
        Self {
            elem: vec![None; n as usize],
            max: max,
        }
    }
}
