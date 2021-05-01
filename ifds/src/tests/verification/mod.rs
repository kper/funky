use crate::solver::bfs::*;
use crate::solver::*;
use itertools::Itertools;

use crate::grammar::*;
use crate::icfg::tabulation::fast::TabulationFast;
use crate::icfg::tabulation::naive::TabulationNaive;
use crate::icfg::tabulation::orig::TabulationOriginal;
use crate::icfg::tabulation::sparse::TabulationSparse;

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

        assert_eq!(s1, s2);
        assert_eq!(s1, s3);
        assert_eq!(s1, s4);
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
