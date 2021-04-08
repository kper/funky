use crate::solver::*;

use crate::icfg::convert::ConvertSummary;
use crate::icfg::tikz::render_to;
use insta::assert_snapshot;

use crate::grammar::*;

use crate::icfg::flowfuncs::taint::flow::TaintNormalFlowFunction;
use crate::icfg::flowfuncs::taint::initial::TaintInitialFlowFunction;

macro_rules! ir {
    ($name:expr, $req:expr, $ir:expr) => {{
        let mut convert = ConvertSummary::new(TaintInitialFlowFunction, TaintNormalFlowFunction);

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
    assert_eq!(3, touched_vars.len());

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

    let sinks = solver.all_sinks(&mut graph, &req).unwrap();

    assert_snapshot!(name, format!("{:#?}", sinks));

    assert_eq!(4, sinks.len());

    let touched_vars = vars(&sinks);
    assert_eq!(2, touched_vars.len());

    let touched_funcs = functions(&sinks);
    assert_eq!(1, touched_funcs.len());
}

#[test]
fn test_functions() {
    let mut solver = IfdsSolver;

    let name = "test_functions";

    let req = Request {
        function: "test".to_string(),
        pc: 0,
        variable: Some("%0".to_string()),
    };

    let mut graph = ir!(
        name,
        req,
        "
        define test (result 0) (define %0) {
            %0 = 1
            CALL mytest(%0)
        };
        define mytest (param %0) (result 0) (define %0 %1)  {
            %0 = 2   
            %1 = 3
            RETURN;
        };
    "
    );

    let sinks = solver.all_sinks(&mut graph, &req).unwrap();

    assert_snapshot!(name, format!("{:#?}", sinks));

    assert_eq!(3, sinks.len());

    let touched_vars = vars(&sinks);
    assert_eq!(2, touched_vars.len());

    let touched_funcs = functions(&sinks);
    assert_eq!(1, touched_funcs.len());
}

#[test]
fn test_gcd() {
    let mut solver = IfdsSolver;

    let name = "test_gcd";

    let req = Request {
        function: "0".to_string(),
        pc: 17,
        variable: Some("%11".to_string()),
    };

    let mut graph = ir!(
        name,
        req,
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

    let sinks = solver.all_sinks(&mut graph, &req).unwrap();

    assert_snapshot!(name, format!("{:#?}", sinks));

    assert_eq!(34, sinks.len());

    let touched_vars = vars(&sinks);
    assert_eq!(7, touched_vars.len());

    let touched_funcs = functions(&sinks);
    assert_eq!(1, touched_funcs.len());
}

#[test]
fn test_globals() {
    let mut solver = IfdsSolver;

    let name = "test_globals";

    let req = Request {
        function: "test".to_string(),
        pc: 0,
        variable: Some("%0".to_string()),
    };

    let mut graph = ir!(
        name,
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

    let sinks = solver.all_sinks(&mut graph, &req).unwrap();

    assert_snapshot!(name, format!("{:#?}", sinks));

    assert_eq!(7, sinks.len());

    let touched_vars = vars(&sinks);
    assert_eq!(4, touched_vars.len());

    let touched_funcs = functions(&sinks);
    assert_eq!(1, touched_funcs.len());
}

#[test]
fn test_returned_value() {
    let mut solver = IfdsSolver;

    let name = "test_returned_values";

    let req = Request {
        function: "0".to_string(),
        pc: 8,
        variable: Some("%11".to_string()),
    };

    let mut graph = ir!(
        name,
        req,
        "
        define 0 (param %0) (result 1) (define %0 %1 %7 %8 %9 %10 %11 %12 %13 %14 %15 %16 %17) {
            BLOCK 0
            BLOCK 1
            BLOCK 3
            %1 = %0
            %7 = %0
            %8 = -1
            %9 = %8 op %7
            %10 <- CALL 0(%9)
            %11 = %0
            %12 = -2
            %13 = %12 op %11
            %14 <- CALL 0(%13)
            %15 = %14 op %13
            KILL %15
            KILL %14
            RETURN %0;
        }; 
    "
    );

    let sinks = solver.all_sinks(&mut graph, &req).unwrap();

    assert_snapshot!(name, format!("{:#?}", sinks));
}

#[test]
fn test_looped_param() {
    let mut solver = IfdsSolver;

    let name = "test_looped_param";

    let req = Request {
        function: "0".to_string(),
        pc: 1,
        variable: Some("%7".to_string()),
    };

    let mut graph = ir!(
        name,
        req,
        "
        define 0 (param %0) (result 1) (define %0 %1 %2 %3 %4 %5 %6 %7 %8 %9 %10 %11 %12 %13 %14 %15 %16 %17) {
            %1 = %0
            %7 = %0
            %9 = %8 op %7
            %10 <- CALL 0(%9)
            %11 = %0
            RETURN %0;
        };
    "
    );

    let sinks = solver.all_sinks(&mut graph, &req).unwrap();

    assert_snapshot!(name, format!("{:#?}", sinks));
}
