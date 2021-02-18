use crate::engine::module::ModuleInstance;
use crate::engine::stack::Frame;
use crate::engine::stack::CtrlStackContent;
use crate::engine::store::Store;
use crate::value::Value;
use anyhow::{anyhow, Context, Result};
use wasm_parser::core::FuncAddr;
use wasm_parser::Module;

type StartFunctionAddr = FuncAddr;

/// Returns the addr of the start function, which needs to be invoked
pub fn instantiation(
    m: &Module,
    mod_instance: &ModuleInstance,
    store: &mut Store,
) -> Result<Option<StartFunctionAddr>> {
    // Step 1

    // Module is already valid, because checked before

    // ... skip to Step 7 TODO

    // Step 7

    let frame = Frame {
        locals: Vec::new(),
        arity: 0,
        //module_instance: Rc::downgrade(&mod_instance),
    };

    // Step 8

    store.ctrl_stack.push(CtrlStackContent::Frame(frame));

    // Step 9 and Step 13
    if let Err(err) = instantiate_elements(m, mod_instance, store) {
        return Err(anyhow!("{}", err));
    }

    // Step 10 and Step 14
    if let Err(err) = instantiate_data(m, mod_instance, store) {
        return Err(anyhow!("{}", err));
    }

    // Step 11 and 12
    if let Some(CtrlStackContent::Frame(f)) = store.ctrl_stack.pop() {
        let frame = Frame {
            locals: Vec::new(),
            arity: 0,
            //module_instance: Rc::downgrade(&mod_instance),
        };

        assert_eq!(frame, f);
    } else {
        return Err(anyhow!("No frame on the stack"));
    }

    // Step 15

    let start_func = instantiate_start(m, mod_instance, store)?;

    Ok(start_func)
}

fn instantiate_elements(
    m: &Module,
    mod_instance: &ModuleInstance,
    store: &mut Store,
) -> Result<()> {
    debug!("instantiate elements");

    let ty = validation::extract::get_elements(&m);

    info!("Module has {} elements defined", ty.len());

    for e in ty.iter() {
        debug!("Instantiate element {:?}", e.offset);
        let eoval = crate::allocation::get_expr_const_ty_global(&e.offset, mod_instance, store)
            .map_err(|_| anyhow!("Fetching const expr failed"))?;

        let table_index = e.table as i32;
        debug!("=> element's table_index {}", table_index);

        if let Value::I32(eo) = eoval {
            debug!("Assertion correct: Element's offset is I32({})", eo);

            let borrow = &mod_instance;

            let table_addr = borrow
                .tableaddrs
                .get(table_index as usize)
                .ok_or_else(|| anyhow!("Table index {} does not exists", table_index))?;

            let table_inst = store
                .tables
                .get_mut(*table_addr as usize)
                .ok_or_else(|| anyhow!("Table addr {:?} does not exists", table_addr))?;

            let eend = table_index + e.init.len() as i32;

            if eend > table_inst.elem.len() as i32 {
                return Err(anyhow!("end is larger than table_inst.elem"));
            }

            // Step 13

            for (j, funcindex) in e.init.iter().enumerate() {
                use std::mem::replace;

                debug!("Updating function's addr in table");

                let funcaddr = borrow
                    .funcaddrs
                    .get(*funcindex as usize)
                    .ok_or_else(|| anyhow!("No function with funcindex"))?;

                debug!("=> Updating for function {:?}", funcaddr);

                let _ = replace(
                    &mut table_inst.elem[eo as usize + j],
                    Some(funcaddr.clone()),
                );
            }
        } else {
            panic!("Assertion failed. Element's offset is not I32");
        }
    }

    debug!("Updated tables in store {:#?}", store.tables);

    Ok(())
}

fn instantiate_data(m: &Module, mod_instance: &ModuleInstance, store: &mut Store) -> Result<()> {
    debug!("instantiate elements");

    let ty = validation::extract::get_data(&m);

    for data in ty.iter() {
        debug!("data offset {:?}", data.offset);

        let doval = crate::allocation::get_expr_const_ty_global(&data.offset, mod_instance, store)
            .map_err(|_| anyhow!("Fetching const expr failed"))?;

        if let Value::I32(mem_idx) = doval {
            debug!("Memory index is {}", mem_idx);

            //mem_idx = do_i
            let borrow = &mod_instance;
            let mem_addr = borrow
                .memaddrs
                .get(0)
                .ok_or_else(|| anyhow!("Memory index does not exists"))?;

            debug!("Memory addr is {}", mem_addr);

            let mem_inst = store
                .memory
                .get_mut(*mem_addr as usize)
                .ok_or_else(|| anyhow!("Memory addr does not exists"))?;

            let dend = mem_idx + data.init.len() as i32;

            if dend > mem_inst.data.len() as i32 {
                return Err(anyhow!("dend is larger than mem_inst.data"));
            }

            // Step 14

            use std::mem::replace;
            for (j, b) in data.init.iter().enumerate() {
                let _ = replace(&mut mem_inst.data[mem_idx as usize + j], *b);
            }
        }
    }

    Ok(())
}

fn instantiate_start(
    m: &Module,
    mod_instance: &ModuleInstance,
    store: &mut Store,
) -> Result<Option<StartFunctionAddr>> {
    debug!("instantiate start");

    if let Some(start_section) = validation::extract::get_start(m).first() {
        debug!("Start section {:#?}", start_section);

        let borrow = &mod_instance;
        let func_addr = borrow
            .funcaddrs
            .get(start_section.index as usize)
            .ok_or_else(|| anyhow!("Start function addr was not found"))?;

        // Check if the functions really exists
        store
            .get_func_instance(&func_addr)
            .context("Checking if start function exists failed")?;

        return Ok(Some(func_addr.clone()));
    } else {
        debug!("No start section");
    }

    Ok(None)
}
