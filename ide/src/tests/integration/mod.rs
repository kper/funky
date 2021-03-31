use crate::ir::wasm_ast::IR;
use funky::engine::module::ModuleInstance;
use funky::engine::*;
use insta::assert_snapshot;
use validation::validate;
use wasm_parser::{parse, read_wasm};

use crate::grammar::*;
use crate::icfg::convert2::ConvertSummary;
use crate::icfg::tikz::render_to;
use crate::solver::Request;

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

        let mut convert = ConvertSummary::new();

        let prog = ProgramParser::new().parse(&ir_code).unwrap();

        let req = Request {
            function: "0".to_string(),
            variable: None,
            pc: 0,
        };

        let graph = convert.visit(&prog, &req).unwrap();

        let output = render_to(&graph);

        assert_snapshot!(format!("{}", $name), output);
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

#[test]
fn test_br_table() {
    run!("br_table", "../tests/br_table.wasm");
}
