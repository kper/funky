use crate::icfg::convert::Convert;
use crate::icfg::tikz::render_to;
use insta::assert_snapshot;

use crate::grammar::*;

macro_rules! ir {
    ($name:expr, $ir:expr) => {
        let mut convert = Convert::new();

        let prog = ProgramParser::new().parse(&$ir).unwrap();

        let graph = convert.visit(prog).unwrap();

        let output = render_to(&graph);

        assert_snapshot!(
            format!("{}_dot", $name),
            output
        );
    };
}

#[test]
fn test_ir_const() {
    ir!(
        "test_ir_const",
        "
         define test (result 0) (define %0) {
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
         define test (result 0) (define %0 %1){
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
         define test (result 0) (define %0 %1){
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
         define test (result 0) (define %0 %1) {
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
fn test_ir_killing() {
    ir!(
        "test_ir_killing",
        "define test (result 0) (define %0) {
            %0 = 1
            %0 = 2
        };"
    );
}

#[test]
fn test_ir_unop() {
    ir!(
        "test_ir_unop",
        "define test (result 0) (define %0 %1) {
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
    ir!(
        "test_ir_killing_op",
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
fn test_ir_functions() {
    ir!(
        "test_ir_functions",
        "define test (result 0) (define %0) {
            %0 = 1
            CALL mytest(%0)
        };
        define mytest (param %0) (result 0) (define %0 %1)  {
            %0 = 2   
            %1 = 3
            RETURN;
        };"
    );
}

#[test]
fn test_ir_return_values() {
    ir!(
        "test_ir_return_values",
        "define test (result 0) (define %0 %1) {
            %0 = 1
            %1 <- CALL mytest(%0)
        };
        define mytest (param %0) (result 1) (define %0 %1) {
            %0 = 2   
            %1 = 3
            RETURN %1;
        };"
    );
}

#[should_panic]
#[test]
fn test_ir_return_mismatched_values() {
    // Assigning %1 but `result 0`
    ir!(
        "test_ir_mismatched_functions",
        "define test (result 0) (define %0 %1) {
            %0 = 1
            %1 <- CALL mytest(%0)
        };
        define mytest (param %0) (result 0) (define %0 %1){
            %0 = 2   
            %1 = 3
            RETURN %0;
        };"
    );
}

#[should_panic]
#[test]
fn test_ir_return_mismatched_values2() {
    // Assigning void but `result 1`
    ir!(
        "test_ir_mismatched_functions2",
        "define test (result 0) (define %0){
            %0 = 1
            CALL mytest(%0)
        };
        define mytest (param %0) (result 1) (define %0 %1){
            %0 = 2   
            %1 = 3
            RETURN %1;
        };"
    );
}

#[test]
fn test_ir_multiple_return_values() {
    ir!(
        "test_ir_multiple_return_values",
        "define test (result 0) (define %0 %1 %2) {
            %0 = 1
            %1 %2 <- CALL mytest(%0)
        };
        define mytest (param %0) (result 2) (define %0 %1) {
            %0 = 2   
            %1 = 3
            RETURN %0 %1;
        };"
    );
}

#[test]
fn test_ir_early_return() {
    ir!(
        "test_ir_early_return",
        "define test (result 0) (define %0 %1) {
            %0 = 1
            %1 <- CALL mytest(%0)
            %0 <- CALL mytestfoo(%0)
        };
        define mytest (param %0) (result 1) (define %0 %1) {
            %0 = 2   
            RETURN %0;
            %1 = 3
            RETURN %1;
        };
        define mytestfoo (param %0) (result 1) (define %0 %1) {
            %0 = 2
            %1 = 3
            RETURN %0;
        };
        "
    );
}

#[test]
fn test_ir_return() {
    ir!(
        "test_ir_return",
        "define test (result 0) (define %0 %1) {
            %0 = 1
            %0 %1 <- CALL mytest(%0)
            %1 = 2
        };
        define mytest (param %0) (result 2) (define %0 %1) {
            %0 = 2   
            %1 = 3
            RETURN %0 %1;
        };
        "
    );
}

#[test]
fn test_ir_if_else() {
    ir!(
        "test_ir_if_else",
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
        };
        "
    );
}

#[test]
fn test_ir_if() {
    ir!(
        "test_ir_if",
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

#[test]
fn test_ir_loop() {
    ir!(
        "test_ir_loop",
        "define main (result 0) (define %0 %1) {
            BLOCK 0
            %0 = 1
            GOTO 0 
        };
        "
    );
}

#[test]
fn test_ir_table() {
    ir!(
        "test_ir_table",
        "define main (result 0) (define %0 %1 %2) {
            BLOCK 0
            %0 = 1
            %1 = 2
            %2 = 3
            BLOCK 1
            %1 = 2
            %2 = 3
            BLOCK 2
            %2 = 4
            TABLE GOTO 0 1 2 ELSE GOTO 2
        };
        "
    );
}