use crate::icfg::tabulation::naive::TabulationNaive;
use crate::icfg::tikz::render_to;
use insta::assert_snapshot;
use log::error;

use crate::grammar::*;

macro_rules! ir {
    ($name:expr, $ir:expr) => {
        let mut convert = TabulationNaive::default();

        let prog = ProgramParser::new().parse(&$ir).unwrap();

        let res = convert.visit(&prog);

        if let Err(err) = res {
            error!("ERROR: {}", err);
            err.chain()
                .skip(1)
                .for_each(|cause| error!("because: {}", cause));
            panic!("")
        }

        //write_ir($name, $ir);

        let (graph, state, _) = res.unwrap();

        let output = render_to(&graph, &state);

        assert_snapshot!(format!("{}_dot", $name), output);
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
         define test (result 0) (define %0 %1) {
            %0 = 1
            %1 = 1
         };
    "
    );
}

#[test]
fn test_ir_double_assign() {
    ir!(
        "test_ir_double_assign",
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
fn test_ir_double_assign_with_params() {
    ir!(
        "test_ir_double_const_with_params",
        "
         define test (param %0) (result 0) (define %0 %1 %2){
            %1 = 1
            %2 = %0
            %2 = %1
         };
    "
    );
}

#[test]
fn test_ir_chain_assign() {
    ir!(
        "test_ir_chain_assign",
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
fn test_ir_binop_offset() {
    ir!(
        "test_ir_binop_offset",
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
    ir!(
        "test_ir_phi",
        "define test (result 0) (define %0 %1 %2) {
            %0 = 1
            %1 = 1
            %2 = phi %0 %1
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
            %0 = %2
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
            %1 = 2
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

#[test]
fn test_global_get_and_set() {
    ir!(
        "test_ir_global_get_and_set",
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
    ir!(
        "test_ir_global_set",
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
fn test_global_get_and_set_multiple_functions() {
    ir!(
        "test_ir_global_get_and_set_multiple_functions",
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
    "
    );
}

#[test]
fn test_global_call() {
    ir!(
        "test_ir_global_call",
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
    ir!(
        "test_ir_globals_writes",
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

#[test]
fn test_global_check_order() {
    ir!(
        "test_ir_globals_check_order",
        "
        define test (result 0) (define %-2 %-1 %0 %2) {
            %0 = 1
            %-1 = %0 
            %2 <- CALL mytest()
        };
        define mytest (param) (result 1) (define %-2 %-1 %0 %1)  {
            %0 = 2   
            %1 = 3
            RETURN %-1;
        };
    "
    );
}

#[test]
fn test_ir_simple_store() {
    ir!(
        "test_ir_simple_store",
        "
         define test (param %0) (result 0) (define %0 %1) {
            STORE FROM %0 OFFSET 0 + %0 ALIGN 2 32
            %0 = 1
         };
    "
    );
}

#[test]
fn test_ir_simple_load() {
    ir!(
        "test_ir_simple_load",
        "
         define test (param %0) (result 0) (define %0 %1) {
            %1 = LOAD OFFSET 0 + %0 ALIGN 0
         };
    "
    );
}

#[test]
fn test_memory_store() {
    ir!(
        "test_ir_memory_store",
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
        STORE FROM %8 OFFSET 0 + %7 ALIGN 3 64
        RETURN ;
        };
    "
    );
}

#[test]
fn test_memory_load() {
    ir!(
        "test_ir_memory_load",
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
    ir!(
        "test_ir_memory_load_different_functions",
        "
       define 0 (result 0) (define %0 %1 %2 %3 %4 %5 %6 %7) {
        BLOCK 0
        %0 = 8
        %1 = -12345
        STORE FROM %1 OFFSET 0 + %0 ALIGN 2 32
        %2 <- CALL 1 ()
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
fn test_memory_load_different_functions2() {
    ir!(
        "test_ir_memory_load_different_functions2",
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
