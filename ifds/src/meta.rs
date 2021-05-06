/// This module handles the meta information which is relevant statistics.
use crate::icfg::graph::Graph;
use crate::icfg::state::State;

use crate::ir::ast::Program;
use serde::Serialize;

use crate::icfg::tabulation::sparse::defuse::DefUseChain;
use rayon::prelude::*;

#[derive(Serialize, Default)]
pub struct Meta {
    /// estimate without memory globals
    estimated_exploded_graph_size: Option<u128>,
    number_path_edges: Option<u128>,
    sparse_relevant_instructions: Option<u128>,
}

pub fn meta_naive(program: &Program) -> Meta {
    let sum: u128 = program
        .functions
        .par_iter()
        .fold(
            || 0u128,
            |a, b| a + (b.get_num_definitions() * b.get_num_instructions()) as u128,
        )
        .sum();

    Meta {
        estimated_exploded_graph_size: Some(sum),
        ..Default::default()
    }
}

pub fn meta_fast(program: &Program, graph: &Graph, _state: &State) -> Meta {
    let mut meta = meta_naive(program);

    let mut num_path_edges = 0;

    for _ in graph.edges.iter().filter(|x| x.is_path()) {
        num_path_edges += 1;
    }

    meta.number_path_edges = Some(num_path_edges);

    meta
}

pub fn meta_sparse(program: &Program, graph: &Graph, state: &State, defuse: &DefUseChain) -> Meta {
    let mut meta = meta_fast(program, &graph, &state);

    let chain_num = defuse.count_all();
    meta.sparse_relevant_instructions = Some(chain_num);

    meta
}
