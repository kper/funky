use funky::engine::module::ModuleInstance;
use funky::engine::*;
use insta::assert_snapshot;
use wasm_parser::{parse, read_wasm};
use validation::validate;
use crate::ir::wasm_ast::IR;

use crate::icfg::convert::Convert;
use crate::icfg::tikz::render_to;
use crate::grammar::*;

macro_rules! wasm {
    ($input:expr) => {{
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

        ir
    }};
}

macro_rules! run {
    ($name:expr, $fs:expr) => {

        let ir = wasm!($fs);

        let ir_code = ir.buffer();

        let mut convert = Convert::new();

        let prog = ProgramParser::new().parse(&ir_code).unwrap();

        let graph = convert.visit(&prog).unwrap();

        let output = render_to(&graph);

        assert_snapshot!(
            format!("{}", $name),
            output
        );
    };
}

#[test]
fn test_add() {
    run!("add", "../tests/add.wasm");
}

#[test]
fn test_fib() {
    run!("fib", "../tests/fib.wasm");
}

#[test]
fn test_fac() {
    run!("fac", "../tests/fac.wasm");
}