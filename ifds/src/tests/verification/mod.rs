use itertools::Itertools;

use log::debug;

use crate::grammar::*;
use crate::icfg::tabulation::fast::TabulationFast;
use crate::icfg::tabulation::naive::TabulationNaive;
use crate::icfg::tabulation::orig::TabulationOriginal;
use crate::icfg::tabulation::sparse::TabulationSparse;

use std::collections::HashSet;

use crate::icfg::flowfuncs::sparse_taint::flow::SparseTaintNormalFlowFunction;
use crate::icfg::flowfuncs::sparse_taint::initial::SparseTaintInitialFlowFunction;
use crate::icfg::flowfuncs::taint::flow::TaintNormalFlowFunction;
use crate::icfg::flowfuncs::taint::initial::TaintInitialFlowFunction;
use crate::icfg::tikz::render_to;
use crate::icfg::tikz2::render_to as render_to2;
use insta::assert_snapshot;
use pretty_assertions::assert_eq;

use crate::solver::bfs::*;
use crate::solver::*;

macro_rules! ir {
    ($ir:expr) => {{
        let prog = ProgramParser::new().parse(&$ir).unwrap();
        prog
    }};
}

macro_rules! naive {
    ($name:expr, $prog:expr, $req:expr) => {{
        let mut convert = TabulationNaive::default();

        let (mut graph, state, _) = convert.visit(&$prog).unwrap();

        let output = render_to(&graph, &state);

        assert_snapshot!(format!("_naive_{}_dot", $name), output);

        let mut solver = Bfs;
        let mut sinks = solver
            .all_sinks(&mut graph, &state, &$req)
            .into_iter()
            .filter(|x| x.pc > $req.pc + 1)
            .map(|x| x.variable)
            .unique()
            .collect::<Vec<_>>();

        sinks.push("taut".to_string());

        sinks
    }};
}

macro_rules! orig {
    ($name:expr, $prog:expr, $req:expr) => {{
        let mut convert = TabulationOriginal::default();

        let (graph, state) = convert.visit(&$prog, $req).unwrap();

        let output = render_to(&graph, &state);

        assert_snapshot!(format!("_orig_{}_dot", $name), output);

        let sinks = graph
            .edges
            .iter()
            .map(|x| x.to())
            .filter(|x| &x.function == &$req.function)
            .map(|x| &x.belongs_to_var)
            .unique()
            .map(|x| x.clone())
            .collect::<Vec<_>>();

        sinks
    }};
}

macro_rules! fast {
    ($name:expr, $prog:expr, $req:expr) => {{
        let mut convert = TabulationFast::new(TaintInitialFlowFunction, TaintNormalFlowFunction);

        let (graph, state) = convert.visit(&$prog, $req).unwrap();

        let output = render_to(&graph, &state);

        assert_snapshot!(format!("_fast_{}_dot", $name), output);

        let sinks = graph
            .edges
            .iter()
            .map(|x| x.to())
            .filter(|x| &x.function == &$req.function)
            .map(|x| &x.belongs_to_var)
            .unique()
            .map(|x| x.clone())
            .collect::<Vec<_>>();

        sinks
    }};
}

macro_rules! sparse {
    ($name:expr, $prog:expr, $req:expr) => {{
        let mut convert = TabulationSparse::new(
            SparseTaintInitialFlowFunction,
            SparseTaintNormalFlowFunction,
        );

        let (graph, state) = convert.visit(&$prog, $req).unwrap();

        let output = render_to2(&graph, &state);

        assert_snapshot!(format!("_sparse_{}_dot", $name), output);

        let sinks = graph
            .edges
            .iter()
            .map(|x| x.to())
            .filter(|x| &x.function == &$req.function)
            .map(|x| &x.belongs_to_var)
            .unique()
            .map(|x| x.clone())
            .collect::<Vec<_>>();

        sinks
    }};
}

macro_rules! run {
    ($name:expr, $ir:expr, $req:expr) => {
        let prog = ir!($ir);
        let mut s1 = naive!($name, &prog, $req);
        s1.sort();
        let mut s2 = orig!($name, &prog, $req);
        s2.sort();
        let mut s3 = fast!($name, &prog, $req);
        s3.sort();
        let mut s4 = sparse!($name, &prog, $req);
        s4.sort();

        debug!("naive {:#?}", s1);
        debug!("orig {:#?}", s2);
        debug!("fast {:#?}", s3);
        debug!("sparse {:#?}", s4);

        let s1 = s1.into_iter().collect::<HashSet<_>>();
        let s2 = s2.into_iter().collect::<HashSet<_>>();
        let s3 = s3.into_iter().collect::<HashSet<_>>();
        let s4 = s4.into_iter().collect::<HashSet<_>>();

        assert!(s1.is_superset(&s2), "naive and orig failed");
        assert!(s1.is_superset(&s3), "naive and fast failed");
        assert!(s1.is_superset(&s4), "naive and sparsed failed");

        assert!(s2.is_superset(&s3), "orig and fast failed");
        assert!(s2.is_superset(&s4), "orig and sparse failed");

        assert_eq!(s1, s2, "naive and orig failed");
        assert_eq!(s1, s3, "naive and fast failed");
        assert_eq!(s1, s4, "naive and sparse failed");
    };
}

#[test]
fn test_intra() {
    let req = Request {
        function: "test".to_string(),
        pc: 0,
        variable: Some("%0".to_string()),
    };

    let ir = "define test (result 0) (define %0 %1) {
            %0 = 1
            %1 = %0
         };";

    run!("intra_reach", ir, &req);
}

#[test]
fn test_if_else() {
    let req = Request {
        function: "test".to_string(),
        pc: 0,
        variable: Some("%0".to_string()),
    };

    let ir = "define test (result 0) (define %0 %1 %2 %3) {
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
        };";

    run!("verification_if_else", ir, &req);
}

#[test]
fn test_early_return() {
    let req = Request {
        function: "test".to_string(),
        pc: 0,
        variable: Some("%0".to_string()),
    };

    let ir = "\
        define test (result 0) (define %0 %1 %2 %3) {
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
    ";

    run!("verification_early_return", ir, &req);
}

#[test]
fn test_memory_load_different_functions2() {
    let req = Request {
        variable: Some("%1".to_string()),
        function: "0".to_string(),
        pc: 2,
    };
    let ir = "
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
    ";
    run!("memory_load_different_functions2", ir, &req);
}

#[test]
fn test_ir_return_double() {
    let req = Request {
        variable: Some("%0".to_string()),
        function: "test".to_string(),
        pc: 0,
    };
    let ir = "define test (result 0) (define %0 %1 %2) {
            %0 = 1
            %1 %2 <- CALL mytest(%0)
            %1 = 2
        };
        define mytest (param %0) (result 2) (define %0 %1) {
            %0 = 2
            %1 = 3
            RETURN %0 %1;
        };
        ";
    run!("return_double", ir, &req);
}

#[test]
fn test_ir_return_double2() {
    let req = Request {
        variable: Some("%0".to_string()),
        function: "test".to_string(),
        pc: 0,
    };
    let ir = "define test (result 0) (define %0 %1 %2) {
            %0 = 1
            %1 %2 <- CALL mytest(%0)
            %1 = 2
        };
        define mytest (param %0) (result 2) (define %0 %1) {
            %1 = 3
            RETURN %0 %1;
        };
        ";
    run!("return_double2", ir, &req);
}

#[test]
fn test_global_get_and_set_multiple_functions() {
    let req = Request {
        variable: Some("%1".to_string()),
        function: "0".to_string(),
        pc: 0,
    };
    let ir = "
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
    ";
    run!("global_get_and_set_multiple_functions", ir, &req);
}

#[test]
fn test_return_branches() {
    let req = Request {
        variable: Some("%0".to_string()),
        function: "test".to_string(),
        pc: 0,
    };
    let ir = "define test (result 0) (define %0 %1) {
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
        ";

    run!("return_branches", ir, &req);
}
