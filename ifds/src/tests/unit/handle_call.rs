use crate::icfg::convert::ConvertSummary;
use crate::icfg::flowfuncs::taint::{
    flow::TaintNormalFlowFunction, initial::TaintInitialFlowFunction,
};
use crate::icfg::graph::*;
use crate::icfg::state::State;
use crate::ir::ast::Function as AstFunction;
use crate::ir::ast::Instruction;

use std::collections::{HashMap, VecDeque};

type PathEdges = Vec<Edge>;
type NormalFlows = Vec<Edge>;
type Worklist = VecDeque<Edge>;
type EndSummary = HashMap<(String, usize, String), Vec<Fact>>;

fn setup() -> (
    ConvertSummary<TaintInitialFlowFunction, TaintNormalFlowFunction>,
    Graph,
    PathEdges,
    Worklist,
    EndSummary,
    NormalFlows,
) {
    (
        ConvertSummary::new(TaintInitialFlowFunction, TaintNormalFlowFunction),
        Graph::default(),
        PathEdges::new(),
        Worklist::new(),
        EndSummary::new(),
        NormalFlows::new(),
    )
}

#[test]
fn test_normal_argument_passing() {
    let (mut convert, mut graph, mut path_edge, mut worklist, mut end_summary, mut normal_flows) =
        setup();

    let mut state = State::default();

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

    let caller_init_facts = state.init_function(&caller_function, 0).unwrap();

    let _ = convert
        .pacemaker(
            &caller_function,
            &mut graph,
            &mut path_edge,
            &mut worklist,
            &mut normal_flows,
            &caller_init_facts,
            &mut state,
        )
        .unwrap();

    let foo = Fact {
        belongs_to_var: "%0".to_string(),
        function: "main".to_string(),
        next_pc: 1, //this is an edge case
        track: 1,
        ..Default::default()
    };

    let _ = state.cache_fact(&caller_function.name, foo.clone());

    let call_to_start_edges = convert
        .pass_args(
            &caller_function,
            &callee_function,
            &vec!["%0".to_string()],
            &mut graph,
            1,
            &"%0".to_string(),
            &mut normal_flows,
            &mut path_edge,
            &mut worklist,
            &mut state,
        )
        .unwrap();

    assert_eq!(call_to_start_edges.len(), 1);
    assert_eq!(
        call_to_start_edges,
        vec![Edge::Call {
            from: foo,
            to: Fact {
                belongs_to_var: "%0".to_string(),
                function: "test".to_string(),
                track: 1,
                ..Default::default()
            }
        }]
    );
}

#[test]
fn test_normal_argument_passing_taut() {
    let (mut convert, mut graph, mut path_edge, mut worklist, mut end_summary, mut normal_flows) =
        setup();

    let mut state = State::default();

    let caller_function = AstFunction {
        name: "main".to_string(),
        results_len: 0,
        definitions: vec!["%0".to_string(), "%1".to_string()],
        instructions: vec![
            Instruction::Const("%0".to_string(), 0.0),
            Instruction::Call("test".to_string(), vec![], vec![]),
        ],
        ..Default::default()
    };

    let callee_function = AstFunction {
        name: "test".to_string(),
        params: vec![],
        results_len: 1,
        definitions: vec!["%0".to_string()],
        instructions: vec![
            Instruction::Const("%0".to_string(), 1.0),
            Instruction::Return(vec!["%0".to_string()]),
        ],
    };

    let caller_init_facts = state.init_function(&caller_function, 0).unwrap();

    let _ = convert
        .pacemaker(
            &caller_function,
            &mut graph,
            &mut path_edge,
            &mut worklist,
            &mut normal_flows,
            &caller_init_facts,
            &mut state,
        )
        .unwrap();

    let foo = Fact {
        belongs_to_var: "taut".to_string(),
        function: "main".to_string(),
        next_pc: 1, //this is an edge case
        track: 0,
        var_is_taut: true,
        ..Default::default()
    };

    let _ = state.cache_fact(&caller_function.name, foo.clone());

    let call_to_start_edges = convert
        .pass_args(
            &caller_function,
            &callee_function,
            &vec!["taut".to_string()],
            &mut graph,
            1,
            &"taut".to_string(),
            &mut normal_flows,
            &mut path_edge,
            &mut worklist,
            &mut state,
        )
        .unwrap();

    assert_eq!(call_to_start_edges.len(), 1);
    assert_eq!(
        call_to_start_edges,
        vec![Edge::Call {
            from: foo,
            to: Fact {
                belongs_to_var: "taut".to_string(),
                function: "test".to_string(),
                track: 0,
                var_is_taut: true,
                ..Default::default()
            }
        }]
    );
}
