use crate::counter::{Counter, StackedCounter};
use crate::ssa::ast::Function as AstFunction;
use anyhow::{bail, Context, Result};
use log::debug;
use std::collections::HashMap;

type VarId = String;
type FunctionName = String;

#[derive(Debug, Default)]
pub struct Graph {
    pub vars: HashMap<FunctionName, Vec<Variable>>,
    pub functions: HashMap<FunctionName, Function>,
    pub facts: Vec<Fact>,
    pub edges: Vec<Edge>,
    pc_counter: Counter,
}

#[derive(Debug, Clone)]
pub struct Fact {
    id: usize,
    belongs_to_var: VarId,
    pc: usize,
    function: FunctionName,
}

#[derive(Debug)]
pub struct Function {
    name: FunctionName,
}

#[derive(Debug)]
pub struct Variable {
    name: FunctionName,
    function: FunctionName,
}

#[derive(Debug, Clone)]
pub enum Edge {
    Normal { from: Fact, to: Fact },
    Call { from: Fact, to: Fact },
    CallToReturn { from: Fact, to: Fact },
    Return { from: Fact, to: Fact },
}

impl Graph {
    pub fn new() -> Self {
        Graph::default()
    }

    pub fn init_function(&mut self, function: &AstFunction) -> Result<()> {
        debug!("Adding new function {} to the graph", function.name);

        self.functions.insert(
            function.name.clone(),
            Function {
                name: function.name,
            },
        );

        Ok(())
    }
}
