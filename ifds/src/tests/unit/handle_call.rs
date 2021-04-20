use crate::icfg::convert::{ConvertSummary, Ctx};
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
    let (mut convert, mut graph, mut path_edge, mut worklist, _end_summary, mut normal_flows) =
        setup();

    let mut state = State::default();

    let mut ctx = Ctx {
        graph: &mut graph,
        state: &mut state,
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

    let foo = Fact {
        belongs_to_var: "%0".to_string(),
        function: "main".to_string(),
        next_pc: 1, //this is an edge case
        track: 1,
        ..Default::default()
    };

    let _ = ctx.state.cache_fact(&caller_function.name, foo.clone());

    let call_to_start_edges = convert
        .pass_args(
            &caller_function,
            &callee_function,
            &vec!["%0".to_string()],
            &mut ctx,
            1,
            &"%0".to_string(),
            &mut normal_flows,
            &mut path_edge,
            &mut worklist,
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
fn test_normal_argument_passing_with_multiple_params() {
    let (mut convert, mut graph, mut path_edge, mut worklist, _end_summary, mut normal_flows) =
        setup();

    let mut state = State::default();

    let mut ctx = Ctx {
        graph: &mut graph,
        state: &mut state,
    };

    let caller_function = AstFunction {
        name: "main".to_string(),
        results_len: 0,
        definitions: vec!["%0".to_string(), "%1".to_string(), "%2".to_string()],
        instructions: vec![
            Instruction::Const("%0".to_string(), 0.0),
            Instruction::Call(
                "test".to_string(),
                vec!["%0".to_string(), "%1".to_string()],
                vec!["%2".to_string()],
            ),
        ],
        ..Default::default()
    };

    let callee_function = AstFunction {
        name: "test".to_string(),
        params: vec!["%0".to_string(), "%1".to_string()],
        results_len: 1,
        definitions: vec!["%0".to_string(), "%1".to_string()],
        instructions: vec![Instruction::Return(vec!["%0".to_string()])],
    };

    let caller_init_facts = ctx.state.init_function(&caller_function, 0).unwrap();

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

    let foo = Fact {
        belongs_to_var: "%0".to_string(),
        function: "main".to_string(),
        next_pc: 1, //this is an edge case
        track: 1,
        ..Default::default()
    };

    let _ = ctx.state.cache_fact(&caller_function.name, foo.clone());

    let bar = Fact {
        belongs_to_var: "%1".to_string(),
        function: "main".to_string(),
        next_pc: 1, //this is an edge case
        track: 2,
        ..Default::default()
    };

    let _ = ctx.state.cache_fact(&caller_function.name, bar.clone());

    // first param
    {
        let call_to_start_edges = convert
            .pass_args(
                &caller_function,
                &callee_function,
                &vec!["%0".to_string(), "%1".to_string()],
                &mut ctx,
                1,
                &"%0".to_string(),
                &mut normal_flows,
                &mut path_edge,
                &mut worklist,
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

    // second param
    {
        let call_to_start_edges = convert
            .pass_args(
                &caller_function,
                &callee_function,
                &vec!["%0".to_string(), "%1".to_string()],
                &mut ctx,
                1,
                &"%1".to_string(),
                &mut normal_flows,
                &mut path_edge,
                &mut worklist,
            )
            .unwrap();

        assert_eq!(call_to_start_edges.len(), 1);
        assert_eq!(
            call_to_start_edges,
            vec![Edge::Call {
                from: bar,
                to: Fact {
                    belongs_to_var: "%1".to_string(),
                    function: "test".to_string(),
                    track: 2,
                    ..Default::default()
                }
            }]
        );
    }
}

#[test]
fn test_normal_argument_passing_taut() {
    let (mut convert, mut graph, mut path_edge, mut worklist, _end_summary, mut normal_flows) =
        setup();

    let mut state = State::default();

    let mut ctx = Ctx {
        graph: &mut graph,
        state: &mut state,
    };

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

    let caller_init_facts = ctx.state.init_function(&caller_function, 0).unwrap();

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

    let foo = Fact {
        belongs_to_var: "taut".to_string(),
        function: "main".to_string(),
        next_pc: 1, //this is an edge case
        track: 0,
        var_is_taut: true,
        ..Default::default()
    };

    let _ = ctx.state.cache_fact(&caller_function.name, foo.clone());

    let call_to_start_edges = convert
        .pass_args(
            &caller_function,
            &callee_function,
            &vec!["taut".to_string()],
            &mut ctx,
            1,
            &"taut".to_string(),
            &mut normal_flows,
            &mut path_edge,
            &mut worklist,
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

#[test]
fn test_pass_global_variable() {
    let (mut convert, mut graph, mut path_edge, mut worklist, _end_summary, mut normal_flows) =
        setup();

    let mut state = State::default();

    let mut ctx = Ctx {
        graph: &mut graph,
        state: &mut state,
    };

    let caller_function = AstFunction {
        name: "main".to_string(),
        results_len: 0,
        definitions: vec!["%-1".to_string(), "%0".to_string()],
        instructions: vec![
            Instruction::Const("%-1".to_string(), 0.0),
            Instruction::Call(
                "test".to_string(),
                vec!["%-1".to_string()],
                vec!["%0".to_string()],
            ),
        ],
        ..Default::default()
    };

    let callee_function = AstFunction {
        name: "test".to_string(),
        params: vec!["%0".to_string()],
        results_len: 1,
        definitions: vec!["%-1".to_string(), "%0".to_string()],
        instructions: vec![
            Instruction::Assign("%0".to_string(), "%-1".to_string()),
            Instruction::Return(vec!["%0".to_string()]),
        ],
    };

    let caller_init_facts = ctx.state.init_function(&caller_function, 0).unwrap();

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

    let foo = Fact {
        belongs_to_var: "%-1".to_string(),
        function: "main".to_string(),
        next_pc: 1, //this is an edge case
        track: 1,
        var_is_global: true,
        ..Default::default()
    };

    let _ = ctx.state.cache_fact(&caller_function.name, foo.clone());

    let call_to_start_edges = convert
        .pass_args(
            &caller_function,
            &callee_function,
            &vec!["%-1".to_string()],
            &mut ctx,
            1,
            &"%-1".to_string(),
            &mut normal_flows,
            &mut path_edge,
            &mut worklist,
        )
        .unwrap();

    assert_eq!(call_to_start_edges.len(), 1);
    assert_eq!(
        call_to_start_edges,
        vec![Edge::Call {
            from: foo,
            to: Fact {
                belongs_to_var: "%-1".to_string(),
                function: "test".to_string(),
                track: 1,
                var_is_global: true,
                ..Default::default()
            }
        }]
    );
}

#[test]
fn test_pass_memory() {
    let (mut convert, mut graph, mut path_edge, mut worklist, _end_summary, mut normal_flows) =
        setup();

    let mut state = State::default();

    let mut ctx = Ctx {
        graph: &mut graph,
        state: &mut state,
    };

    let caller_function = AstFunction {
        name: "main".to_string(),
        results_len: 0,
        definitions: vec!["%0".to_string(), "%1".to_string(), "%2".to_string()],
        instructions: vec![
            Instruction::Const("%0".to_string(), 0.0),
            Instruction::Const("%1".to_string(), 0.0),
            Instruction::Store("%0".to_string(), 0.0, "%1".to_string()),
            Instruction::Call("test".to_string(), vec![], vec!["%2".to_string()]),
        ],
        ..Default::default()
    };

    let callee_function = AstFunction {
        name: "test".to_string(),
        params: vec![],
        results_len: 1,
        definitions: vec!["%0".to_string(), "%1".to_string()],
        instructions: vec![
            Instruction::Const("%0".to_string(), 0.0),
            Instruction::Load("%1".to_string(), 0.0, "%0".to_string()),
            Instruction::Return(vec!["%1".to_string()]),
        ],
    };

    let caller_init_facts = ctx.state.init_function(&caller_function, 0).unwrap();

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

    let _ = ctx.state.add_memory_var(caller_function.name.clone(), 0.0);

    let foo = Fact {
        belongs_to_var: "mem@0".to_string(),
        function: "main".to_string(),
        next_pc: 1, //this is an edge case
        track: 2,
        var_is_memory: true,
        ..Default::default()
    };

    let _ = ctx.state.cache_fact(&caller_function.name, foo.clone());

    let call_to_start_edges = convert
        .pass_args(
            &caller_function,
            &callee_function,
            &vec![],
            &mut ctx,
            1,
            &"mem@0".to_string(),
            &mut normal_flows,
            &mut path_edge,
            &mut worklist,
        )
        .unwrap();

    assert_eq!(call_to_start_edges.len(), 1);
    assert_eq!(
        call_to_start_edges,
        vec![Edge::Call {
            from: foo,
            to: Fact {
                belongs_to_var: "mem@0".to_string(),
                function: "test".to_string(),
                track: 3,
                var_is_memory: true,
                ..Default::default()
            }
        }]
    );
}
