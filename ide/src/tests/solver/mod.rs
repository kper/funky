use crate::solver::*;

use crate::icfg::convert2::ConvertSummary;
use crate::icfg::tikz::render_to;
use insta::assert_snapshot;

use crate::grammar::*;

macro_rules! ir {
    ($name:expr, $req:expr, $ir:expr) => {{
        let mut convert = ConvertSummary::new();

        let prog = ProgramParser::new().parse(&$ir).unwrap();

        let graph = convert.visit(&prog, &$req).unwrap();

        let output = render_to(&graph);

        assert_snapshot!(format!("{}_dot", $name), output);

        graph
    }};
}

fn vars(sink: &Vec<Taint>) -> Vec<String> {
    let mut touched_vars: Vec<_> = sink.iter().map(|x| x.variable.clone()).collect();
    touched_vars.sort_unstable();
    touched_vars.dedup();

    touched_vars
}

fn functions(sink: &Vec<Taint>) -> Vec<String> {
    let mut touched_vars: Vec<_> = sink.iter().map(|x| x.function.clone()).collect();
    touched_vars.sort_unstable();
    touched_vars.dedup();

    touched_vars
}

#[test]
fn test_intra_reachability() {
    let mut solver = IfdsSolver;

    let name = "intra_reach";

    let req = Request {
        function: "test".to_string(),
        pc: 0,
        variable: Some("%0".to_string()),
    };

    let mut graph = ir!(
        name,
        req,
        "
        define test (result 0) (define %0 %1) {
            %0 = 1
            %1 = %0
         };
    "
    );

    let sinks = solver
        .all_sinks(
            &mut graph,
            &Request {
                variable: Some("%0".to_string()),
                function: "test".to_string(),
                pc: 0,
            },
        )
        .unwrap();

    assert_snapshot!(name, format!("{:#?}", sinks));

    assert_eq!(4, sinks.len());

    let touched_vars = vars(&sinks);
    assert_eq!(2, touched_vars.len());

    let touched_funcs = functions(&sinks);
    assert_eq!(1, touched_funcs.len());
}

#[test]
fn test_loop() {
    let mut solver = IfdsSolver;

    let name = "test_loop";

    let req = Request {
        function: "main".to_string(),
        pc: 0,
        variable: Some("%0".to_string()),
    };

    let mut graph = ir!(
        name,
        req,
        "
        define main (result 0) (define %0 %1) {
            BLOCK 0
            %0 = 1
            GOTO 0 
        };
    "
    );

    let sinks = solver
        .all_sinks(
            &mut graph,
            &req,
        )
        .unwrap();

    assert_snapshot!(name, format!("{:#?}", sinks));

    assert_eq!(4, sinks.len());

    let touched_vars = vars(&sinks);
    assert_eq!(2, touched_vars.len());

    let touched_funcs = functions(&sinks);
    assert_eq!(1, touched_funcs.len());
}
