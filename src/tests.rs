use crate::engine::*;
use insta::assert_snapshot;
use validation::validate;
use wasm_parser::parse;
use wasm_parser::read_wasm;

macro_rules! test_file_module_instance {
    ($fs_name:expr) => {
        let file = read_wasm!(&format!("tests/{}", $fs_name));
        let module = parse(file).expect("Parsing failed");
        assert!(validate(&module).is_ok());

        let instance = ModuleInstance::new(module);

        assert_snapshot!($fs_name, format!("{:#?}", instance));
    };
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
