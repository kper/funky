use crate::icfg::tabulation::fast::{TabulationFast, Ctx};
use crate::icfg::flowfuncs::taint::{
    flow::TaintNormalFlowFunction, initial::TaintInitialFlowFunction,
};
use crate::icfg::graph::*;
use crate::icfg::state::State;
use crate::ir::ast::Function as AstFunction;
use crate::ir::ast::Instruction;
use crate::ir::ast::Program;

use std::collections::{HashMap, VecDeque};

type PathEdges = Vec<Edge>;
type NormalFlows = Vec<Edge>;
type Worklist = VecDeque<Edge>;
type EndSummary = HashMap<(String, usize, String), Vec<Fact>>;
type Incoming = HashMap<(String, usize, String), Vec<Fact>>;

fn setup() -> (
    TabulationFast<TaintInitialFlowFunction, TaintNormalFlowFunction>,
    Graph,
    PathEdges,
    Worklist,
    EndSummary,
    NormalFlows,
) {
    (
        TabulationFast::new(TaintInitialFlowFunction, TaintNormalFlowFunction),
        Graph::default(),
        PathEdges::new(),
        Worklist::new(),
        EndSummary::new(),
        NormalFlows::new(),
    )
}

#[test]
fn test_normal_return() {
    let (mut convert, mut graph, mut path_edge, mut worklist, mut end_summary, mut normal_flows) =
        setup();
    let mut state = State::default();

    let mut ctx = Ctx {
        graph: &mut graph,
        state: &mut state,
    };

    let d1 = Fact {
        belongs_to_var: "%0".to_string(),
        function: "test".to_string(),
        next_pc: 0, //this is an edge case
        ..Default::default()
    };

    let d2 = Fact {
        belongs_to_var: "%0".to_string(),
        function: "test".to_string(),
        next_pc: 1,
        ..Default::default()
    };

    let caller_function = AstFunction {
        name: "main".to_string(),
        results_len: 0,
        definitions: vec!["%0".to_string(), "%1".to_string()],
        instructions: vec![
            Instruction::Const("%0".to_string(), 0.0),
            Instruction::Call(
                "test".to_string(),
                vec!["%0".to_string()],
                vec!["%1".to_string()],
            ),
        ],
        ..Default::default()
    };

    let callee_function = AstFunction {
        name: "test".to_string(),
        params: vec!["%0".to_string()],
        results_len: 1,
        definitions: vec!["%0".to_string()],
        instructions: vec![Instruction::Return(vec!["%0".to_string()])],
    };

    let caller_init_facts = ctx.state.init_function(&caller_function, 0).unwrap();
    let callee_init_facts = ctx.state.init_function(&callee_function, 0).unwrap();

    let _ = convert
        .pacemaker(
            &caller_function,
            &mut ctx,
            &mut path_edge,
            &mut worklist,
            &mut normal_flows,
            &caller_init_facts,
        )
        .unwrap();

    let _ = convert
        .pacemaker(
            &callee_function,
            &mut ctx,
            &mut path_edge,
            &mut worklist,
            &mut normal_flows,
            &callee_init_facts,
        )
        .unwrap();

    let d1 = ctx.state.cache_fact(&caller_function.name, d1).unwrap().clone();
    let d2 = ctx.state.cache_fact(&callee_function.name, d2).unwrap().clone();

    let program = Program {
        functions: vec![caller_function.clone(), callee_function.clone()],
    };

    // call of the caller
    let foo = Fact {
        belongs_to_var: "%0".to_string(),
        function: "main".to_string(),
        next_pc: 1, //this is an edge case
        track: 1,
        ..Default::default()
    };
    let caller_call_fact = ctx.state
        .cache_fact(&caller_function.name, foo)
        .unwrap()
        .clone();

    let mut summary_edge = Vec::new();
    let mut incoming = Incoming::default();

    incoming.insert(
        (callee_function.name.clone(), 0, "%0".to_string()),
        vec![caller_call_fact],
    );

    convert
        .end_procedure(
            &program,
            &mut ctx,
            &mut summary_edge,
            &mut incoming,
            &mut end_summary,
            &d1,
            &d2,
            &mut path_edge,
            &mut worklist,
        )
        .unwrap();

    assert_eq!(end_summary.len(), 1);

    assert_eq!(
        end_summary.get(&(callee_function.name.clone(), 0, "%0".to_string())),
        Some(&vec![Fact {
            belongs_to_var: "%0".to_string(),
            function: callee_function.name,
            next_pc: 1,
            ..Default::default()
        }])
    );
}
