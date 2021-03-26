use std::env::var;

use crate::icfg::convert2::ConvertSummary;
use crate::icfg::tikz::render_to;
use crate::solver::Request;
use insta::assert_snapshot;

use crate::grammar::*;

macro_rules! ir {
    ($name:expr, $req:expr, $ir:expr) => {
        let mut convert = ConvertSummary::new();

        let prog = ProgramParser::new().parse(&$ir).unwrap();

        let graph = convert.visit(&prog, &$req).unwrap();

        let output = render_to(&graph);

        assert_snapshot!(format!("{}_dot", $name), output);
    };
}

#[test]
fn test_ir_const() {
    let req = Request {
        variable: "%0".to_string(),
        function: "test".to_string(),
        pc: 1,
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
    let req = Request {
        variable: "%2".to_string(),
        function: "test".to_string(),
        pc: 3,
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
fn test_ir_unop() {
    let req = Request {
        variable: "%0".to_string(),
        function: "test".to_string(),
        pc: 1,
    };
    ir!(
        "test_ir_unop",
        req,
        "define test (result 0) (define %0 %1) {
            %0 = 1
            %1 = op %0
            %1 = op %0   
        };"
    );
}

#[test]
fn test_ir_binop() {
    let req = Request {
        variable: "%0".to_string(),
        function: "test".to_string(),
        pc: 1,
    };
    ir!(
        "test_ir_binop",
        req,
        "define test (result 0) (define %0 %1 %2) {
            %0 = 1
            %1 = 1
            %2 = %0 op %1
            %2 = %1 op %0   
        };"
    );
}


#[test]
fn test_ir_killing_op() {
    let req = Request {
        variable: "%0".to_string(),
        function: "test".to_string(),
        pc: 1,
    };
    ir!(
        "test_ir_killing_op",
        req,
        "define test (result 0) (define %0 %1 %2)  {
            %0 = 1
            %1 = 1
            KILL %0
            KILL %1
            %2 = 1
        };"
    );
}

#[test]
fn test_ir_block() {
    let req = Request {
        variable: "%0".to_string(),
        function: "test".to_string(),
        pc: 2,
    };
    ir!(
        "test_ir_block",
        req,
        "define test (result 0) (define %0 %1) {
            BLOCK 0
            %0 = 1
            GOTO 1
            BLOCK 1
            %1 = 2
        };"
    );
}

#[test]
fn test_ir_if_else() {
    env_logger::init();
    let req = Request {
        variable: "%0".to_string(),
        function: "main".to_string(),
        pc: 2,
    };
    ir!(
        "test_ir_if_else",
        req,
        "define main (result 0) (define %0 %1 %2) {
            BLOCK 0
            %0 = 1
            IF %1 THEN GOTO 1 ELSE GOTO 2 
            BLOCK 1
            %1 = 2
            %2 = 3
            GOTO 3
            BLOCK 2
            %2 = 4
            GOTO 3
            BLOCK 3
            %0 = %2
        };
        "
    );
}

#[test]
fn test_ir_if() {
    let req = Request {
        variable: "%0".to_string(),
        function: "main".to_string(),
        pc: 2,
    };
    ir!(
        "test_ir_if",
        req,
        "define main (result 0) (define %0 %1 %2) {
            BLOCK 0
            %0 = 1
            IF %1 THEN GOTO 0
            %1 = 2
            %2 = 3
        };
        "
    );
}

