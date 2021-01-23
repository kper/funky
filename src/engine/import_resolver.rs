use crate::engine::{GlobalInstance, TableInstance};
use anyhow::{anyhow, Result};
use std::collections::HashMap;
//use wasm_parser::core::ImportEntry;

pub type Imports = Vec<Import>;

type Module = String;
type Name = String;

//TODO add more types
#[derive(Debug)]
pub enum Import {
    Global(Module, Name, GlobalInstance),
    Table(Module, Name, TableInstance),
}

/// Private lookup table for
/// modules and names
#[derive(Debug, Default)]
struct LookupTable {
    modules: HashMap<(Module, Name), Import>,
}

impl LookupTable {
    pub fn lookup(&self, module: impl Into<String>, name: impl Into<String>) -> Option<&Import> {
        if let Some(m) = self.modules.get(&(module.into(), name.into())) {
            return Some(m);
        }

        None
    }
}

/// The `ImportResolver` matches the `wasm_parser::core::ImportDesc`
/// with the actual given imports.
#[derive(Debug)]
pub(crate) struct ImportResolver {
    imports: LookupTable,
}

impl ImportResolver {
    pub fn new() -> Self {
        Self {
            imports: LookupTable::default(),
        }
    }

    /// Get the imported global by module and name
    pub fn resolve_global(&self, module: &String, name: &String) -> Result<GlobalInstance> {
        debug!("resolve global {} {}", module, name);

        if let Some(Import::Global(_, _, instance)) = self.imports.lookup(module, name) {
            return Ok(instance.clone());
        }

        Err(anyhow!("Cannot find global for {} {}", module, name))
    }

    /// Get the imported table by module and name
    pub fn resolve_table(&self, module: &String, name: &String) -> Result<TableInstance> {
        debug!("resolve table {} {}", module, name);

        if let Some(Import::Table(_, _, instance)) = self.imports.lookup(module, name) {
            return Ok(instance.clone());
        }

        Err(anyhow!("Cannot find table for {} {}", module, name))
    }

    pub fn inject_global(
        &mut self,
        module: Module,
        name: Name,
        instance: &GlobalInstance,
    ) -> Result<()> {
        self.imports.modules.insert(
            (module.clone(), name.clone()),
            Import::Global(module, name, instance.clone()),
        );

        Ok(())
    }

    pub fn inject_table(
        &mut self,
        module: Module,
        name: Name,
        instance: &TableInstance,
    ) -> Result<()> {
        self.imports.modules.insert(
            (module.clone(), name.clone()),
            Import::Table(module, name, instance.clone()),
        );

        Ok(())
    }
}
