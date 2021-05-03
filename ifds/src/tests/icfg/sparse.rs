use crate::icfg::tabulation::sparse::TabulationSparse;
use crate::icfg::tikz2::render_to;
use crate::solver::Request;
use insta::assert_snapshot;
use log::error;

use crate::grammar::*;

use crate::icfg::flowfuncs::sparse_taint::flow::SparseTaintNormalFlowFunction;
use crate::icfg::flowfuncs::sparse_taint::initial::SparseTaintInitialFlowFunction;

macro_rules! ir {
    ($name:expr, $req:expr, $ir:expr) => {{
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
            panic!("Macro panicked")
        }

        let (graph, state) = res.unwrap();

        let output = render_to(&graph, &state);

        assert_snapshot!(format!("{}_dot", $name), output);

        convert
    }};
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
        "define test (result 0) (define %0 %1 %2) {
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
        "define test (result 0) (define %0 %1 %2 %3) {
            %0 = 1
            %1 = 1
            %2 = %0 op %1
            %3 = %1 op %0   
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
fn test_ir_if_else_binop() {
    let req = Request {
        variable: None,
        function: "test".to_string(),
        pc: 0,
    };
    let tabulation = ir!(
        "test_ir_if_else_binop",
        req,
        "define test (result 0) (define %0 %1 %2 %3) {
            %0 = 1
            IF %1 THEN GOTO 1 ELSE GOTO 2 
            BLOCK 1
            %1 = %0
            %2 = 3
            GOTO 3
            BLOCK 2
            %1 = 1
            GOTO 3
            BLOCK 3
            %3 = %1 op %0   
        };"
    );

    let scfg = tabulation
        .get_scfg_graph(&"test".to_string(), &"%0".to_string())
        .unwrap();

    let output = scfg
        .edges
        .iter()
        .map(|x| format!("({}) -> ({})", x.get_from().pc, x.to().pc))
        .collect::<Vec<_>>()
        .join("\n");

    assert_snapshot!(format!("{}_scfg_dot", "test_ir_if_else_binop"), output);
}

#[test]
fn test_ir_if_else() {
    let req = Request {
        variable: None,
        function: "main".to_string(),
        pc: 1,
    };
    ir!(
        "test_ir_if_else",
        req,
        "define main (result 0) (define %0 %1 %2 %3) {
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
            %3 = %2
        };
        "
    );
}

#[test]
fn test_ir_if_else_offset() {
    // This test checks how the sparse algorithm behaves
    // when the analysis starts in a branch.
    let req = Request {
        variable: None,
        function: "main".to_string(),
        pc: 4,
    };
    ir!(
        "test_ir_if_else_offset",
        req,
        "define main (result 0) (define %0 %1 %2 %3) {
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
            %3 = %1
        };
        "
    );
}

#[test]
fn test_ir_if() {
    let req = Request {
        variable: None,
        function: "main".to_string(),
        pc: 1,
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
        pc: 1,
    };
    ir!(
        "test_ir_loop",
        req,
        "define main (result 0) (define %0 %1) {
            BLOCK 0
            %0 = 1
            %1 = 2
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
        pc: 1,
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
    let tabulation = ir!(
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

    let scfg = tabulation
        .get_scfg_graph(&"test".to_string(), &"taut".to_string())
        .unwrap();

    let output = scfg
        .edges
        .iter()
        .map(|x| format!("({}) -> ({})", x.get_from().pc, x.to().pc))
        .collect::<Vec<_>>()
        .join("\n");

    assert_snapshot!(format!("{}_scfg_dot", "test_ir_function"), output);
}

#[test]
fn test_ir_multiple_functions() {
    let req = Request {
        variable: None,
        function: "test".to_string(),
        pc: 0,
    };
    ir!(
        "test_ir_multiple_functions",
        req,
        "define test (result 0) (define %0 %1 %2) {
            %0 = 1
            %1 <- CALL mytest(%0)
            %2 <- CALL mytest(%0)
        };
        define mytest (param %0) (result 1) (define %0)  {
            RETURN %0;
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
        "define test (result 0) (define %0 %1) {
            %0 = 1
            %1 <- CALL mytest(%0)
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
fn test_ir_early_return() {
    let req = Request {
        variable: None,
        function: "test".to_string(),
        pc: 0,
    };

    let tabulation = ir!(
        "test_ir_early_return",
        req,
        "define test (result 0) (define %0 %1 %2 %3) {
            %0 = 1
            %1 <- CALL mytest(%0)
            %2 <- CALL mytestfoo(%0)
            %3 <- CALL mytestfoo(%0)
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
    let scfg = tabulation
        .get_scfg_graph(&"test".to_string(), &"%0".to_string())
        .unwrap();

    let output = scfg
        .edges
        .iter()
        .map(|x| format!("({}) -> ({})", x.get_from().pc, x.to().pc))
        .collect::<Vec<_>>()
        .join("\n");

    assert_snapshot!(format!("{}_scfg_dot", "test_ir_early_return"), output);
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
        "define test (result 0) (define %0 %1 %2) {
            %0 = 1
            %1 %2 <- CALL mytest(%0)
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
    ir!(
        "test_ir_global_get_and_set",
        req,
        "
        define 0 (param %0) (result 0) (define %-2 %-1 %0 %1) {
        %1 = %-1
        %-2 = %1
        };
    "
    );
}

#[test]
fn test_global_set() {
    let req = Request {
        variable: None,
        function: "0".to_string(),
        pc: 0,
    };
    ir!(
        "test_ir_global_set",
        req,
        "
        define 0 (param %0) (result 0) (define %-1 %0 %1) {
            %0 = 1
            %-1 = %0
            %1 <- CALL 1()
        };

        define 1 (param) (result 1) (define %-1 %0) {
            %0 = %-1
            RETURN %0;
        };
    "
    );
}

#[test]
fn test_ir_initial_function() {
    let req = Request {
        variable: None,
        function: "test".to_string(),
        pc: 1,
    };

    ir!(
        "test_ir_initial_function",
        req,
        "define test (result 0) (define %0 %1 %2) {
            %0 = 1
            %1 <- CALL mytest(%0)
            %2 = %1 op %0   
        };
        define mytest (param %0) (result 1) (define %0)  {
            %0 = 2   
            RETURN %0;
        };"
    );
}

#[test]
fn test_global_get_and_set_multiple_functions() {
    let req = Request {
        variable: None,
        function: "0".to_string(),
        pc: 0,
    };
    ir!(
        "test_ir_global_get_and_set_multiple_functions",
        req,
        "
        define 0 (param %0) (result 0) (define %-2 %-1 %0 %1 %2) {
        %1 = %-1
        %-2 = %1
        %2 = 1
        %0 <- CALL 1 (%2)
        %1 <- CALL 2 ()
        };

        define 1 (param %0) (result 1) (define %-2 %-1 %0) {
        %0 = %-2
        %-1 = 1
        RETURN %0;
        };

        define 2 (result 1) (define %-2 %-1 %0) {
        %0 = 1
        %-2 = 1
        RETURN %0;
        };
    "
    );
}

#[test]
fn test_memory_store() {
    let req = Request {
        variable: None,
        function: "0".to_string(),
        pc: 1,
    };
    ir!(
        "test_ir_memory_store",
        req,
        "
        define 0 (result 0) (define %0 %1 %2 %3 %4 %5 %6 %7 %8 %9) {
        BLOCK 0
        %1 = -12345
        STORE FROM %1 OFFSET 0 + %0 ALIGN 2 32
        %2 = 8
        %3 = -12345
        STORE FROM %3 OFFSET 0 + %2 ALIGN 3 64
        %5 = 8
        %6 = -12345
        STORE FROM %6 OFFSET 0 + %5 ALIGN 2 32
        %7 = 8
        %8 = -12345
        STORE FROM %8 OFFSET 1 + %7 ALIGN 3 64
        RETURN ;
        };
    "
    );
}

#[test]
fn test_memory_store_offset() {
    let req = Request {
        variable: None,
        function: "0".to_string(),
        pc: 2,
    };
    ir!(
        "test_ir_memory_store_offset",
        req,
        "
        define 0 (result 0) (define %0 %1 %2 %3 %4 %5 %6 %7 %8 %9) {
        BLOCK 0
        %1 = -12345
        STORE FROM %1 OFFSET 0 + %0 ALIGN 2 32
        %2 = 8
        %3 = -12345
        STORE FROM %3 OFFSET 0 + %2 ALIGN 3 64
        %5 = 8
        %6 = -12345
        STORE FROM %6 OFFSET 0 + %5 ALIGN 2 32
        %7 = 8
        %8 = -12345
        STORE FROM %8 OFFSET 1 + %7 ALIGN 3 64
        RETURN ;
        };
    "
    );
}

#[test]
fn test_memory_load() {
    let req = Request {
        variable: None,
        function: "0".to_string(),
        pc: 2,
    };
    ir!(
        "test_ir_memory_load",
        req,
        "
       define 0 (result 0) (define %0 %1 %2 %3 %4 %5 %6 %7) {
        BLOCK 0
        %0 = 8
        %1 = -12345
        STORE FROM %1 OFFSET 0 + %0 ALIGN 2 32
        %4 = 8
        %5 = LOAD OFFSET 0 + %4 ALIGN 0
        %6 = 8
        %7 = LOAD OFFSET 0 + %6 ALIGN 0
        KILL %7
        KILL %6
        RETURN ;
       };
    "
    );
}

#[test]
fn test_memory_load_different_functions() {
    let req = Request {
        variable: None,
        function: "0".to_string(),
        pc: 2,
    };
    ir!(
        "test_ir_memory_load_different_functions",
        req,
        "
       define 0 (result 0) (define %0 %1 %2 %3 %4 %5 %6 %7) {
        BLOCK 0
        %0 = 8
        %1 = -12345
        %2 <- CALL 1 (%1)
        %3 = LOAD OFFSET 0 + %1 ALIGN 0
        RETURN ;
       }; 

       define 1 (param %0) (result 1) (define %0 %1) {
        STORE FROM %0 OFFSET 0 + %0 ALIGN 2 32
        %1 = LOAD OFFSET 0 + %0 ALIGN 0
        RETURN %1;
       };
    "
    );
}

#[test]
fn test_memory_load_different_functions2() {
    let req = Request {
        variable: None,
        function: "0".to_string(),
        pc: 2,
    };
    ir!(
        "test_ir_memory_load_different_functions2",
        req,
        "
       define 0 (result 0) (define %0 %1 %2 %3 %4 %5 %6 %7) {
        BLOCK 0
        %0 = 8
        %1 = -12345
        STORE FROM %1 OFFSET 0 + %0 ALIGN 2 32
        %2 <- CALL 1 ()
        STORE FROM %1 OFFSET 1 + %0 ALIGN 2 32
        %3 <- CALL 1 ()
        RETURN ;
       }; 

       define 1 (result 1) (define %0 %1) {
        %1 = 8
        %0 = LOAD OFFSET 0 + %1 ALIGN 0
        RETURN %0;
       };
    "
    );
}

#[test]
fn test_global_call() {
    let req = Request {
        variable: None,
        function: "1".to_string(),
        pc: 1,
    };
    ir!(
        "test_ir_global_call",
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
    "
    );
}

#[test]
fn test_global_writes() {
    let req = Request {
        variable: None,
        function: "test".to_string(),
        pc: 0,
    };
    ir!(
        "test_ir_globals_writes",
        req,
        "
        define test (result 0) (define %-1 %0 %2) {
            %0 = 1
            %-1 = %0 
            %2 <- CALL mytest()
        };
        define mytest (param) (result 1) (define %-1 %0 %1)  {
            %0 = 2   
            %1 = 3
            RETURN %-1;
        };
    "
    );
}
