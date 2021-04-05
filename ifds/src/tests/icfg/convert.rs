use crate::icfg::convert::ConvertSummary;
use crate::icfg::tikz::render_to;
use crate::solver::Request;
use insta::assert_snapshot;
use log::error;

use crate::grammar::*;

macro_rules! ir {
    ($name:expr, $req:expr, $ir:expr) => {
        let mut convert = ConvertSummary::new();

        let prog = ProgramParser::new().parse(&$ir).unwrap();

        let graph = convert.visit(&prog, &$req);

        if let Err(err) = graph {
                error!("ERROR: {}", err);
                err.chain()
                    .skip(1)
                    .for_each(|cause| error!("because: {}", cause));
                panic!("")
            }

        let output = render_to(&graph.unwrap());

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
fn test_ir_chain_assign() {
    let req = Request {
        variable: None,
        function: "test".to_string(),
        pc: 0,
    };
    ir!(
        "test_ir_chain_assign",
        req,
        "
         define test (result 0) (define %0 %1 %2 %3){
            %0 = 1
            %1 = 1
            %2 = %0
            %3 = %2
         };
    "
    );
}

#[test]
fn test_ir_unop() {
    let req = Request {
        variable: None,
        function: "test".to_string(),
        pc: 0,
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
        variable: None,
        function: "test".to_string(),
        pc: 0,
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
fn test_ir_binop_offset() {
    let req = Request {
        variable: None,
        function: "test".to_string(),
        pc: 1,
    };
    ir!(
        "test_ir_binop_offset",
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
fn test_ir_phi() {
    let req = Request {
        variable: None,
        function: "test".to_string(),
        pc: 0,
    };
    ir!(
        "test_ir_phi",
        req,
        "define test (result 0) (define %0 %1 %2) {
            %0 = 1
            %1 = 1
            %2 = phi %0 %1
        };"
    );
}

#[test]
fn test_ir_killing_op() {
    let req = Request {
        variable: None,
        function: "test".to_string(),
        pc: 0,
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
        variable: None,
        function: "test".to_string(),
        pc: 0,
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
    let req = Request {
        variable: None,
        function: "main".to_string(),
        pc: 0,
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
        variable: None,
        function: "main".to_string(),
        pc: 0,
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

#[test]
fn test_ir_loop() {
    let req = Request {
        variable: None,
        function: "main".to_string(),
        pc: 0,
    };
    ir!(
        "test_ir_loop",
        req,
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
    let req = Request {
        variable: None,
        function: "main".to_string(),
        pc: 0,
    };
    ir!(
        "test_ir_table",
        req,
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

#[test]
fn test_ir_functions() {
    let req = Request {
        variable: None,
        function: "test".to_string(),
        pc: 0,
    };
    ir!(
        "test_ir_functions",
        req,
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
fn test_ir_functions_rename_reg() {
    let req = Request {
        variable: None,
        function: "test".to_string(),
        pc: 0,
    };
    ir!(
        "test_ir_functions_rename_regs",
        req,
        "define test (result 0) (define %0) {
            %0 = 1
            %0 <- CALL mytest(%0)
        };
        define mytest (param %5) (result 1) (define %5 %6)  {
            %6 = %5
            RETURN %6;
        };"
    );
}

#[test]
fn test_ir_return_values() {
    let req = Request {
        variable: None,
        function: "test".to_string(),
        pc: 0,
    };
    ir!(
        "test_ir_return_values",
        req,
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

#[test]
fn test_ir_return_passed_value() {
    let req = Request {
        variable: None,
        function: "test".to_string(),
        pc: 0,
    };
    ir!(
        "test_ir_return_passed_value",
        req,
        "define test (result 0) (define %0 %1) {
            %0 = 1
            %1 <- CALL mytest(%0)
        };
        define mytest (param %0) (result 1) (define %0 %1) {
            RETURN %0;
        };"
    );
}

#[test]
fn test_ir_return_values2() {
    let req = Request {
        variable: None,
        function: "test".to_string(),
        pc: 0,
    };
    ir!(
        "test_ir_return_values2",
        req,
        "define test (result 0) (define %0 %1 %2) {
            %0 = 1
            %1 = 2
            %2 <- CALL mytest(%0)
        };
        define mytest (param %0) (result 1) (define %0 %1) {
            %1 = 3
            RETURN %0;
        };"
    );
}

#[test]
fn test_ir_return_values3() {
    let req = Request {
        variable: None,
        function: "test".to_string(),
        pc: 0,
    };
    ir!(
        "test_ir_return_values3",
        req,
        "define test (result 0) (define %0 %1 %2) {
            %0 = 1
            %1 = 2
            %2 <- CALL mytest(%0)
        };
        define mytest (param %0) (result 1) (define %0 %1) {
            RETURN %0;
        };"
    );
}

#[test]
fn test_ir_overwrite_return_values() {
    let req = Request {
        variable: None,
        function: "test".to_string(),
        pc: 0,
    };
    ir!(
        "test_ir_overwrite_return_values",
        req,
        "define test (result 0) (define %0) {
            %0 = 1
            %0 <- CALL mytest(%0)
        };
        define mytest (param %0) (result 1) (define %0 %1) {
            %0 = 2   
            %1 = 3
            RETURN %1;
        };"
    );
}

#[test]
fn test_ir_early_return() {
    let req = Request {
        variable: None,
        function: "test".to_string(),
        pc: 0,
    };
    ir!(
        "test_ir_early_return",
        req,
        "define test (result 0) (define %0 %1 %2) {
            %0 = 1
            %1 <- CALL mytest(%0)
            %0 <- CALL mytestfoo(%0)
            %1 <- CALL mytestfoo(%0)
        };
        define mytest (param %0) (result 1) (define %0 %1) {
            %0 = 2   
            %1 = 3
            RETURN %1;
        };
        define mytestfoo (param %0) (result 1) (define %0 %1) {
            %1 = 3
            RETURN %0;
        };
        "
    );
}

#[test]
fn test_ir_return_double() {
    let req = Request {
        variable: None,
        function: "test".to_string(),
        pc: 0,
    };
    ir!(
        "test_ir_return",
        req,
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
fn test_ir_return_branches() {
    let req = Request {
        variable: None,
        function: "test".to_string(),
        pc: 0,
    };
    ir!(
        "test_ir_return_branches",
        req,
        "define test (result 0) (define %0 %1) {
            %0 = 5
            %1 <- CALL mytest(%0)
        };
        define mytest (param %0) (result 1) (define %0 %1 %2) {
            %1 = 1
            IF %1 THEN GOTO 1 ELSE GOTO 2 
            BLOCK 1
            %1 = 2
            %2 = 3
            RETURN %1;
            GOTO 3
            BLOCK 2
            %2 = 4
            GOTO 3
            BLOCK 3
            RETURN %0;
        };
        "
    );
}

#[test]
fn test_ir_self_loop() {
    let req = Request {
        variable: None,
        function: "test".to_string(),
        pc: 1,
    };
    ir!(
        "test_ir_self_loop",
        req,
        "define test (param %0) (result 1) (define %0 %1 %2) {
            %2 = 1
            %0 = 5
            %1 <- CALL test(%0)
            %0 = %1
            RETURN %0;
        };
        "
    );
}

#[test]
fn test_global_get_and_set() {
     let req = Request {
        variable: None,
        function: "0".to_string(),
        pc: 0,
    };
    ir!("test_ir_global_get_and_set", 
    req,
    "
        define 0 (param %0) (result 0) (define %-2 %-1 %0 %1) {
        %1 = %-1
        %-2 = %1
        };
    ");
}

#[test]
fn test_global_get_and_set_multiple_functions() {
     let req = Request {
        variable: None,
        function: "0".to_string(),
        pc: 0,
    };
    ir!("test_ir_global_get_and_set_multiple_functions", 
    req,
    "
        define 0 (param %0) (result 0) (define %-2 %-1 %0 %1 %2) {
        %1 = %-1
        %-2 = %1
        %2 = 1
        %0 <- CALL 1 (%2)
        %1 <- CALL 2 ()
        };

        define 1 (param %0) (result 1) (define %-2 %0) {
        %0 = %-2
        RETURN %0;
        };

        define 2 (result 1) (define %-2 %0) {
        %0 = 1
        %-2 = 1
        RETURN %0;
        };
    ");
}

#[ignore]
#[test]
fn test_global_call() {
     let req = Request {
        variable: None,
        function: "0".to_string(),
        pc: 0,
    };
    ir!("test_ir_global_call", 
    req,
    "
        define 0 (param %0) (result 0) (define %-2 %-1 %0 %1) {
        BLOCK 0
        %1 = %0
        %-1 = %1
        RETURN ;
        };
        define 1 (param %0) (result 0) (define %-2 %-1 %0 %1 %2) {
        BLOCK 1
        %1 = 1
        CALL 0(%1)
        %2 = %0
        %-2 = %2
        RETURN ;
        };
    ");
}