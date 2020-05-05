use wasm_parser::core::*;
use wasm_parser::Module;

type IResult<T> = Result<T, &'static str>;

type Expr = [Instruction];

pub mod extract;
//pub mod instructions;

use extract::*;

use log::{debug, error};

#[derive(Debug, Clone)]
pub struct FuncType {
    param_types: Vec<ValueType>,
    return_types: Vec<ValueType>,
}

#[derive(Debug, Clone)]
struct Context<'a> {
    types: Vec<&'a FunctionSignature>,
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
    let functions: Vec<IResult<_>> = get_funcs(&module)
        .iter()
        .map(|w| get_ty_of_function(&types, **w as usize))
        .collect();

    for f in functions.iter() {
        if f.is_err() {
            error!("Function {:?}", f);
            return Err("Function is not defined");
        }
    }

    let tables = get_tables(&module);
    let mems = get_mems(&module);
    let (global_entries, globals_ty) = get_globals(&module);

    let c = Context {
        types,
        functions: functions.into_iter().map(|w| w.unwrap()).collect(), //save because checked
        tables,
        mems,
        global_entries,
        globals_ty,
        locals: Vec::new(),
        labels: Vec::new(),
        _return: Vec::new(),
    };

    c.validate(&module)?;

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

    pub fn validate(&self, module: &Module) -> IResult<()> {
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
            check_memory_ty(mem)?;
        }

        debug!("MemoryTypes are valid");

        // Check global

        for entry in self.global_entries.iter() {
            // For each global under the Context C'

            let init = &entry.init;
            let init_expr_ty = get_expr_const_ty_global(init, &c_prime.globals_ty)?;

            if entry.ty.value_type != init_expr_ty {
                //Expr has not the same type as the global
                return Err("Expr has not the same type as the global");
            }
        }

        debug!("GlobalTypes are valid");

        // Check elem

        let elements = get_elemens(module);
        for elem in elements {
            check_elem_ty(elem, &self.tables, &self.functions)?;
        }

        debug!("Elements are valid");

        // Check data

        let data = get_data(module);
        for d in data {
            check_data_ty(d, &self.mems)?;
        }

        debug!("Data is valid");

        // Start

        let start = get_start(module);

        if let Some(s) = start.get(0) {
            check_start(s, &self.functions)?;
        }

        debug!("Start is valid");

        // Imports

        let imports = get_imports(module);
        for e in imports {
            debug!("Checking import {}/{}", e.module_name, e.name);
            check_import_ty(e, &self.types)?;
        }

        debug!("Imports are valid");

        // Check exports

        let exports = get_exports(module);
        for e in exports {
            debug!("Checking export {}", e.name);
            check_export_ty(
                e,
                &self.functions,
                &self.tables,
                &self.mems,
                &self.globals_ty,
            )?;
        }

        debug!("Exports are valid");

        check_lengths(self)?;

        let exports = get_exports(module);
        check_export_names(&exports)?;

        Ok(())
    }
}

fn check_lengths(c: &Context) -> IResult<()> {
    // tables must not be larger than 1

    if c.tables.len() > 1 {
        return Err("More than one table");
    }

    debug!("Table size is ok");

    // Memory must not be larger than 1

    if c.mems.len() > 1 {
        return Err("More than one memory");
    }

    Ok(())
}

/// All export names must be different
fn check_export_names(exports: &[&ExportEntry]) -> IResult<()> {
    let mut set = std::collections::HashSet::new();

    for e in exports.iter() {
        if !set.contains(&e.name) {
            set.insert(e.name.clone());
        } else {
            return Err("Export function names are not unique");
        }
    }

    debug!("Export names are unique");

    Ok(())
}

/// Evalutes the expr `init` and checks if it returns const
fn get_expr_const_ty_global(init: &Expr, globals_ty: &[&GlobalType]) -> IResult<ValueType> {
    use wasm_parser::core::NumericInstructions::*;
    use wasm_parser::core::VarInstructions::*;

    if init.is_empty() {
        return Err("No expr to evaluate");
    }

    match init.get(0).unwrap() {
        Instruction::Num(n) => match *n {
            OP_I32_CONST(_) => Ok(ValueType::I32),
            OP_I64_CONST(_) => Ok(ValueType::I64),
            OP_F32_CONST(_) => Ok(ValueType::F32),
            OP_F64_CONST(_) => Ok(ValueType::F64),
            _ => Err("Expression is not a const"),
        },
        Instruction::Var(n) => match *n {
            OP_GLOBAL_GET(lidx) => match globals_ty.get(lidx as usize).as_ref() {
                Some(global) => {
                    if global.mu == Mu::Var {
                        return Err("Global var is mutable");
                    }

                    Ok(global.value_type)
                }
                None => Err("Global does not exist"),
            },
            _ => Err("Only Global get allowed"),
        },
        _ => Err("Wrong expression"),
    }
}

fn check_elem_ty(
    elem_ty: &ElementSegment,
    tables: &[&TableType],
    func_ty: &[FuncType],
) -> IResult<bool> {
    debug!("check_elem_ty");
    //https://webassembly.github.io/spec/core/valid/modules.html#element-segments

    let table_idx = &elem_ty.table;
    let offset = &elem_ty.offset;
    let funcs_idx = &elem_ty.init;

    if tables.get(*table_idx as usize).is_none() {
        return Err("No table defined for element's index");
    }

    get_expr_const_i32_ty(offset)?;


    debug!("defined functions {:#?}", func_ty);

    let not_def_funcs: Vec<_> = funcs_idx
        .iter()
        .filter(|w| func_ty.get(**w as usize).is_none())
        .collect();

    for f in not_def_funcs.iter() {
        error!("function is not defined {:?}", f);
    }

    if !not_def_funcs.is_empty() {
        return Err("Element section is not correct");
    }

    Ok(true)
}

/// Evalutes the expr `init` and checks if it returns const and I32
fn get_expr_const_i32_ty(init: &Expr) -> IResult<ValueType> {
    use wasm_parser::core::NumericInstructions::*;

    if init.is_empty() {
        return Err("No expr to evaluate");
    }

    match init.get(0).unwrap() {
        Instruction::Num(n) => match *n {
            OP_I32_CONST(_) => Ok(ValueType::I32),
            _ => Err("Expression is not a I32 const"),
        },
        _ => Err("Wrong expression"),
    }
}

fn check_data_ty(data_ty: &DataSegment, memtypes: &[&MemoryType]) -> IResult<bool> {
    //https://webassembly.github.io/spec/core/valid/modules.html#data-segments

    let mem_idx = data_ty.data;
    let offset = &data_ty.offset;

    if memtypes.get(mem_idx as usize).is_none() {
        panic!("Memory does not exist");
    }

    get_expr_const_i32_ty(&offset)?;

    Ok(true)
}

fn check_start(start: &StartSection, functypes: &[FuncType]) -> IResult<bool> {
    //https://webassembly.github.io/spec/core/valid/modules.html#valid-start

    let fidx = start.index;

    if let Some(f) = functypes.get(fidx as usize).as_ref() {
        if !f.param_types.is_empty() && !f.return_types.is_empty() {
            error!("Function {:?}", f);
            return Err("Function is not a valid start function");
        }
    }

    Ok(true)
}

fn check_import_ty(import_ty: &ImportEntry, types: &[&FunctionSignature]) -> IResult<bool> {
    check_import_desc(&import_ty.desc, types)
}

fn check_export_ty(
    export_ty: &ExportEntry,
    functypes: &[FuncType],
    tabletypes: &[&TableType],
    memtypes: &[&MemoryType],
    globaltypes: &[&GlobalType],
) -> IResult<bool> {
    //https://webassembly.github.io/spec/core/valid/modules.html#exports

    macro_rules! exists(
        ($e:ident, $w:ident, $k:expr) => (
            match $e.get($w as usize).as_ref() {
                Some(_) => {Ok(true)}, //exists
                _ => Err($k)
            }
        )
    );

    match export_ty.kind {
        ExternalKindType::Function { ty } => exists!(functypes, ty, "Function does not exist"),
        ExternalKindType::Table { ty } => exists!(tabletypes, ty, "Table does not exist"),
        ExternalKindType::Memory { ty } => exists!(memtypes, ty, "Memory does not exist"),
        ExternalKindType::Global { ty } => exists!(globaltypes, ty, "Global does not exist"),
    }
}

// If there exists a `typeidx` in `types`, then `typeidx` has its type.
fn get_ty_of_function(types: &[&FunctionSignature], typeidx: usize) -> IResult<FuncType> {
    if let Some(t) = types.get(typeidx) {
        return Ok(FuncType {
            param_types: t.param_types.clone(),
            return_types: t.return_types.clone(),
        });
    }

    Err("No function with this index")
}

fn check_import_desc(e: &ImportDesc, types: &[&FunctionSignature]) -> IResult<bool> {
    let b = match e {
        ImportDesc::Function { ty } => get_ty_of_function(types, *ty as usize).is_ok(),
        ImportDesc::Table { .. } => true, //Limits are u32 that's why they are valid
        ImportDesc::Memory { ty } => check_memory_ty(&ty).is_ok(),
        ImportDesc::Global { .. } => true, // this is true, because `mut` is always correct and `valuetype` was correctly parsed
    };

    Ok(b)
}

fn check_memory_ty(memory: &MemoryType) -> IResult<()> {
    let b = match memory.limits {
        Limits::Zero(n) => n < 2u32.checked_pow(16).unwrap(), //cannot overflow
        Limits::One(n, m) => n < 2u32.checked_pow(16).unwrap() && m < 2u32.checked_pow(16).unwrap(), //cannot overflow
    };

    if b {
        Ok(())
    } else {
        Err("Memory exhausted")
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use wasm_parser::parse;
    use wasm_parser::read_wasm;

    #[test]
    fn test_empty_module() {
        let module = Module { sections: vec![] };

        assert!(validate(&module).is_ok());
    }

    #[test]
    fn test_check_if_function_defined() {
        let w = FunctionSection { types: vec![0, 1] };

        let t = TypeSection {
            entries: vec![FunctionSignature::empty()],
        };

        let module = Module {
            sections: vec![Section::Type(t), Section::Function(w)],
        };

        assert_eq!(Err("Function is not defined"), validate(&module));
    }

    #[test]
    fn test_check_function_defined() {
        let w = FunctionSection { types: vec![0, 0] };

        let t = TypeSection {
            entries: vec![FunctionSignature::empty()],
        };

        let module = Module {
            sections: vec![Section::Type(t), Section::Function(w)],
        };

        assert!(validate(&module).is_ok());
    }

    #[test]
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

        let w = FunctionSection { types: vec![0, 1] };

        let t = TypeSection {
            entries: vec![FunctionSignature::empty(), FunctionSignature::empty()],
        };

        let module = Module {
            sections: vec![Section::Type(t), Section::Function(w), Section::Export(k)],
        };

        assert_eq!(
            Err("Export function names are not unique"),
            validate(&module)
        );
    }

    #[test]
    fn test_table_in_export() {
        let k = ExportSection {
            entries: vec![ExportEntry {
                name: "test1".to_string(),
                kind: ExternalKindType::Table { ty: 0 },
            }],
        };

        let w = TableSection {
            entries: vec![TableType {
                element_type: 0x70, //default
                limits: Limits::Zero(10),
            }],
        };

        let module = Module {
            sections: vec![Section::Table(w), Section::Export(k)],
        };

        assert!(validate(&module).is_ok());
    }

    #[test]
    fn test_no_table_in_export() {
        let k = ExportSection {
            entries: vec![ExportEntry {
                name: "test1".to_string(),
                kind: ExternalKindType::Table { ty: 0 },
            }],
        };

        let w = TableSection { entries: vec![] };

        let module = Module {
            sections: vec![Section::Export(k), Section::Table(w)],
        };

        assert_eq!(Err("Table does not exist"), validate(&module));
    }

    #[test]
    fn test_memory_in_export() {
        let k = ExportSection {
            entries: vec![ExportEntry {
                name: "test1".to_string(),
                kind: ExternalKindType::Memory { ty: 0 },
            }],
        };

        let w = MemorySection {
            entries: vec![MemoryType {
                limits: Limits::Zero(10),
            }],
        };

        let module = Module {
            sections: vec![Section::Memory(w), Section::Export(k)],
        };

        assert!(validate(&module).is_ok());
    }

    #[test]
    fn test_no_memory_in_export() {
        let k = ExportSection {
            entries: vec![ExportEntry {
                name: "test1".to_string(),
                kind: ExternalKindType::Memory { ty: 0 },
            }],
        };

        let w = MemorySection { entries: vec![] };

        let module = Module {
            sections: vec![Section::Export(k), Section::Memory(w)],
        };

        assert_eq!(Err("Memory does not exist"), validate(&module));
    }

    #[test]
    fn test_global_in_export() {
        let k = ExportSection {
            entries: vec![ExportEntry {
                name: "test1".to_string(),
                kind: ExternalKindType::Global { ty: 0 },
            }],
        };

        let w = GlobalSection {
            globals: vec![GlobalVariable {
                ty: GlobalType {
                    value_type: ValueType::I32,
                    mu: Mu::Const,
                },
                init: vec![Instruction::Num(NumericInstructions::OP_I32_CONST(1))],
            }],
        };

        let module = Module {
            sections: vec![Section::Export(k), Section::Global(w)],
        };

        assert!(validate(&module).is_ok());
    }

    #[test]
    fn test_no_global_in_export() {
        let k = ExportSection {
            entries: vec![ExportEntry {
                name: "test1".to_string(),
                kind: ExternalKindType::Global { ty: 0 },
            }],
        };

        let w = GlobalSection { globals: vec![] };

        let module = Module {
            sections: vec![Section::Export(k), Section::Global(w)],
        };

        assert_eq!(Err("Global does not exist"), validate(&module));
    }

    #[test]
    fn test_function_import() {
        let k = ImportSection {
            entries: vec![ImportEntry {
                module_name: "test".to_string(),
                name: "test1".to_string(),
                desc: ImportDesc::Function { ty: 0 },
            }],
        };

        let x = TypeSection {
            entries: vec![FunctionSignature {
                param_types: vec![],
                return_types: vec![],
            }],
        };

        let w = FunctionSection { types: vec![0] };

        let module = Module {
            sections: vec![Section::Import(k), Section::Type(x), Section::Function(w)],
        };

        assert!(validate(&module).is_ok());
    }

    #[test]
    fn test_no_function_import() {
        let k = ImportSection {
            entries: vec![ImportEntry {
                module_name: "test".to_string(),
                name: "test1".to_string(),
                desc: ImportDesc::Function { ty: 0 },
            }],
        };

        let x = TypeSection { entries: vec![] };

        let w = FunctionSection { types: vec![] };

        let module = Module {
            sections: vec![Section::Import(k), Section::Type(x), Section::Function(w)],
        };

        assert_eq!(Err("Function is not defined"), validate(&module));
    }

    #[test]
    fn test_table_import() {
        let k = ImportSection {
            entries: vec![ImportEntry {
                module_name: "test".to_string(),
                name: "test1".to_string(),
                desc: ImportDesc::Table {
                    ty: TableType {
                        element_type: 0x70,
                        limits: Limits::Zero(0),
                    },
                },
            }],
        };

        let module = Module {
            sections: vec![Section::Import(k)],
        };

        assert!(validate(&module).is_ok());
    }

    #[test]
    fn test_memory_import() {
        let k = ImportSection {
            entries: vec![ImportEntry {
                module_name: "test".to_string(),
                name: "test1".to_string(),
                desc: ImportDesc::Memory {
                    ty: MemoryType {
                        limits: Limits::Zero(0),
                    },
                },
            }],
        };

        let module = Module {
            sections: vec![Section::Import(k)],
        };

        assert!(validate(&module).is_ok());
    }

    #[test]
    fn test_memory_exhaust_import() {
        let k = ImportSection {
            entries: vec![ImportEntry {
                module_name: "test".to_string(),
                name: "test1".to_string(),
                desc: ImportDesc::Memory {
                    ty: MemoryType {
                        limits: Limits::Zero(u32::max_value()),
                    },
                },
            }],
        };

        let module = Module {
            sections: vec![Section::Import(k)],
        };

        assert_eq!(Err("Memory exhausted"), validate(&module));
    }

    #[test]
    fn test_global_import() {
        let k = ImportSection {
            entries: vec![ImportEntry {
                module_name: "test".to_string(),
                name: "test1".to_string(),
                desc: ImportDesc::Global {
                    ty: GlobalType {
                        value_type: ValueType::I32,
                        mu: Mu::Var,
                    },
                },
            }],
        };

        let module = Module {
            sections: vec![Section::Import(k)],
        };

        assert!(validate(&module).is_ok());
    }

    #[test]
    fn test_double_tables() {
        let k = TableSection {
            entries: vec![TableType {
                element_type: 0x70,
                limits: Limits::Zero(0 as u32),
            }],
        };

        let w = TableSection {
            entries: vec![TableType {
                element_type: 0x70,
                limits: Limits::Zero(0 as u32),
            }],
        };

        let module = Module {
            sections: vec![Section::Table(k), Section::Table(w)],
        };

        assert_eq!(Err("More than one table"), validate(&module));
    }

    #[test]
    fn test_double_memory() {
        let k = MemorySection {
            entries: vec![MemoryType {
                limits: Limits::Zero(0 as u32),
            }],
        };

        let w = MemorySection {
            entries: vec![MemoryType {
                limits: Limits::Zero(0 as u32),
            }],
        };

        let module = Module {
            sections: vec![Section::Memory(k), Section::Memory(w)],
        };

        assert_eq!(Err("More than one memory"), validate(&module));
    }

    #[test]
    fn test_memory_exhaust() {
        let k = MemorySection {
            entries: vec![MemoryType {
                limits: Limits::Zero(u32::max_value()),
            }],
        };

        let module = Module {
            sections: vec![Section::Memory(k)],
        };

        assert_eq!(Err("Memory exhausted"), validate(&module));
    }

    #[test]
    fn test_memory_exhaust_function() {
        let ty = MemoryType {
            limits: Limits::Zero(u32::max_value()),
        };

        assert_eq!(Err("Memory exhausted"), check_memory_ty(&ty));
    }

    macro_rules! test_file {
        ($fs_name:expr) => {
            let file = read_wasm!(&format!("../wasm_parser/test_files/{}", $fs_name));
            let ast = parse(file).unwrap();
            assert!(validate(&ast).is_ok());
        };
    }

    #[test]
    fn test_parse_return_i32() {
        test_file!("return_i32.wasm");
    }

    #[test]
    fn test_return_i64() {
        test_file!("return_i64.wasm");
    }

    #[test]
    fn test_function_call() {
        test_file!("function_call.wasm");
    }

    #[test]
    fn test_arithmetic() {
        test_file!("arithmetic.wasm");
    }

    #[test]
    fn test_block_add_i32() {
        test_file!("block_add_i32.wasm");
    }

    #[test]
    fn test_loop_mult() {
        test_file!("loop_mult.wasm");
    }

    #[test]
    fn test_unreachable() {
        test_file!("unreachable.wasm");
    }

    #[test]
    fn test_if_loop() {
        test_file!("if_loop.wasm");
    }

    #[test]
    fn test_logic() {
        test_file!("logic.wasm");
    }

    #[test]
    fn test_gcd() {
        test_file!("gcd.wasm");
    }
}
