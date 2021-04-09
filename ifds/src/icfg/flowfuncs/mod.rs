/// This module contains the trait definition for the flow functions.
/// The developer can implement those traits and the corresponding
/// methods will be called by [`ConvertSummary`].
pub use crate::icfg::graph::*;
pub use crate::ir::ast::Function as AstFunction;
pub use crate::ir::ast::Instruction;
pub use anyhow::{Result, Context};
pub use log::debug;
use crate::icfg::state::State;

pub mod taint;

type FunctionName = String;
type BlockNum = String;
type PC = usize;

use std::collections::HashMap;

pub type BlockResolver = HashMap<(FunctionName, BlockNum), PC>;

/// This flow function is an edge case.
/// The graph will be built on demand by a given statement.
/// Therefore, we need different flow functions to kick off
/// the analysis.
pub trait InitialFlowFunction {
    fn flow(
        &self,
        function: &AstFunction,
        graph: &mut Graph,
        pc: usize,
        init_facts: &Vec<Fact>,
        normal_flows_debug: &mut Vec<Edge>,
        state: &mut State,
    ) -> Result<Vec<Edge>>;
}

/// Those flow functions keep propagating only the relevant
/// edges. Not relevant ones will be killed.
pub trait NormalFlowFunction {
    fn flow(
        &self,
        function: &AstFunction,
        graph: &mut Graph,
        pc: usize,
        variable: &String,
        block_resolver: &BlockResolver,
        state: &mut State,
    ) -> Result<Vec<Edge>>;
}
