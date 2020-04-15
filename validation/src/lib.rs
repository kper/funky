use wasm_parser::core::*;
use wasm_parser::Module;

type IResult<T> = Result<T, &'static str>;

type Expr = [Instruction];

pub mod instructions;
mod extract;

use concat::*;

// Leading question: Should validation return errors or panic?

use log::{debug, error};

#[derive(Debug, Clone)]
struct Context<'a> {
    types: Vec<&'a FuncType>,
    functions: Vec<FuncType>,
    tables: Vec<&'a TableType>,
    mems: Vec<&'a MemoryType>,
    global_entries: Vec<&'a GlobalVariable>,
    globals_ty: Vec<&'a GlobalType>,
    locals: Vec<()>,  //TODO
    labels: Vec<()>,  //TODO
    _return: Vec<()>, //TODO
}

pub fn validate(module: &Module) -> IResult<()> {
    let types = get_types(&module);
    let functions = get_funcs(&module)
        .iter()
        .map(|w| get_ty_of_function(&types, **w as usize).unwrap())
        .collect();
    let tables = get_tables(&module);
    let mems = get_mems(&module);
    let (global_entries, globals_ty) = get_globals(&module);

    let c = Context {
        types,
        functions,
        tables,
        mems,
        global_entries,
        globals_ty,
        locals: Vec::new(),
        labels: Vec::new(),
        _return: Vec::new(),
    };

    c.validate(&module);

    Ok(())
}

impl<'a> Context<'a> {
    pub fn get_c_prime(&self) -> Self {
        let copied = (&self.globals_ty).to_vec(); //TODO is this really copied?
        let copied2 = (&self.global_entries).to_vec(); //TODO is this really copied?

        Context {
            types: Vec::new(),
            functions: Vec::new(),
            tables: Vec::new(),
            mems: Vec::new(),
            global_entries: copied2,
            globals_ty: copied,
            locals: Vec::new(),
            labels: Vec::new(),
            _return: Vec::new(),
        }
    }

    pub fn validate(&self, module: &Module) {
        let c_prime = self.get_c_prime().clone(); //TODO this might not be necessary

        // Check functype

        // -> https://webassembly.github.io/spec/core/valid/types.html#t-1-n-xref-syntax-types-syntax-functype-rightarrow-t-2-m

        // they are always valid
        debug!("Functypes are valid");

        // Check func

        // due to optimization, that has been already checked in top-level `validate`
        debug!("FunctionsTypes are valid");

        // Check table

        //https://webassembly.github.io/spec/core/valid/types.html#table-types

        // table tables are valid when the limit is in u32 range
        // that's statically guaranted
        debug!("TablesTypes are valid");

        // Check mem

        for mem in self.mems.iter() {
            assert!(check_memory_ty(mem));
        }

        debug!("MemoryTypes are valid");

        // Check global

        for entry in self.global_entries.iter() {
            // For each global under the Context C'

            let init = &entry.init;
            let init_expr_ty = get_expr_const_ty_global(init, &c_prime.globals_ty);

            if entry.ty.value_type != init_expr_ty {
                //Expr has not the same type as the global
                panic!("Expr has not the same type as the global");
            }
        }

        debug!("GlobalTypes are valid");

        // Check elem

        let elements = get_elemens(module);
        for elem in elements {
            assert!(check_elem_ty(elem, &self.tables, &self.types));
        }

        debug!("Elements are valid");

        // Check data

        let data = get_data(module);
        for d in data {
            assert!(check_data_ty(d, &self.mems));
        }

        debug!("Data is valid");

        // Start

        let start = get_start(module);

        if let Some(s) = start.get(0) {
            assert!(check_start(s, &self.types));
        }

        debug!("Start is valid");

        // Imports

        let imports = get_imports(module);
        for e in imports {
            assert!(check_import_ty(e, &self.types,));
        }

        debug!("Imports are valid");

        // Check exports

        let exports = get_exports(module);
        for e in exports {
            assert!(check_export_ty(
                e,
                &self.types,
                &self.tables,
                &self.mems,
                &self.globals_ty
            ));
        }

        debug!("Exports are valid");

        check_lengths(self);

        let exports = get_exports(module);
        check_export_names(&exports);
    }
}

fn check_lengths(c: &Context) {
    // tables must not be larger than 1

    if c.tables.len() > 1 {
        panic!("Tables are larger than 1");
    }

    debug!("Table size is ok");

    // Memory must not be larger than 1

    if c.mems.len() > 1 {
        panic!("Memory are larger than 1");
    }
}

/// All export names must be different
fn check_export_names(exports: &[&ExportEntry]) {
    let mut set = std::collections::HashSet::new();

    for e in exports.iter() {
        if !set.contains(&e.name) {
            set.insert(e.name.clone());
        } else {
            panic!("Export function names are not unique");
        }
    }

    debug!("Export names are unique");
}

/// Evalutes the expr `init` and checks if it returns const
fn get_expr_const_ty_global(init: &Expr, globals_ty: &[&GlobalType]) -> ValueType {
    use wasm_parser::core::NumericInstructions::*;
    use wasm_parser::core::VarInstructions::*;

    if init.is_empty() {
        panic!("No expr to evaluate");
    }

    match init.get(0).unwrap() {
        Instruction::Num(n) => match *n {
            OP_I32_CONST(_) => ValueType::I32,
            OP_I64_CONST(_) => ValueType::I64,
            OP_F32_CONST(_) => ValueType::F32,
            OP_F64_CONST(_) => ValueType::F64,
            _ => panic!("Expression is not a const"),
        },
        Instruction::Var(n) => match *n {
            OP_GLOBAL_GET(lidx) => match globals_ty.get(lidx as usize).as_ref() {
                Some(global) => {
                    if global.mu == Mu::Var {
                        panic!("Global var is mutable");
                    }

                    global.value_type.clone()
                }
                None => panic!("Global does not exist"),
            },
            _ => panic!("Only Global get allowed"),
        },
        _ => panic!("Wrong expression"),
    }
}

fn check_elem_ty(elem_ty: &ElementSegment, tables: &[&TableType], func_ty: &[&FuncType]) -> bool {
    //https://webassembly.github.io/spec/core/valid/modules.html#element-segments

    let table_idx = &elem_ty.index;
    let offset = &elem_ty.offset;
    let funcs_idx = &elem_ty.elems;

    if tables.get(*table_idx as usize).is_none() {
        panic!("No table defined for element's index");
    }

    get_expr_const_i32_ty(offset);

    // All function must be defined

    let not_def_funcs: Vec<_> = funcs_idx
        .iter()
        .filter_map(|w| func_ty.get(*w as usize))
        .collect();

    for f in not_def_funcs.iter() {
        error!("function is not defined {:?}", f);
    }

    if !not_def_funcs.is_empty() {
        panic!("Element section is not correct");
    }

    true
}

/// Evalutes the expr `init` and checks if it returns const and I32
fn get_expr_const_i32_ty(init: &Expr) {
    use wasm_parser::core::NumericInstructions::*;

    if init.is_empty() {
        panic!("No expr to evaluate");
    }

    let _ = match init.get(0).unwrap() {
        Instruction::Num(n) => match *n {
            OP_I32_CONST(_) => ValueType::I32,
            //OP_I64_CONST(_) => ValueType::I64,
            //OP_F32_CONST(_) => ValueType::F32,
            //OP_F64_CONST(_) => ValueType::F64,
            _ => panic!("Expression is not a I32 const"),
        },
        _ => panic!("Wrong expression"),
    };
}

fn check_data_ty(data_ty: &DataSegment, memtypes: &[&MemoryType]) -> bool {
    //https://webassembly.github.io/spec/core/valid/modules.html#data-segments

    let mem_idx = data_ty.index;
    let offset = &data_ty.offset;

    if memtypes.get(mem_idx as usize).is_none() {
        panic!("Memory does not exist");
    }

    get_expr_const_i32_ty(&offset);

    true
}

fn check_start(start: &StartSection, functypes: &[&FuncType]) -> bool {
    //https://webassembly.github.io/spec/core/valid/modules.html#valid-start

    let fidx = start.index;

    if let Some(f) = functypes.get(fidx as usize).as_ref() {
        if !f.param_types.is_empty() && !f.return_types.is_empty() {
            panic!("Function is not a valid start function {:?}", f);
        }
    }

    true
}

fn check_import_ty(import_ty: &ImportEntry, functypes: &[&FuncType]) -> bool {
    check_import_desc(&import_ty.desc, functypes)
}

fn check_export_ty(
    export_ty: &ExportEntry,
    functypes: &[&FuncType],
    tabletypes: &[&TableType],
    memtypes: &[&MemoryType],
    globaltypes: &[&GlobalType],
) -> bool {
    //https://webassembly.github.io/spec/core/valid/modules.html#exports

    macro_rules! exists(
        ($e:ident, $w:ident, $k:expr) => (
            match $e.get($w as usize).as_ref() {
                Some(_) => {}, //exists
                _ => panic!($k)
            }
        )
    );

    match export_ty.kind {
        ExternalKindType::Function { ty } => exists!(functypes, ty, "Function does not exist"),
        ExternalKindType::Table { ty } => exists!(tabletypes, ty, "Table does not exist"),
        ExternalKindType::Memory { ty } => exists!(memtypes, ty, "Memory does not exist"),
        ExternalKindType::Global { ty } => exists!(globaltypes, ty, "Global does not exist"),
    }

    true
}

/*
/// k is the range
/// k must be between `n` and `m`
fn check_limits(limit: &Limits, k: u32) -> bool {
    match limit {
        Limits::Zero(n) => &k > n,
        Limits::One(n, m) => &k > n && m > &k && n < m,
    }
}
*/

/*
pub fn get_ty_of_blocktype(blocktype: BlockType, types: Vec<FuncType>) -> IResult<FuncType> {
    use std::convert::TryInto;

    let w = match blocktype {
        BlockType::ValueType(v) => get_ty_of_valuetype(v),
        BlockType::Empty => get_ty_of_valuetype_empty(),
        BlockType::S33(v) => get_ty_of_function(types, v.try_into().unwrap()).unwrap(), //TODO make this safe
    };

    Ok(w)
}
*/

// If there exists a `typeidx` in `types`, then `typeidx` has its type.
fn get_ty_of_function(types: &[&FuncType], typeidx: usize) -> IResult<FuncType> {
    if let Some(t) = types.get(typeidx) {
        return Ok(FuncType {
            param_types: t.param_types.clone(),
            return_types: t.return_types.clone(),
        });
    }

    Err("No function with this index")
}

/*
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
*/

fn check_import_desc(e: &ImportDesc, types: &[&FuncType]) -> bool {
    match e {
        ImportDesc::Function { ty } => get_ty_of_function(types, *ty as usize).is_ok(),
        ImportDesc::Table { .. } => true, //Limits are u32 that's why they are valid
        ImportDesc::Memory { ty } => check_memory_ty(&ty),
        ImportDesc::Global { .. } => true, // this is true, because `mut` is always correct and `valuetype` was correctly parsed
    }
}

fn check_memory_ty(memory: &MemoryType) -> bool {
    match memory.limits {
        Limits::Zero(n) => n < 2u32.checked_pow(16).unwrap(), //cannot overflow
        Limits::One(n, m) => n < 2u32.checked_pow(16).unwrap() && m < 2u32.checked_pow(16).unwrap(), //cannot overflow
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    #[should_panic("Export Functions are not unique")]
    fn test_duplicated_export_entries() {
        let k = ExportSection {
            entries: vec![
                ExportEntry {
                    name: "test1".to_string(),
                    kind: ExternalKindType::Function { ty: 0 },
                },
                ExportEntry {
                    name: "test1".to_string(),
                    kind: ExternalKindType::Function { ty: 1 },
                },
            ],
        };

        //TODO if this is a requirement too?
        let w = FunctionSection { types: vec![0, 1] };

        let t = TypeSection {
            entries: vec![FuncType::empty(), FuncType::empty()],
        };

        let module = Module {
            sections: vec![Section::Type(t), Section::Function(w), Section::Export(k)],
        };

        validate(&module).unwrap();
    }

    /*
    #[test]
    fn test_check_limits() {
        let l = Limits::One(10, 20);
        let l2 = Limits::Zero(10);

        assert!(check_limits(&l, 15));
        assert!(check_limits(&l2, 15));

        assert_eq!(false, check_limits(&l, 9));
        assert_eq!(false, check_limits(&l2, 9));
        assert_eq!(false, check_limits(&l, 21));
    }*/
}
