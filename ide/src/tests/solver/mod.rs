use crate::solver::bfs::*;
use crate::solver::*;

use crate::icfg::convert::Convert;
use crate::icfg::tikz::render_to;
use insta::assert_snapshot;

use crate::grammar::*;

macro_rules! ir {
    ($name:expr, $ir:expr) => {{
        let mut convert = Convert::new();

        let prog = ProgramParser::new().parse(&$ir).unwrap();

        let graph = convert.visit(&prog).unwrap();

        let output = render_to(&graph);

        assert_snapshot!(format!("{}_dot", $name), output);

        graph
    }};
}

fn vars(sink: &Vec<Taint>) -> Vec<String> {
    let mut touched_vars: Vec<_> = sink.iter().map(|x| x.to.clone()).collect();
    touched_vars.sort_unstable();
    touched_vars.dedup();

    touched_vars
}

fn functions(sink: &Vec<Taint>) -> Vec<String> {
    let mut touched_vars: Vec<_> = sink.iter().map(|x| x.to_function.clone()).collect();
    touched_vars.sort_unstable();
    touched_vars.dedup();

    touched_vars
}

#[test]
fn test_bfs_reachability_simple() {
    let mut solver = IfdsSolver::new(BFS);

    let mut graph = ir!(
        "bfs_simple",
        "
        define test (result 0) (define %0 %1) {
            %0 = 1
            %1 = %0
         };
    "
    );

    let sinks = solver.all_sinks(
        &mut graph,
        &Request {
            variable: "%0".to_string(),
            function: "test".to_string(),
            pc: 1,
        },
    );

    assert_eq!(3, sinks.len());

    let touched_vars = vars(&sinks);
    assert_eq!(2, touched_vars.len());

    let touched_funcs = functions(&sinks);
    assert_eq!(1, touched_funcs.len());
}

#[test]
fn test_bfs_reachability_call() {
    let mut solver = IfdsSolver::new(BFS);

    let mut graph = ir!(
        "bfs_call",
        "
        define test (result 0) (define %0 %1) {
            %0 = 1
            %1 = %0
            %1 <- CALL mytest(%0)
        };
        define mytest (param %2) (result 1) (define %2 %3) {
            %3 = %2
            RETURN %3;
        };
    "
    );

    let sinks = solver.all_sinks(
        &mut graph,
        &Request {
            variable: "%0".to_string(),
            function: "test".to_string(),
            pc: 1,
        },
    );

    assert_eq!(10, sinks.len());

    let touched_vars = vars(&sinks);
    assert_eq!(4, touched_vars.len());

    let touched_funcs = functions(&sinks);
    assert_eq!(2, touched_funcs.len());
}

#[test]
fn test_bfs_functions() {
    let mut solver = IfdsSolver::new(BFS);

    let mut graph = ir!("bfs_functions", 
        "define test (result 0) (define %0 %1 %2) {
            %0 = 1
            %1 <- CALL mytest(%0)
            %2 = %1
        };
        define mytest (param %0) (result 1) (define %0) {
            RETURN %0;
        };"
    );

    let sinks = solver.all_sinks(
        &mut graph,
        &Request {
            variable: "%0".to_string(),
            function: "test".to_string(),
            pc: 1,
        },
    );

    assert!(sinks.iter().find(|x| x.to_pc == 1 && x.to_function == "test".to_string()).is_some());
    assert!(sinks.iter().find(|x| x.to_pc == 2 && x.to_function == "test".to_string()).is_some());
    assert!(sinks.iter().find(|x| x.to_pc == 3 && x.to_function == "test".to_string()).is_some());
    assert!(sinks.iter().find(|x| x.to_pc == 1 && x.to_function == "mytest".to_string()).is_some());
}