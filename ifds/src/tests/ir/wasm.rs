use crate::ir::wasm_ast::IR;
use funky::engine::module::ModuleInstance;
use funky::engine::*;
use validation::validate;
use wasm_parser::{parse, read_wasm};

macro_rules! wasm {
    ($name:expr, $input:expr) => {{
        // Read it
        let file = read_wasm!($input);
        let module = parse(file).expect("Parsing failed");
        assert!(validate(&module).is_ok());

        let imports = Vec::new();

        let instance = ModuleInstance::new(&module);
        let engine = Engine::new(
            instance,
            &module,
            Box::new(funky::debugger::RelativeProgramCounter::default()),
            &imports,
        )
        .unwrap();

        let mut ir = IR::new();

        ir.visit(&engine).unwrap();

        //assert_snapshot!($name, format!("{}", ir.buffer()));

        ir
    }};
}

#[test]
fn test_wasi() {
    wasm!("wasi_test", "./../tests/wasi_test.wasm");
}
