use crate::engine::export::ExportInstance;
use anyhow::{anyhow, Result};
use wasm_parser::core::*;
use wasm_parser::Module;

#[derive(Debug, Clone)]
pub struct ModuleInstance {
    pub start: u32,
    pub code: Vec<FunctionBody>,
    pub fn_types: Vec<FunctionSignature>,
    pub funcaddrs: Vec<FuncAddr>,
    pub tableaddrs: Vec<TableIdx>,
    pub memaddrs: Vec<MemoryIdx>,
    pub globaladdrs: Vec<GlobalAddr>,
    pub exports: Vec<ExportInstance>,
}

impl ModuleInstance {
    pub fn new(m: &Module) -> Self {
        let mut mi = ModuleInstance {
            start: 0,
            code: Vec::new(),
            fn_types: Vec::new(),
            funcaddrs: Vec::new(),
            tableaddrs: Vec::new(),
            memaddrs: Vec::new(),
            globaladdrs: Vec::new(),
            exports: Vec::new(),
        };
        for section in m.sections.iter() {
            match section {
                Section::Code(CodeSection { entries: x }) => {
                    mi.code = x.clone();
                }
                Section::Type(TypeSection { entries: x }) => {
                    mi.fn_types = x.clone();
                }
                _ => {}
            }
        }

        mi
    }

    /// Adding a new function type.
    /// We need this function to test blocks, with multiple
    /// return values.
    pub(crate) fn add_func_type(&mut self, r: Vec<ValueType>) -> usize {
        let instance = FunctionSignature {
            param_types: vec![],
            return_types: r,
        };

        self.fn_types.push(instance);

        self.fn_types.len() - 1
    }

    /// Looking up the function's address in the store by given function's module address.
    pub fn lookup_function_addr(&self, function_module_addr: FuncIdx) -> Result<FuncAddr> {
        debug!(
            "Looking up store's func addr for idx {}",
            function_module_addr
        );

        Ok(self
            .funcaddrs
            .get(function_module_addr as usize)
            .ok_or_else(|| anyhow!("Cannot lookup funcaddr for id {:?}", function_module_addr))?
            .clone())
    }
}
