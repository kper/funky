use crate::solver::bfs::*;
use crate::solver::*;
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
        let mut convert = TabulationOriginal::default();

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

        let (mut graph, state) = convert.visit(&$prog, $req).unwrap();

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

        assert_eq!(s3, s4, "fast and sparse failed");
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
