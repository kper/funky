use wasm_parser::core::{ExternalKindType, ExportEntry};

#[derive(Debug, Clone, PartialEq)]
pub struct ExportInstance {
    pub name: String,
    pub value: ExternalKindType,
}

impl Into<ExportInstance> for &ExportEntry {
    fn into(self) -> ExportInstance {
        ExportInstance {
            name: self.name.clone(),
            value: self.kind,
        }
    }
}
