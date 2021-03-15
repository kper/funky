use crate::icfg::convert::Convert;
//use crate::icfg::graphviz::render_to;
use insta::assert_snapshot;
use std::io::Cursor;

use crate::grammar::*;

macro_rules! ir {
    ($name:expr, $ir:expr) => {
        let mut convert = Convert::new();

        let prog = ProgramParser::new().parse(&$ir).unwrap();

        let res = convert.visit(prog).unwrap();

        //let mut dot = Cursor::new(Vec::new());
        //render_to(&res, &mut dot);

        /* 
        assert_snapshot!(
            format!("{}_dot", $name),
            std::str::from_utf8(dot.get_ref()).unwrap()
        );*/
    };
}

#[test]
fn test_ir_const() {
    ir!(
        "test_ir_const",
        "
         define test (result 0) {
            %0 = 1
         };
    "
    );
}

#[test]
fn test_ir_double_const() {
    ir!(
        "test_ir_double_const",
        "
         define test (result 0) {
            %0 = 1
            %1 = 1
         };
    "
    );
}

#[test]
fn test_ir_assignment() {
    ir!(
        "test_ir_assignment",
        "
         define test (result 0) {
            %1 = 1
            %0 = %1
         };
    "
    );
}

#[test]
fn test_ir_double_assignment() {
    ir!(
        "test_ir_double_assignment",
        "
         define test (result 0) {
            %1 = 1
            %0 = %1
            %0 = %1
         };
    "
    );
}

#[test]
fn test_ir_block() {
    ir!(
        "test_ir_block",
        "define test (result 0) {
            BLOCK 0
            %0 = 1
            GOTO 1
            BLOCK 1
            %1 = 2
        };"
    );
}

#[test]
fn test_ir_killing() {
    ir!(
        "test_ir_killing",
        "define test (result 0) {
            %0 = 1
            %0 = 2
        };"
    );
}

#[test]
fn test_ir_unop() {
    ir!(
        "test_ir_unop",
        "define test (result 0)  {
            %0 = 1
            %1 = op %0
            %1 = op %0   
        };"
    );
}

#[test]
fn test_ir_binop() {
    ir!(
        "test_ir_binop",
        "define test (result 0)  {
            %0 = 1
            %1 = 1
            %2 = %0 op %1
            %2 = %1 op %0   
        };"
    );
}

#[test]
fn test_ir_killing_op() {
    ir!(
        "test_ir_killing_op",
        "define test (result 0)  {
            %0 = 1
            %1 = 1
            KILL %0
            KILL %1
            %2 = 1
        };"
    );
}

#[test]
fn test_ir_functions() {
    ir!(
        "test_ir_functions",
        "define test (result 0) {
            %0 = 1
            CALL mytest(%0)
        };
        define mytest (param %0) (result 0)  {
            %0 = 2   
            %1 = 3
        };"
    );
}

#[test]
fn test_ir_return_values() {
    ir!(
        "test_ir_return_values",
        "define test (result 0) {
            %0 = 1
            %1 <- CALL mytest(%0)
        };
        define mytest (param %0) (result 1) {
            %0 = 2   
            %1 = 3
        };"
    );
}

#[should_panic]
#[test]
fn test_ir_return_mismatched_values() {
    // Assigning %1 but `result 0`
    ir!(
        "test_ir_mismatched_functions",
        "define test (result 0) {
            %0 = 1
            %1 <- CALL mytest(%0)
        };
        define mytest (param %0) (result 0) {
            %0 = 2   
            %1 = 3
        };"
    );
}

#[should_panic]
#[test]
fn test_ir_return_mismatched_values2() {
    // Assigning void but `result 1`
    ir!(
        "test_ir_mismatched_functions",
        "define test (result 0) {
            %0 = 1
            CALL mytest(%0)
        };
        define mytest (param %0) (result 1) {
            %0 = 2   
            %1 = 3
        };"
    );
}
