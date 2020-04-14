use wasm_parser::core::*;
use wasm_parser::Module;

type IResult<T> = Result<T, &'static str>;

pub mod instructions;
//TODO rename module to `extract`
mod concat;

use concat::*;

// Leading question: Should validation return errors or panic?

use log::debug;

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
        .map(|w| get_ty_of_function(Vec::new(), **w as usize).unwrap())
        .collect();
    let tables = get_tables(&module);
    let mems = get_mems(&module);
    let (global_entries, globals_ty) = get_globals(&module);

    let C = Context {
        types: types,
        functions: functions,
        tables: tables,
        mems: mems,
        global_entries: global_entries,
        globals_ty: globals_ty,
        locals: Vec::new(),
        labels: Vec::new(),
        _return: Vec::new(),
    };

    C.validate();

    // Check elem

    let elements = get_elemens(module);
    for elem in elements {
        assert!(check_elem_ty(elem));
    }

    // Check data

    let data = get_data(module);
    for d in data {
        assert!(check_data_ty(d));
    }

    // Start

    let start : Vec<_> = module
        .sections
        .iter()
        .filter_map(|w| match w {
            Section::Start(t) => Some(t),
            _ => None,
        })
        .collect();

    if let Some(s) = start.get(0) {
        assert!(check_start(s));
    }

    Ok(())
}

impl<'a> Context<'a> {
    pub fn get_C_prime(&self) -> Self {
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

    pub fn validate(self) {
        let C_prime = self.get_C_prime().clone(); //TODO this might not be necessary

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

        for mem in self.mems {
            assert!(check_memory_ty(mem));
        }

        debug!("MemoryTypes are valid");

        // Check global

        for entry in self.global_entries {
            // For each global under the Context C'

            let init = &entry.init;
            let init_expr_ty = get_expr_const_ty(init, &C_prime.globals_ty);

            if entry.ty.value_type != init_expr_ty {
                //Expr has not the same type as the global
                panic!("Expr has not the same type as the global");
            }
        }

        debug!("GlobalTypes are valid");
    }
}

/// Evalutes the expr `init` and checks if it returns const
fn get_expr_const_ty(init: &Expr, globals_ty: &Vec<&GlobalType>) -> ValueType {
    use wasm_parser::core::Instruction;
    use wasm_parser::core::Mu;
    use wasm_parser::core::NumericInstructions::*;
    use wasm_parser::core::VarInstructions::*;

    if init.len() == 0 {
        panic!("No expr to evaluate");
    }

    let expr_ty = match init.get(0).unwrap() {
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
    };

    expr_ty
}

fn check_elem_ty(elem_ty: &ElementSegment) -> bool {
    true
}

fn check_data_ty(data_ty: &DataSegment) -> bool {
    true
}

fn check_start(start: &&StartSection) -> bool {
    true
}

/*
fn check_import_ty(import_ty: &ImportEntry) -> bool {
    true
}

fn check_export_ty(import_ty: &ExportEntry) -> bool {
    true
}
*/

/// k is the range
/// k must be between `n` and `m`
fn check_limits(limit: &Limits, k: u32) -> bool {
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
        BlockType::S33(v) => get_ty_of_function(types, v.try_into().unwrap()).unwrap(), //TODO make this safe
    };

    Ok(w)
}

// If there exists a `typeidx` in `types`, then `typeidx` has its type.
fn get_ty_of_function(types: Vec<FuncType>, typeidx: usize) -> IResult<FuncType> {
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
            if let Ok(_) = get_ty_of_function(types, ty as usize) {
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
