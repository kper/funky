use crate::icfg::flowfuncs::NormalFlowFunction;
use crate::icfg::flowfuncs::{taint::flow::TaintNormalFlowFunction, BlockResolver};
use crate::icfg::graph::{Edge, Fact};
use crate::ir::ast::Instruction;

use crate::icfg::state::State;
use crate::ir::ast::Function as AstFunction;

use pretty_assertions::assert_eq;

const MAIN_FUNCTION: &'static str = "main";

fn flow(
    start_pc: usize,
    def: Vec<&str>,
    instructions: Vec<Instruction>,
    variable: &String,
    state: &mut State,
) -> Vec<Edge> {
    let function = AstFunction {
        name: MAIN_FUNCTION.to_string(),
        results_len: 0,
        definitions: def.into_iter().map(|x| x.to_string()).collect::<Vec<_>>(),
        instructions,
        ..Default::default()
    };

    let _ = state.init_function(&function, start_pc);

    let flow_function = TaintNormalFlowFunction;

    let mut block_resolver = BlockResolver::default();

    for (pc, instruction) in function
        .instructions
        .iter()
        .enumerate()
        .skip(start_pc)
        .filter(|x| matches!(x.1, Instruction::Block(_)))
    {
        match instruction {
            Instruction::Block(block) => {
                block_resolver.insert((function.name.clone(), block.clone()), pc);
            }
            _ => {
                panic!("This code should be unreachable.");
            }
        }
    }

    flow_function
        .flow(&function, start_pc, variable, &mut block_resolver, state)
        .unwrap()
}

#[test]
fn test_const_initial_flow() {
    let mut state = State::default();

    let _ = state.cache_fact(
        &MAIN_FUNCTION.to_string(),
        Fact {
            belongs_to_var: "taut".to_string(),
            next_pc: 0,
            track: 0,
            var_is_taut: true,
            function: MAIN_FUNCTION.to_string(),
            ..Default::default()
        },
    );

    let _ = state.cache_fact(
        &MAIN_FUNCTION.to_string(),
        Fact {
            belongs_to_var: "%0".to_string(),
            next_pc: 0,
            track: 1,
            function: MAIN_FUNCTION.to_string(),
            ..Default::default()
        },
    );

    let edges = flow(
        0,
        vec!["%0"],
        vec![Instruction::Const("%0".to_string(), 0.0)],
        &"taut".to_string(),
        &mut state,
    );

    assert_eq!(0, edges.len());
}

#[test]
fn test_unop_initial_flow() {
    let mut state = State::default();

    for pc in 0..=1 {
        let _ = state.cache_fact(
            &MAIN_FUNCTION.to_string(),
            Fact {
                belongs_to_var: "taut".to_string(),
                next_pc: pc,
                track: 0,
                var_is_taut: true,
                function: MAIN_FUNCTION.to_string(),
                ..Default::default()
            },
        );

        let _ = state.cache_fact(
            &MAIN_FUNCTION.to_string(),
            Fact {
                belongs_to_var: "%0".to_string(),
                next_pc: pc,
                track: 1,
                function: MAIN_FUNCTION.to_string(),
                ..Default::default()
            },
        );
    }

    let edges = flow(
        1,
        vec!["%0", "%1"],
        vec![
            Instruction::Const("%0".to_string(), 0.0),
            Instruction::Unop("%1".to_string(), "%0".to_string()),
        ],
        &"%0".to_string(),
        &mut state,
    );

    assert_eq!(edges.len(), 2);

    assert_eq!(
        Some(&Edge::Normal {
            from: Fact {
                next_pc: 1,
                belongs_to_var: "%0".to_string(),
                function: MAIN_FUNCTION.to_string(),
                track: 1,
                ..Default::default()
            },
            to: Fact {
                pc: 1,
                next_pc: 2,
                belongs_to_var: "%1".to_string(),
                function: MAIN_FUNCTION.to_string(),
                track: 2,
                ..Default::default()
            },
            curved: false,
        }),
        edges.get(0),
    );

    assert_eq!(
        Some(&Edge::Normal {
            from: Fact {
                next_pc: 1,
                belongs_to_var: "%0".to_string(),
                function: MAIN_FUNCTION.to_string(),
                track: 1,
                ..Default::default()
            },
            to: Fact {
                pc: 1,
                next_pc: 2,
                belongs_to_var: "%0".to_string(),
                function: MAIN_FUNCTION.to_string(),
                track: 1,
                ..Default::default()
            },
            curved: false,
        }),
        edges.get(1),
    );
}

#[test]
fn test_binop_initial_flow() {
    let mut state = State::default();

    for pc in 0..=2 {
        let _ = state.cache_fact(
            &MAIN_FUNCTION.to_string(),
            Fact {
                belongs_to_var: "taut".to_string(),
                pc: (pc as usize).checked_sub(1).unwrap_or(0),
                next_pc: pc,
                track: 0,
                var_is_taut: true,
                function: MAIN_FUNCTION.to_string(),
                ..Default::default()
            },
        );

        let _ = state.cache_fact(
            &MAIN_FUNCTION.to_string(),
            Fact {
                belongs_to_var: "%0".to_string(),
                pc: (pc as usize).checked_sub(1).unwrap_or(0),
                next_pc: pc,
                track: 1,
                function: MAIN_FUNCTION.to_string(),
                ..Default::default()
            },
        );
    }

    let edges = flow(
        2,
        vec!["%0", "%1", "%2"],
        vec![
            Instruction::Const("%0".to_string(), 0.0),
            Instruction::Const("%1".to_string(), 0.0),
            Instruction::BinOp("%2".to_string(), "%0".to_string(), "%1".to_string()),
        ],
        &"%0".to_string(),
        &mut state,
    );

    assert_eq!(edges.len(), 2);

    assert_eq!(
        Some(&Edge::Normal {
            from: Fact {
                pc: 1,
                next_pc: 2,
                belongs_to_var: "%0".to_string(),
                function: MAIN_FUNCTION.to_string(),
                track: 1,
                ..Default::default()
            },
            to: Fact {
                pc: 2,
                next_pc: 3,
                belongs_to_var: "%2".to_string(),
                function: MAIN_FUNCTION.to_string(),
                track: 3,
                ..Default::default()
            },
            curved: false,
        }),
        edges.get(0),
    );

    assert_eq!(
        Some(&Edge::Normal {
            from: Fact {
                pc: 1,
                next_pc: 2,
                belongs_to_var: "%0".to_string(),
                function: MAIN_FUNCTION.to_string(),
                track: 1,
                ..Default::default()
            },
            to: Fact {
                pc: 2,
                next_pc: 3,
                belongs_to_var: "%0".to_string(),
                function: MAIN_FUNCTION.to_string(),
                track: 1,
                ..Default::default()
            },
            curved: false,
        }),
        edges.get(1),
    );
}

#[test]
fn test_phi_initial_flow() {
    let mut state = State::default();

    for pc in 0..=2 {
        let _ = state.cache_fact(
            &MAIN_FUNCTION.to_string(),
            Fact {
                belongs_to_var: "taut".to_string(),
                pc: (pc as usize).checked_sub(1).unwrap_or(0),
                next_pc: pc,
                track: 0,
                var_is_taut: true,
                function: MAIN_FUNCTION.to_string(),
                ..Default::default()
            },
        );

        let _ = state.cache_fact(
            &MAIN_FUNCTION.to_string(),
            Fact {
                belongs_to_var: "%0".to_string(),
                pc: (pc as usize).checked_sub(1).unwrap_or(0),
                next_pc: pc,
                track: 1,
                function: MAIN_FUNCTION.to_string(),
                ..Default::default()
            },
        );
    }

    let edges = flow(
        2,
        vec!["%0", "%1", "%2"],
        vec![
            Instruction::Const("%0".to_string(), 0.0),
            Instruction::Const("%1".to_string(), 0.0),
            Instruction::Phi("%2".to_string(), "%0".to_string(), "%1".to_string()),
        ],
        &"%0".to_string(),
        &mut state,
    );

    assert_eq!(edges.len(), 2);

    assert_eq!(
        Some(&Edge::Normal {
            from: Fact {
                pc: 1,
                next_pc: 2,
                belongs_to_var: "%0".to_string(),
                function: MAIN_FUNCTION.to_string(),
                track: 1,
                ..Default::default()
            },
            to: Fact {
                pc: 2,
                next_pc: 3,
                belongs_to_var: "%2".to_string(),
                function: MAIN_FUNCTION.to_string(),
                track: 3,
                ..Default::default()
            },
            curved: false,
        }),
        edges.get(0),
    );

    assert_eq!(
        Some(&Edge::Normal {
            from: Fact {
                pc: 1,
                next_pc: 2,
                belongs_to_var: "%0".to_string(),
                function: MAIN_FUNCTION.to_string(),
                track: 1,
                ..Default::default()
            },
            to: Fact {
                pc: 2,
                next_pc: 3,
                belongs_to_var: "%0".to_string(),
                function: MAIN_FUNCTION.to_string(),
                track: 1,
                ..Default::default()
            },
            curved: false,
        }),
        edges.get(1),
    );
}
