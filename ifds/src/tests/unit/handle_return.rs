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
fn test_normal_function_call() {
    let (mut convert, mut graph, mut path_edge, mut worklist, mut end_summary, mut normal_flows) =
        setup();

    let mut state = State::default();

    let mut ctx = Ctx {
        graph: &mut graph,
        state: &mut state,
    };

    let d2 = Fact {
        belongs_to_var: "%0".to_string(),
        function: "test".to_string(),
        next_pc: 0, //this is an edge case
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

    let d2 = ctx
        .state
        .cache_fact(&callee_function.name, d2)
        .unwrap()
        .clone();

    let init_fact = ctx
        .state
        .get_taut(&callee_function.name, 0)
        .unwrap()
        .unwrap()
        .clone();

    convert
        .handle_return(
            &callee_function,
            &d2,
            &mut ctx,
            &mut normal_flows,
            &mut path_edge,
            &mut worklist,
            &init_fact,
            &mut end_summary,
        )
        .unwrap();

    assert_eq!(end_summary.len(), 1);

    assert_eq!(
        end_summary.get(&(callee_function.name.clone(), 0, "taut".to_string())),
        Some(&vec![Fact {
            belongs_to_var: "%0".to_string(),
            function: callee_function.name.clone(),
            ..Default::default()
        }])
    );
}

#[test]
fn test_return_directly_global() {
    let (mut convert, mut graph, mut path_edge, mut worklist, mut end_summary, mut normal_flows) =
        setup();

    let mut state = State::default();

    let mut ctx = Ctx {
        graph: &mut graph,
        state: &mut state,
    };

    let d2 = Fact {
        belongs_to_var: "%-1".to_string(),
        function: "test".to_string(),
        next_pc: 0, //this is an edge case
        ..Default::default()
    };

    let caller_function = AstFunction {
        name: "main".to_string(),
        results_len: 0,
        definitions: vec!["%-1".to_string(), "%0".to_string()],
        instructions: vec![
            Instruction::Const("%-1".to_string(), 0.0),
            Instruction::Call("test".to_string(), vec![], vec!["%1".to_string()]),
        ],
        ..Default::default()
    };

    let callee_function = AstFunction {
        name: "test".to_string(),
        params: vec![],
        results_len: 1,
        definitions: vec!["%-1".to_string(), "%0".to_string()],
        instructions: vec![Instruction::Return(vec!["%-1".to_string()])],
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

    let d2 = ctx
        .state
        .cache_fact(&callee_function.name, d2)
        .unwrap()
        .clone();

    let init_fact = ctx
        .state
        .get_taut(&callee_function.name, 0)
        .unwrap()
        .unwrap()
        .clone();

    convert
        .handle_return(
            &callee_function,
            &d2,
            &mut ctx,
            &mut normal_flows,
            &mut path_edge,
            &mut worklist,
            &init_fact,
            &mut end_summary,
        )
        .unwrap();

    assert_eq!(end_summary.len(), 1);

    assert_eq!(
        end_summary.get(&(callee_function.name.clone(), 0, "taut".to_string())),
        Some(&vec![Fact {
            belongs_to_var: "%-1".to_string(),
            function: callee_function.name.clone(),
            ..Default::default()
        }])
    );
}

#[test]
fn test_return_memory() {
    let (mut convert, mut graph, mut path_edge, mut worklist, mut end_summary, mut normal_flows) =
        setup();

    let mut state = State::default();

    let mut ctx = Ctx {
        graph: &mut graph,
        state: &mut state,
    };

    let d2 = Fact {
        belongs_to_var: "%1".to_string(),
        function: "test".to_string(),
        next_pc: 0, //this is an edge case
        ..Default::default()
    };

    let caller_function = AstFunction {
        name: "main".to_string(),
        results_len: 0,
        definitions: vec!["%0".to_string(), "%1".to_string(), "%2".to_string()],
        instructions: vec![
            Instruction::Const("%0".to_string(), 1.0),
            Instruction::Store("%1".to_string(), 0.0, "%0".to_string()),
            Instruction::Call(
                "test".to_string(),
                vec!["%1".to_string()],
                vec!["%2".to_string()],
            ),
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

    let d2 = ctx
        .state
        .cache_fact(&callee_function.name, d2)
        .unwrap()
        .clone();

    let init_fact = ctx
        .state
        .get_taut(&callee_function.name, 0)
        .unwrap()
        .unwrap()
        .clone();

    convert
        .handle_return(
            &callee_function,
            &d2,
            &mut ctx,
            &mut normal_flows,
            &mut path_edge,
            &mut worklist,
            &init_fact,
            &mut end_summary,
        )
        .unwrap();

    assert_eq!(end_summary.len(), 1);

    assert_eq!(
        end_summary.get(&(callee_function.name.clone(), 0, "taut".to_string())),
        Some(&vec![Fact {
            belongs_to_var: "%1".to_string(),
            function: callee_function.name.clone(),
            ..Default::default()
        }])
    );
}
