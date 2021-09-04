use crate::engine::export::ExportInstance;
use anyhow::{Result};
use wasm_parser::core::*;
use wasm_parser::Module;

/// The module instance contains instance information for a module.
#[derive(Debug, Default, Clone)]
pub struct ModuleInstance {
    /// Contains all functions of a module
    code: Vec<FunctionBody>,
    /// Declares the function signatures of the functions
    fn_types: Vec<FunctionSignature>,
    /// Keeps the indexes of the function
    func_addrs: Vec<FuncAddr>,
    /// Keeps the indexes of the table
    table_addrs: Vec<TableAddr>,
    /// Keeps the indexes of the memories (currently the spec only allows one memory section)
    mem_addrs: Vec<MemoryAddr>,
    /// Keeps the indexes of the global variables
    global_addrs: Vec<GlobalAddr>,
    exports: Vec<ExportInstance>,
}

impl ModuleInstance {
    pub fn new(m: &Module) -> Self {
        let mut mi = ModuleInstance {
            code: Vec::new(),
            fn_types: Vec::new(),
            func_addrs: Vec::new(),
            table_addrs: Vec::new(),
            mem_addrs: Vec::new(),
            global_addrs: Vec::new(),
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
    pub fn lookup_function_addr(&self, idx: &FuncIdx) -> Option<&FuncAddr> {
        self
            .func_addrs
            .get((*idx) as usize)
    }

    pub fn lookup_table_addr(&self, idx: &TableIdx) -> Option<&TableAddr> {
        self
            .table_addrs
            .get(*idx as usize)
    }

    pub fn lookup_memory_addr(&self, idx: &MemoryIdx) -> Option<&MemoryAddr> {
        self
            .mem_addrs
            .get(*idx as usize)
    }

    pub fn lookup_global_addr(&self, idx: &GlobalIdx) -> Option<&GlobalAddr> {
        self
            .global_addrs
            .get(*idx as usize)
    }

    /// Looking up the the func type with given index.
    pub fn lookup_func_types(&self, index: &u32) -> Option<&FunctionSignature> {
        self.fn_types.get(*index as usize)
    }

    /// Looking up the code with given index.
    pub fn lookup_code(&self, index: usize) -> Option<&FunctionBody> {
        self.code.get(index)
    }

    /// Storing a new function addr.
    pub fn store_func_addr(&mut self, new_addr: FuncAddr) -> Result<()> {
        self.func_addrs.push(new_addr);

        Ok(())
    }

    /// Storing a new table addr.
    pub fn store_table_addr(&mut self, new_addr: TableAddr) -> Result<()> {
        self.table_addrs.push(new_addr);

        Ok(())
    }

    /// Storing a new memory addr.
    pub fn store_memory_addr(&mut self, new_addr: MemoryAddr) -> Result<()> {
        self.mem_addrs.push(new_addr);

        Ok(())
    }

    /// Storing a new global addr.
    pub fn store_global_addr(&mut self, new_addr: GlobalAddr) -> Result<()> {
        self.global_addrs.push(new_addr);

        Ok(())
    }

    /// Storing a new export instance.
    pub fn store_export(&mut self, export: ExportInstance) -> Result<()> {
        self.exports.push(export);

        Ok(())
    }

    /// Get an immutable borrow for all memory addresses.
    pub fn get_mem_addrs(&self) -> &Vec<MemoryAddr> {
        &self.mem_addrs
    }

    /// Get the index of the export instance by iterating through all exports and filtering for the given name.
    pub fn position_export_instance_by_name(&self, name: impl Into<String>) -> Option<usize> {
        let k = name.into();
        self.exports.iter().position(|x| x.name == k)
    }

    /// Get an export instance by iterating through all exports and filtering for the given name.
    pub fn get_export_instance_by_name(&self, name: impl Into<String>) -> Option<&ExportInstance> {
        let k = name.into();
        self.exports.iter().find(|x| x.name == k)
    }

    /// Get the export instance by index.
    pub fn get_export_instance(&self, idx: usize) -> Option<&ExportInstance> {
        self.exports.get(idx)
    }

    /// Add a code.
    pub fn add_code(&mut self, body: FunctionBody) -> Result<()> {
        self.code.push(body);

        Ok(())
    }

    /// Get an immutable borrow of the code.
    pub fn get_code(&self) -> &[FunctionBody] {
        &self.code
    }

    pub fn get_fn_types(&self) -> &[FunctionSignature] {
        &self.fn_types
    }

    pub fn get_func_addrs(&self) -> &[FuncAddr] {
        &self.func_addrs
    }

    pub fn get_table_addrs(&self) -> &[TableAddr] {
        &self.table_addrs
    }

    pub fn get_global_addrs(&self) -> &[GlobalAddr] {
        &self.global_addrs
    }

    pub fn get_exports(&self) -> &[ExportInstance] {
        &self.exports
    }
}
