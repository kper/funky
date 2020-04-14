use wasm_parser::core::*;
use wasm_parser::Module;

type IResult<T> = Result<T, &'static str>;

pub mod instructions;

// Leading question: Should validation return errors or panic?

use log::debug;

macro_rules! matches(
    ($e:expr, $p:pat) => (
        match $e {
            $p => true,
            _ => false
        }
    )
);

pub fn validate(module: &Module) -> IResult<()> {
    //https://webassembly.github.io/spec/core/valid/modules.html#valid-module

    // For each functypei in module.types, the function type functypei must be valid.

    debug!("Function types are valid"); //always valid

    // For each funci in module.funcs, the definition funci must be valid with a function type fti.

    debug!("TODO check functions");

    // For each tablei in module.tables, the definition tablei must be valid with a table type tti.

    let table_types: Vec<_> = module
        .sections
        .iter()
        .filter_map(|ref w| match w {
            Section::Table(t) => Some(&t.entries),
            _ => None,
        })
        .flatten()
        .collect();

    for ty in table_types {
        assert!(check_table_ty(ty));
    }

    debug!("Table types are valid");

    // For each memi in module.mems, the definition memi must be valid with a memory type mti.

    let mem_types: Vec<_> = module
        .sections
        .iter()
        .filter_map(|ref w| match w {
            Section::Memory(t) => Some(&t.entries),
            _ => None,
        })
        .flatten()
        .collect();

    for ty in mem_types {
        assert!(check_memory_ty(ty));
    }

    debug!("Memory types are valid");

    // For each globali in module.globals : ...

    //TODO

    debug!("Global types are valid");

    let elem_types: Vec<_> = module
        .sections
        .iter()
        .filter_map(|ref w| match w {
            Section::Element(t) => Some(&t.entries),
            _ => None,
        })
        .flatten()
        .collect();

    for ty in elem_types {
        assert!(check_elem_ty(ty));
    }

    debug!("Element types are valid");

    let data_types: Vec<_> = module
        .sections
        .iter()
        .filter_map(|ref w| match w {
            Section::Data(t) => Some(&t.entries),
            _ => None,
        })
        .flatten()
        .collect();

    for ty in data_types {
        assert!(check_data_ty(ty));
    }

    debug!("Data types are valid");

    let import_types: Vec<_> = module
        .sections
        .iter()
        .filter_map(|ref w| match w {
            Section::Import(t) => Some(&t.entries),
            _ => None,
        })
        .flatten()
        .collect();

    for ty in import_types {
        assert!(check_import_ty(ty));
    }

    debug!("Import types are valid");

    let export_types: Vec<_> = module
        .sections
        .iter()
        .filter_map(|ref w| match w {
            Section::Export(t) => Some(&t.entries),
            _ => None,
        })
        .flatten()
        .collect();

    for ty in export_types {
        assert!(check_export_ty(ty));
    }

    debug!("Export types are valid");

    // The length of C.tables must not be larger than 1.

    let tables_sections = module
        .sections
        .iter()
        .filter_map(|ref w| match w {
            Section::Table(ref t) => Some(t),
            _ => None,
        })
        .count();

    debug!("Count Table Sections {}", tables_sections);

    assert!(tables_sections <= 1);

    // The length of C.mems must not be larger than 1.

    let memory_sections = module
        .sections
        .iter()
        .filter_map(|ref w| match w {
            Section::Memory(ref t) => Some(t),
            _ => None,
        })
        .count();

    debug!("Count Memory Sections {}", memory_sections);

    assert!(memory_sections <= 1);

    // All export names exporti.name must be different.

    let export_names: Vec<_> = module
        .sections
        .iter()
        .filter_map(|ref w| match w {
            Section::Export(t) => Some(&t.entries),
            _ => None,
        })
        .flatten()
        .map(|e| &e.name)
        .collect();

    let mut names = std::collections::HashSet::new();

    for name in export_names {
        if !names.contains(&name) {
            names.insert(name);
        } else {
            panic!("Name duplicate {}", name);
        }
    }

    Ok(())
}

fn check_table_ty(table_type: &TableType) -> bool {
    true
}

fn check_elem_ty(elem_ty: &ElementSegment) -> bool {
    true
}

fn check_data_ty(data_ty: &DataSegment) -> bool {
    true
}

fn check_import_ty(import_ty: &ImportEntry) -> bool {
    true
}

fn check_export_ty(import_ty: &ExportEntry) -> bool {
    true
}
/// k is the range
/// k must be between `n` and `m`
pub fn check_limits(limit: &Limits, k: u32) -> bool {
    match limit {
        Limits::Zero(n) => &k > n,
        Limits::One(n, m) => &k > n && m > &k && n < m,
    }
}

pub fn get_ty_of_blocktype(blocktype: BlockType, types: Vec<FuncType>) -> IResult<FuncType> {
    use std::convert::TryInto;

    let w = match blocktype {
        BlockType::ValueType(v) => get_ty_of_valuetype(v),
        BlockType::Empty => get_ty_of_valuetype_empty(),
        BlockType::S33(v) => get_ty_of_typeidx(types, v.try_into().unwrap()).unwrap(), //TODO make this safe
    };

    Ok(w)
}

// If there exists a `typeidx` in `types`, then `typeidx` has its type.
fn get_ty_of_typeidx(types: Vec<FuncType>, typeidx: usize) -> IResult<FuncType> {
    if let Some(t) = types.get(typeidx) {
        return Ok(FuncType {
            param_types: t.param_types.clone(),
            return_types: t.return_types.clone(),
        });
    }

    Err("No function with this index")
}

/// The valuetype has the type `[] -> [valtype]`
fn get_ty_of_valuetype(val: ValueType) -> FuncType {
    match val {
        ValueType::F32 => FuncType {
            param_types: vec![],
            return_types: vec![ValueType::F32],
        },
        ValueType::F64 => FuncType {
            param_types: vec![],
            return_types: vec![ValueType::F32],
        },
        ValueType::I32 => FuncType {
            param_types: vec![],
            return_types: vec![ValueType::I32],
        },
        ValueType::I64 => FuncType {
            param_types: vec![],
            return_types: vec![ValueType::I64],
        },
    }
}

fn check_memory_ty(memory: &MemoryType) -> bool {
    match memory.limits {
        Limits::Zero(n) => n < 2u32.checked_pow(16).unwrap(), //cannot overflow
        Limits::One(n, m) => n < 2u32.checked_pow(16).unwrap() && m < 2u32.checked_pow(16).unwrap(), //cannot overflow
    }
}

fn check_import_desc(e: ImportDesc, types: Vec<FuncType>) -> bool {
    match e {
        ImportDesc::Function { ty } => {
            if let Ok(_) = get_ty_of_typeidx(types, ty as usize) {
                return true;
            }
            false
        }
        ImportDesc::Table { ty: _ } => true, //Limits are u32 that's why they are valid
        ImportDesc::Memory { ty } => check_memory_ty(&ty),
        ImportDesc::Global { ty: _ } => true, // this is true, because `mut` is always correct and `valuetype` was correctly parsed
    }
}

/// The ty has the type `[] -> []`
fn get_ty_of_valuetype_empty() -> FuncType {
    FuncType {
        param_types: vec![],
        return_types: vec![],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_check_limits() {
        let l = Limits::One(10, 20);
        let l2 = Limits::Zero(10);

        assert!(check_limits(&l, 15));
        assert!(check_limits(&l2, 15));

        assert_eq!(false, check_limits(&l, 9));
        assert_eq!(false, check_limits(&l2, 9));
        assert_eq!(false, check_limits(&l, 21));
    }

    #[test]
    fn test_typeidx() {
        let types = vec![FuncType {
            param_types: vec![ValueType::F32, ValueType::F64],
            return_types: vec![ValueType::I64],
        }];

        let ty = FuncType {
            param_types: vec![ValueType::F32, ValueType::F64],
            return_types: vec![ValueType::I64],
        };

        assert_eq!(ty, get_ty_of_typeidx(types, 0).unwrap());
    }

    #[test]
    fn test_blocktype_funcidx() {
        let types = vec![FuncType {
            param_types: vec![ValueType::F32, ValueType::F64],
            return_types: vec![ValueType::I64],
        }];

        let ty = FuncType {
            param_types: vec![ValueType::F32, ValueType::F64],
            return_types: vec![ValueType::I64],
        };

        let bty = get_ty_of_blocktype(BlockType::S33(0), types).unwrap();

        assert_eq!(ty, bty);
    }

    #[test]
    fn test_blocktype_valuetype() {
        let types = vec![];

        let ty = FuncType {
            param_types: vec![],
            return_types: vec![ValueType::I64],
        };

        let bty = get_ty_of_blocktype(BlockType::ValueType(ValueType::I64), types).unwrap();

        assert_eq!(ty, bty);
    }
}
