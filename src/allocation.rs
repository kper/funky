use crate::engine::*;
use std::cell::RefCell;
use std::rc::Rc;
use wasm_parser::core::*;
use wasm_parser::Module;

pub fn allocate(
    m: &Module,
    mod_instance: &ModuleInstance,
    store: &mut Store,
) -> Result<(), ()> {
    debug!("allocate");

    // Step 1
    let _imports = get_extern_values_in_imports(m)?;

    // Step 2a and 6
    allocate_functions(m, mod_instance, store)?;
    //TODO host functions

    // Step 3a and 7
    allocate_tables(m, mod_instance, store)?;

    // Step 4a and 8
    allocate_memories(m, mod_instance, store)?;

    // Step 5a and 9
    allocate_globals(m, mod_instance, store)?;

    // ... Step 13

    // Step 14.

    allocate_exports(m, mod_instance, store)?;

    // Step 15.

    Ok(())
}

fn get_extern_values_in_imports<'a>(m: &'a Module) -> Result<Vec<&'a ImportDesc>, ()> {
    let ty: Vec<_> = m
        .sections
        .iter()
        .filter_map(|ref w| match w {
            Section::Import(t) => Some(&t.entries),
            _ => None,
        })
        .flatten()
        .map(|w| &w.desc)
        .collect();

    Ok(ty)
}

fn allocate_functions(
    m: &Module,
    mod_instance: &ModuleInstance,
    store: &mut Store,
) -> Result<(), ()> {
    debug!("allocate function");

    debug!("module {:#?}", m);

    // Gets all functions and imports
    let ty = validation::extract::get_funcs(&m);

    debug!("functions extracted {:#?}", ty);

    for (code_index, t) in ty.iter().enumerate() {
        debug!("Function {:#?}", t);
        // Allocate function

        {
            let borrow = mod_instance;
            let fbody = match borrow.fn_types.get(**t as usize) {
                Some(fbody) => fbody,
                None => {
                    panic!("{} function type is not defined", t);
                }
            };

            let fcode = match borrow.code.get(code_index as usize) {
                Some(fcode) => fcode,
                None => {
                    panic!("{} code is not defined", t);
                }
            };

            let instance = FuncInstance {
                ty: fbody.clone(),
                //module: weak.clone(),
                code: fcode.clone(),
            };

            store.funcs.push(instance);
        }

        mod_instance
            .funcaddrs
            .push(store.funcs.len() as u32 - 1);
    }

    debug!("Functions in store {:#?}", store.funcs);

    Ok(())
}

fn allocate_tables(
    m: &Module,
    mod_instance: &ModuleInstance,
    store: &mut Store,
) -> Result<(), ()> {
    debug!("allocate tables");

    // Gets all tables and imports
    let ty = validation::extract::get_tables(&m);

    for t in ty.iter() {
        debug!("table {:#?}", t);
        let instance = match t.limits {
            Limits::Zero(n) => TableInstance {
                elem: vec![None; n as usize],
                max: None,
            },
            Limits::One(n, m) => TableInstance {
                elem: vec![None; n as usize],
                max: Some(m),
            },
        };

        mod_instance
            .tableaddrs
            .push(store.tables.len() as u32);
        store.tables.push(instance);
    }

    debug!("Tables in mod_i {:?}", mod_instance.tableaddrs);
    debug!("Tables in store {:#?}", store.tables);

    Ok(())
}

fn allocate_memories(
    m: &Module,
    mod_instance: &ModuleInstance,
    store: &mut Store,
) -> Result<(), ()> {
    debug!("allocate memories");
    // Gets all memories and imports
    let ty = validation::extract::get_mems(&m);

    for memtype in ty.iter() {
        debug!("memtype {:#?}", memtype);
        let instance = match memtype.limits {
            Limits::Zero(n) => MemoryInstance {
                data: vec![0u8; (n * 1024 * 64) as usize],
                max: None,
            },
            Limits::One(n, m) => MemoryInstance {
                data: vec![0u8; (n * 1024 * 64) as usize],
                max: Some(m),
            },
        };

        mod_instance
            .memaddrs
            .push(store.memory.len() as u32);
        store.memory.push(instance);
    }

    debug!("Memories in mod_i {:?}", mod_instance.memaddrs);
    debug!("Memories in store {:#?}", store.memory);

    Ok(())
}

fn allocate_globals(
    m: &Module,
    mod_instance: &ModuleInstance,
    store: &mut Store,
) -> Result<(), ()> {
    debug!("allocate globals");
    // Gets all globals and imports
    let ty = validation::extract::get_globals(&m);

    for gl in ty.0.iter() {
        debug!("global {:#?}", gl);
        let instance = Variable {
            mutable: match gl.ty.mu {
                Mu::Var => true,
                _ => false,
            },
            val: get_expr_const_ty_global(&gl.init)?, //TODO this might be a problem, because we only accept CONST and no other globals
        };

        mod_instance
            .globaladdrs
            .push(store.globals.len() as u32);
        store.globals.push(instance);
    }

    debug!("Globals in mod_i {:?}", mod_instance.globaladdrs);
    debug!("Globals in store {:#?}", store.globals);

    Ok(())
}

fn allocate_exports(
    m: &Module,
    mod_instance: &ModuleInstance,
    _store: &mut Store,
) -> Result<(), ()> {
    debug!("allocate exports");

    // Gets all exports
    let ty = validation::extract::get_exports(&m);

    for export in ty.into_iter() {
        debug!("Export {:?}", export);

        mod_instance.exports.push(export.into());
    }

    debug!("Exports in mod_i {:?}", mod_instance.exports);

    Ok(())
}

pub(crate) fn get_expr_const_ty_global(init: &[Instruction]) -> Result<Value, ()> {
    use wasm_parser::core::NumericInstructions::*;

    if init.is_empty() {
        error!("No expr to evaluate");
        return Err(());
    }

    match init.get(0).unwrap() {
        Instruction::Num(n) => match *n {
            OP_I32_CONST(v) => Ok(Value::I32(v)),
            OP_I64_CONST(v) => Ok(Value::I64(v)),
            OP_F32_CONST(v) => Ok(Value::F32(v)),
            OP_F64_CONST(v) => Ok(Value::F64(v)),
            _ => {
                error!("Expression is not a const");
                Err(())
            }
        },
        _ => {
            error!("Wrong expression");
            Err(())
        }
    }
}
