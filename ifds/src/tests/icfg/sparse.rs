use crate::icfg::tabulation::sparse::TabulationSparse;
use crate::icfg::tikz::render_to;
use crate::solver::Request;
use insta::assert_snapshot;
use log::error;

use crate::grammar::*;

use crate::icfg::flowfuncs::sparse_taint::flow::SparseTaintNormalFlowFunction;
use crate::icfg::flowfuncs::sparse_taint::initial::SparseTaintInitialFlowFunction;

macro_rules! ir {
    ($name:expr, $req:expr, $ir:expr) => {
        let mut convert = TabulationSparse::new(
            SparseTaintInitialFlowFunction,
            SparseTaintNormalFlowFunction,
        );

        let prog = ProgramParser::new().parse(&$ir).unwrap();

        let res = convert.visit(&prog, &$req);

        if let Err(err) = res {
            error!("ERROR: {}", err);
            err.chain()
                .skip(1)
                .for_each(|cause| error!("because: {}", cause));
            panic!("")
        }

        let (graph, state) = res.unwrap();

        let output = render_to(&graph, &state);

        assert_snapshot!(format!("{}_dot", $name), output);
    };
}

#[test]
fn test_ir_const() {
    let req = Request {
        variable: None,
        function: "test".to_string(),
        pc: 0,
    };

    ir!(
        "test_ir_const",
        req,
        "
         define test (result 0) (define %0) {
            %0 = 1
         };
    "
    );
}

#[test]
fn test_ir_double_assign() {
    env_logger::init();
    let req = Request {
        variable: None,
        function: "test".to_string(),
        pc: 0,
    };
    ir!(
        "test_ir_double_const",
        req,
        "
         define test (result 0) (define %0 %1 %2){
            %0 = 1
            %1 = 1
            %2 = %0
            %2 = %1
         };
    "
    );
}


#[test]
fn test_ir_simple_store() {
    let req = Request {
        variable: None,
        function: "test".to_string(),
        pc: 0,
    };

    ir!(
        "test_ir_simple_store",
        req,
        "
         define test (param %0) (result 0) (define %0 %1) {
            STORE FROM %0 OFFSET 0 + %0 ALIGN 2 32
         };
    "
    );
}

#[test]
fn test_ir_simple_load() {
    let req = Request {
        variable: None,
        function: "test".to_string(),
        pc: 0,
    };

    ir!(
        "test_ir_simple_load",
        req,
        "
         define test (param %0) (result 0) (define %0 %1) {
            %1 = LOAD OFFSET 0 + %0 ALIGN 0
         };
    "
    );
}