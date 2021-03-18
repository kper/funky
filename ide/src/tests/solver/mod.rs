use crate::solver::bfs::*;
use crate::solver::*;

use crate::icfg::convert::Convert;
use crate::icfg::tikz::render_to;
use insta::assert_snapshot;
use structopt::clap::value_t;

use log::debug;

use crate::grammar::*;

macro_rules! ir {
    ($name:expr, $ir:expr) => {{
        let mut convert = Convert::new();

        let prog = ProgramParser::new().parse(&$ir).unwrap();

        let graph = convert.visit(prog).unwrap();

        let output = render_to(&graph);

        assert_snapshot!(format!("{}_dot", $name), output);

        graph
    }};
}

fn vars(sink: &Vec<Taint>) -> Vec<String> {
    let mut touched_vars: Vec<_> = sink.iter().map(|x| x.to.clone()).collect();
    touched_vars.dedup();

    touched_vars
}

fn functions(sink: &Vec<Taint>) -> Vec<String> {
    let mut touched_vars: Vec<_> = sink.iter().map(|x| x.to_function.clone()).collect();
    touched_vars.dedup();

    touched_vars
}

#[test]
fn test_bfs_reachability_simple() {
    env_logger::init();

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
        Request {
            variable: "%0".to_string(),
            function: "test".to_string(),
            pc: 1,
        },
    );

    assert_eq!(2, sinks.len());

    let touched_vars = vars(&sinks);
    assert_eq!(2, touched_vars.len());

    let touched_funcs = functions(&sinks);
    assert_eq!(1, touched_funcs.len());
}
