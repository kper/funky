use crate::icfg::flowfuncs::taint::initial::TaintInitialFlowFunction;
use crate::icfg::flowfuncs::InitialFlowFunction;
use crate::icfg::graph::{Edge, Fact};
use crate::ir::ast::Instruction;

use crate::icfg::state::State;
use crate::ir::ast::Function as AstFunction;

use pretty_assertions::assert_eq;

fn flow(pc: usize, def: Vec<&str>, instructions: Vec<Instruction>) -> (Vec<Edge>, Vec<Edge>) {
    let mut state = State::default();

    let function = AstFunction {
        name: "main".to_string(),
        results_len: 0,
        definitions: def.into_iter().map(|x| x.to_string()).collect::<Vec<_>>(),
        instructions,
        ..Default::default()
    };

    let init_facts = state.init_function(&function, pc).unwrap();
    let mut normal_flows = Vec::new();

    let flow_function = TaintInitialFlowFunction;

    (
        flow_function
            .flow(&function, pc, &init_facts, &mut normal_flows, &mut state)
            .unwrap(),
        normal_flows,
    )
}

#[test]
fn test_const_initial_flow() {
    let (edges, _normal_flows) = flow(
        0,
        vec!["%0"],
        vec![Instruction::Const("%0".to_string(), 0.0)],
    );

    assert_eq!(edges.len(), 2);

    assert_eq!(
        Some(&Edge::Path {
            from: Fact {
                var_is_taut: true,
                next_pc: 0,
                belongs_to_var: "taut".to_string(),
                function: "main".to_string(),
                ..Default::default()
            },
            to: Fact {
                var_is_taut: true,
                next_pc: 0,
                belongs_to_var: "taut".to_string(),
                function: "main".to_string(),
                ..Default::default()
            },
        }),
        edges.get(0),
    );
    assert_eq!(
        Some(&Edge::Path {
            from: Fact {
                var_is_taut: true,
                next_pc: 0,
                belongs_to_var: "taut".to_string(),
                function: "main".to_string(),
                ..Default::default()
            },
            to: Fact {
                next_pc: 1,
                belongs_to_var: "%0".to_string(),
                function: "main".to_string(),
                track: 1,
                ..Default::default()
            },
        }),
        edges.get(1),
    );
}

#[test]
fn test_assign_initial_flow() {
    let (edges, normal_flows) = flow(
        1,
        vec!["%0", "%1"],
        vec![
            Instruction::Const("%0".to_string(), 0.0),
            Instruction::Assign("%1".to_string(), "%0".to_string()),
        ],
    );

    assert_eq!(edges.len(), 2);

    assert_eq!(
        Some(&Edge::Path {
            from: Fact {
                var_is_taut: true,
                next_pc: 1,
                belongs_to_var: "taut".to_string(),
                function: "main".to_string(),
                ..Default::default()
            },
            to: Fact {
                var_is_taut: true,
                next_pc: 1,
                belongs_to_var: "taut".to_string(),
                function: "main".to_string(),
                ..Default::default()
            },
        }),
        edges.get(0),
    );
    assert_eq!(
        Some(&Edge::Path {
            from: Fact {
                var_is_taut: true,
                next_pc: 1,
                belongs_to_var: "taut".to_string(),
                function: "main".to_string(),
                ..Default::default()
            },
            to: Fact {
                next_pc: 2,
                belongs_to_var: "%1".to_string(),
                function: "main".to_string(),
                track: 2,
                ..Default::default()
            },
        }),
        edges.get(1)
    );

    assert_eq!(
        Some(&Edge::Normal {
            from: Fact {
                var_is_taut: true,
                next_pc: 1,
                belongs_to_var: "taut".to_string(),
                function: "main".to_string(),
                ..Default::default()
            },
            to: Fact {
                next_pc: 2,
                belongs_to_var: "%1".to_string(),
                function: "main".to_string(),
                track: 2,
                ..Default::default()
            },
            curved: false
        }),
        normal_flows.get(0)
    );
}

#[test]
fn test_unop_initial_flow() {
    let (edges, normal_flows) = flow(
        1,
        vec!["%0", "%1"],
        vec![
            Instruction::Const("%0".to_string(), 0.0),
            Instruction::Unop("%1".to_string(), "%0".to_string()),
        ],
    );

    assert_eq!(edges.len(), 2);

    assert_eq!(
        Some(&Edge::Path {
            from: Fact {
                var_is_taut: true,
                next_pc: 1,
                belongs_to_var: "taut".to_string(),
                function: "main".to_string(),
                ..Default::default()
            },
            to: Fact {
                var_is_taut: true,
                next_pc: 1,
                belongs_to_var: "taut".to_string(),
                function: "main".to_string(),
                ..Default::default()
            },
        }),
        edges.get(0),
    );
    assert_eq!(
        Some(&Edge::Path {
            from: Fact {
                var_is_taut: true,
                next_pc: 1,
                belongs_to_var: "taut".to_string(),
                function: "main".to_string(),
                ..Default::default()
            },
            to: Fact {
                next_pc: 2,
                belongs_to_var: "%1".to_string(),
                function: "main".to_string(),
                track: 2,
                ..Default::default()
            },
        }),
        edges.get(1)
    );

    assert_eq!(
        Some(&Edge::Normal {
            from: Fact {
                var_is_taut: true,
                next_pc: 1,
                belongs_to_var: "taut".to_string(),
                function: "main".to_string(),
                ..Default::default()
            },
            to: Fact {
                next_pc: 2,
                belongs_to_var: "%1".to_string(),
                function: "main".to_string(),
                track: 2,
                ..Default::default()
            },
            curved: false
        }),
        normal_flows.get(0)
    );
}

#[test]
fn test_binop_initial_flow() {
    let (edges, normal_flows) = flow(
        2,
        vec!["%0", "%1", "%2"],
        vec![
            Instruction::Const("%0".to_string(), 0.0),
            Instruction::Const("%1".to_string(), 0.0),
            Instruction::BinOp("%2".to_string(), "%1".to_string(), "%0".to_string()),
        ],
    );

    assert_eq!(edges.len(), 2);

    assert_eq!(
        Some(&Edge::Path {
            from: Fact {
                var_is_taut: true,
                next_pc: 2,
                belongs_to_var: "taut".to_string(),
                function: "main".to_string(),
                ..Default::default()
            },
            to: Fact {
                var_is_taut: true,
                next_pc: 2,
                belongs_to_var: "taut".to_string(),
                function: "main".to_string(),
                ..Default::default()
            },
        }),
        edges.get(0),
    );
    assert_eq!(
        Some(&Edge::Path {
            from: Fact {
                var_is_taut: true,
                next_pc: 2,
                belongs_to_var: "taut".to_string(),
                function: "main".to_string(),
                ..Default::default()
            },
            to: Fact {
                next_pc: 3,
                belongs_to_var: "%2".to_string(),
                function: "main".to_string(),
                track: 3,
                ..Default::default()
            },
        }),
        edges.get(1)
    );

    assert_eq!(
        Some(&Edge::Normal {
            from: Fact {
                var_is_taut: true,
                next_pc: 2,
                belongs_to_var: "taut".to_string(),
                function: "main".to_string(),
                ..Default::default()
            },
            to: Fact {
                next_pc: 3,
                belongs_to_var: "%2".to_string(),
                function: "main".to_string(),
                track: 3,
                ..Default::default()
            },
            curved: false
        }),
        normal_flows.get(0)
    );
}

#[test]
fn test_conditional_initial_flow() {
    let (edges, normal_flows) = flow(
        1,
        vec!["%0", "%1"],
        vec![
            Instruction::Const("%0".to_string(), 0.0),
            Instruction::Conditional("%0".to_string(), vec!["0".to_string()]),
            Instruction::Block("0".to_string()),
        ],
    );

    assert_eq!(edges.len(), 2);

    assert_eq!(
        Some(&Edge::Path {
            from: Fact {
                var_is_taut: true,
                next_pc: 1,
                belongs_to_var: "taut".to_string(),
                function: "main".to_string(),
                ..Default::default()
            },
            to: Fact {
                var_is_taut: true,
                next_pc: 1,
                belongs_to_var: "taut".to_string(),
                function: "main".to_string(),
                ..Default::default()
            },
        }),
        edges.get(0),
    );
    assert_eq!(
        Some(&Edge::Path {
            from: Fact {
                var_is_taut: true,
                next_pc: 1,
                belongs_to_var: "taut".to_string(),
                function: "main".to_string(),
                ..Default::default()
            },
            to: Fact {
                next_pc: 2,
                belongs_to_var: "%0".to_string(),
                function: "main".to_string(),
                track: 1,
                ..Default::default()
            },
        }),
        edges.get(1)
    );

    assert_eq!(
        Some(&Edge::Normal {
            from: Fact {
                var_is_taut: true,
                next_pc: 1,
                belongs_to_var: "taut".to_string(),
                function: "main".to_string(),
                ..Default::default()
            },
            to: Fact {
                next_pc: 2,
                belongs_to_var: "%0".to_string(),
                function: "main".to_string(),
                track: 1,
                ..Default::default()
            },
            curved: false
        }),
        normal_flows.get(0)
    );
}

#[test]
fn test_phi_initial_flow() {
    let (edges, normal_flows) = flow(
        2,
        vec!["%0", "%1", "%2"],
        vec![
            Instruction::Const("%0".to_string(), 0.0),
            Instruction::Const("%1".to_string(), 0.0),
            Instruction::Phi("%2".to_string(), "%1".to_string(), "%0".to_string()),
        ],
    );

    assert_eq!(edges.len(), 2);

    assert_eq!(
        Some(&Edge::Path {
            from: Fact {
                var_is_taut: true,
                next_pc: 2,
                belongs_to_var: "taut".to_string(),
                function: "main".to_string(),
                ..Default::default()
            },
            to: Fact {
                var_is_taut: true,
                next_pc: 2,
                belongs_to_var: "taut".to_string(),
                function: "main".to_string(),
                ..Default::default()
            },
        }),
        edges.get(0),
    );
    assert_eq!(
        Some(&Edge::Path {
            from: Fact {
                var_is_taut: true,
                next_pc: 2,
                belongs_to_var: "taut".to_string(),
                function: "main".to_string(),
                ..Default::default()
            },
            to: Fact {
                next_pc: 3,
                belongs_to_var: "%2".to_string(),
                function: "main".to_string(),
                track: 3,
                ..Default::default()
            },
        }),
        edges.get(1)
    );

    assert_eq!(
        Some(&Edge::Normal {
            from: Fact {
                var_is_taut: true,
                next_pc: 2,
                belongs_to_var: "taut".to_string(),
                function: "main".to_string(),
                ..Default::default()
            },
            to: Fact {
                next_pc: 3,
                belongs_to_var: "%2".to_string(),
                function: "main".to_string(),
                track: 3,
                ..Default::default()
            },
            curved: false
        }),
        normal_flows.get(0)
    );
}

#[test]
fn test_block_instruction_initial_flow() {
    let (edges, normal_flows) = flow(
        2,
        vec!["%0", "%1", "%2"],
        vec![
            Instruction::Const("%0".to_string(), 0.0),
            Instruction::Block("0".to_string()),
            Instruction::Const("%2".to_string(), 0.0),
        ],
    );

    assert_eq!(edges.len(), 2);

    assert_eq!(
        Some(&Edge::Path {
            from: Fact {
                var_is_taut: true,
                next_pc: 2,
                belongs_to_var: "taut".to_string(),
                function: "main".to_string(),
                ..Default::default()
            },
            to: Fact {
                var_is_taut: true,
                next_pc: 2,
                belongs_to_var: "taut".to_string(),
                function: "main".to_string(),
                ..Default::default()
            },
        }),
        edges.get(0),
    );
    assert_eq!(
        Some(&Edge::Path {
            from: Fact {
                var_is_taut: true,
                next_pc: 2,
                belongs_to_var: "taut".to_string(),
                function: "main".to_string(),
                ..Default::default()
            },
            to: Fact {
                next_pc: 3,
                belongs_to_var: "%2".to_string(),
                function: "main".to_string(),
                track: 3,
                ..Default::default()
            },
        }),
        edges.get(1)
    );

    assert_eq!(
        Some(&Edge::Normal {
            from: Fact {
                var_is_taut: true,
                next_pc: 2,
                belongs_to_var: "taut".to_string(),
                function: "main".to_string(),
                ..Default::default()
            },
            to: Fact {
                next_pc: 3,
                belongs_to_var: "%2".to_string(),
                function: "main".to_string(),
                track: 3,
                ..Default::default()
            },
            curved: false
        }),
        normal_flows.get(0)
    );
}