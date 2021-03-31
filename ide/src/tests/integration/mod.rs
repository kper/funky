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
    ($name:expr, $req:expr, $fs:expr) => {
        let ir = wasm!($fs);

        let ir_code = ir.buffer();

        let mut convert = ConvertSummary::new();

        let prog = ProgramParser::new().parse(&ir_code).unwrap();

        let graph = convert.visit(&prog, &$req).unwrap();

        let output = render_to(&graph);

        assert_snapshot!(format!("{}", $name), output);
    };
}

#[test]
fn test_add() {
    let req = Request {
        function: "0".to_string(),
        variable: None,
        pc: 0,
    };

    run!("add", req, "../tests/add.wasm");
}

#[test]
fn test_fib() {
    let req = Request {
        function: "0".to_string(),
        variable: None,
        pc: 0,
    };

    run!("fib", req, "../tests/fib.wasm");
}

#[test]
fn test_fac() {
    let req = Request {
        function: "0".to_string(),
        variable: None,
        pc: 0,
    };

    run!("fac", req, "../tests/fac.wasm");
}

#[test]
fn test_br_table() {
    let req = Request {
        function: "0".to_string(),
        variable: None,
        pc: 0,
    };

    run!("br_table", req, "../tests/br_table.wasm");
}
