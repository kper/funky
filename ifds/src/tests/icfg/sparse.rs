use crate::icfg::tikz2::render_to;
use crate::icfg::{state::State, tabulation::sparse::TabulationSparse};
use crate::solver::Request;
use insta::assert_snapshot;
use log::error;

use crate::grammar::*;

use crate::icfg::flowfuncs::sparse_taint::flow::SparseTaintNormalFlowFunction;
use crate::icfg::flowfuncs::sparse_taint::initial::SparseTaintInitialFlowFunction;
use crate::ir::ast::Function as AstFunction;

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
            panic!("")
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

#[ignore]
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

#[ignore]
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

#[ignore]
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

#[ignore]
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

#[ignore]
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

#[ignore]
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

#[ignore]
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
        .get_scfg_graph(&"test".to_string(), &"%1".to_string())
        .unwrap();

    let output = scfg
        .edges
        .iter()
        .map(|x| format!("({}) -> ({})", x.get_from().pc, x.to().pc))
        .collect::<Vec<_>>().join("\n");

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
