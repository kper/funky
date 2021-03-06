use crate::ir::wasm_ast::IR;
use funky::engine::module::ModuleInstance;
use funky::engine::*;
use insta::assert_snapshot;
use validation::validate;
use wasm_parser::{parse, read_wasm};

use crate::grammar::*;
use crate::icfg::tabulation::fast::TabulationFast;
use crate::icfg::tikz::render_to;
use crate::icfg::tikz2::render_to as render_to2;
use crate::solver::Request;

use crate::icfg::flowfuncs::taint::flow::TaintNormalFlowFunction;
use crate::icfg::flowfuncs::taint::initial::TaintInitialFlowFunction;

use crate::icfg::flowfuncs::sparse_taint::flow::SparseTaintNormalFlowFunction;
use crate::icfg::flowfuncs::sparse_taint::initial::SparseTaintInitialFlowFunction;
use crate::icfg::tabulation::sparse::TabulationSparse;

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

        //assert_snapshot!(format!("{}_ir", $name), ir_code);

        let mut convert = TabulationFast::new(TaintInitialFlowFunction, TaintNormalFlowFunction);

        let prog = ProgramParser::new().parse(&ir_code).unwrap();

        let (graph, res) = convert.visit(&prog, &$req).unwrap();

        let output = render_to(&graph, &res);

        assert_snapshot!(format!("{}", $name), output);
    };
}

macro_rules! run_sparse {
    ($name:expr, $req:expr, $fs:expr) => {
        let ir = wasm!($fs);

        let ir_code = ir.buffer();

        let mut convert = TabulationSparse::new(
            SparseTaintInitialFlowFunction,
            SparseTaintNormalFlowFunction,
        );

        let prog = ProgramParser::new().parse(&ir_code).unwrap();

        let (graph, res) = convert.visit(&prog, &$req).unwrap();

        let output = render_to2(&graph, &res);

        assert_snapshot!(format!("sparse_{}", $name), output);
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
fn test_fib_sparse() {
    let req = Request {
        function: "0".to_string(),
        variable: None,
        pc: 3,
    };

    run_sparse!("fib", req, "../tests/fib.wasm");
}

#[test]
fn test_fib_func_1() {
    let req = Request {
        function: "1".to_string(),
        variable: None,
        pc: 0,
    };

    run!("fib_func_1", req, "../tests/fib.wasm");
}

#[test]
fn test_fib_func_1_offset() {
    let req = Request {
        function: "1".to_string(),
        variable: None,
        pc: 1,
    };

    run!("fib_func_1_offset", req, "../tests/fib.wasm");
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

#[test]
fn test_block_add() {
    let req = Request {
        function: "0".to_string(),
        variable: Some("%2".to_string()),
        pc: 2,
    };

    run!("block_add", req, "../tests/block_add_i32.wasm");
}
