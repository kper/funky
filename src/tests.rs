use crate::allocation::allocate;
use crate::engine::*;
use insta::assert_snapshot;
use std::cell::RefCell;
use std::rc::{Rc, Weak};
use validation::validate;
use wasm_parser::{parse, read_wasm, Module};

macro_rules! test_file_module_instance {
    ($fs_name:expr) => {
        let file = read_wasm!(&format!("tests/{}", $fs_name));
        let module = parse(file).expect("Parsing failed");
        assert!(validate(&module).is_ok());

        let instance = ModuleInstance::new(&module);
        let engine = Engine::new(instance, &module);

        assert_snapshot!($fs_name, format!("{:#?}", engine));
    };
}

#[test]
fn test_allocation() {
    let module = Module {
        sections: Vec::new(),
    };

    let mut store = Store {
        funcs: Vec::new(),
        tables: Vec::new(),
        stack: Vec::new(),
        globals: Vec::new(),
        memory: Vec::new(),
    };
    let instance = ModuleInstance::new(&module);
    let rc = Rc::new(RefCell::new(instance));
    allocate(&module, &rc, &mut store);
}

#[test]
fn test_empty_wasm() {
    test_file_module_instance!("empty.wasm");
}

#[test]
fn test_return_i32() {
    test_file_module_instance!("return_i32.wasm");
}

/*
#[test]
fn test_return_i64() {
    test_file_module_instance!("return_i64.wasm");
}

#[test]
fn test_function_call() {
    test_file_module_instance!("function_call.wasm");
}

#[test]
fn test_arithmetic() {
    test_file_module_instance!("arithmetic.wasm");
}

#[test]
fn test_block_add_i32() {
    test_file_module_instance!("block_add_i32.wasm");
}

#[test]
fn test_loop_mult() {
    test_file_module_instance!("loop_mult.wasm");
}

#[test]
fn test_unreachable() {
    test_file_module_instance!("unreachable.wasm");
}

#[test]
fn test_if_loop() {
    test_file_module_instance!("if_loop.wasm");
}

#[test]
fn test_logic() {
    test_file_module_instance!("logic.wasm");
}

#[test]
fn test_gcd() {
    test_file_module_instance!("gcd.wasm");
}
*/
