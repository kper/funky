use crate::solver::*;
use crate::solver::bfs::*;

use crate::icfg::tabulation::naive::TabulationNaive;
use crate::icfg::tikz::render_to;
use insta::assert_snapshot;

use crate::grammar::*;

macro_rules! ir {
    ($name:expr, $ir:expr) => {{
        let mut convert = TabulationNaive::default();

        let prog = ProgramParser::new().parse(&$ir).unwrap();

        let (graph, state, _) = convert.visit(&prog).unwrap();

        let output = render_to(&graph, &state);

        assert_snapshot!(format!("_naive_{}_dot", $name), output);

        (graph, state)
    }};
}

#[test]
fn test_gcd() {
    let mut solver = Bfs;

    let name = "test_gcd";

    let req = Request {
        function: "0".to_string(),
        pc: 2,
        variable: Some("%4".to_string()),
    };

    let (mut graph, state) = ir!(
        name,
        "
       define 0 (param %0 %1) (result 1) (define %0 %1 %2 %3 %4 %5 %6 %7 %8 %9 %10 %11 %12 %13 %14 %15 %16) {
        BLOCK 0
        BLOCK 1
        %4 = %0
        %5 = %1
        %6 = %5 op %4
        IF %6 THEN GOTO 3 ELSE GOTO 4
        BLOCK 3 
        %7 = %0
        %2 = %7
        KILL %7
        GOTO 2
        GOTO 4
        BLOCK 4
        BLOCK 5
        %8 = %1
        %9 = 0
        %10 = %0
        %11 = %1
        %12 = %11 op %10
        %13 = %12
        %2 = %12
        %14 = %13 op %12
        %3 = %14
        %15 = %1
        %16 = 0
        BLOCK 2
        RETURN %15;
        }; 
    "
    );

    let sinks = solver.all_sinks(&mut graph, &state, &req);

    assert_snapshot!(format!("{}_naive", name), format!("{:#?}", sinks));
}
