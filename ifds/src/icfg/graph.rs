#![allow(dead_code)]

use crate::counter::Counter;
use crate::ir::ast::Function as AstFunction;
use anyhow::{Context, Result};
use log::debug;
use std::collections::HashMap;

type VarId = String;
type FunctionName = String;

/// The datastructure for the graph.
#[derive(Debug, Default)]
pub struct Graph {
    //pub facts: Vec<Fact>,
    pub edges: Vec<Edge>,
    //pub notes: Vec<Note>,
    pub fact_counter: Counter,
    note_counter: Counter,
    /// `init_facts` is a helper struct for getting the initial facts
    /// of a functions. We need this because we have to reinitalize the
    /// function when function is calling itself.
    init_facts: HashMap<FunctionName, Vec<Fact>>,
}

/// A helper struct for the graph representation in `tikz`
/// An instruction is a note in the .tex file because it makes the graph easier to read.
#[derive(Debug, Clone)]
pub struct Note {
    pub id: usize,
    pub function: String,
    pub pc: usize,
    pub note: String,
}

/// A fact is an variable at a given instruction. The instruction is defined
/// as `next_pc`.
#[derive(Debug, Clone, PartialEq, PartialOrd)]
pub struct Fact {
    pub belongs_to_var: VarId,
    pub var_is_global: bool,
    pub var_is_taut: bool,
    pub var_is_memory: bool,
    pub next_pc: usize,
    pub track: usize,
    pub function: FunctionName,
    /// if the fact saves a memory variable
    /// then save the offset.
    pub memory_offset: Option<f64>,
}

/// An IFDS representation for a function.
#[derive(Debug)]
pub struct Function {
    pub name: FunctionName,
    pub definitions: usize,
    pub return_count: usize,
}

/// The register which will be used at some point in the module.
#[derive(Debug, Clone, PartialEq)]
pub struct Variable {
    pub name: FunctionName,
    pub function: FunctionName,
    /// variable is a global variable
    pub is_global: bool,
    /// variable represents the tautological fact
    pub is_taut: bool,
    /// variable is a memory variable
    pub is_memory: bool,
    /// if variable is a memory variable, then also save
    /// the memory's offset
    pub memory_offset: Option<f64>,
}

/// The datastructure for an edge in the graph.
#[derive(Debug, Clone, PartialEq)]
pub enum Edge {
    Normal { from: Fact, to: Fact, curved: bool },
    Call { from: Fact, to: Fact },
    CallToReturn { from: Fact, to: Fact },
    Return { from: Fact, to: Fact },
    Path { from: Fact, to: Fact },
    Summary { from: Fact, to: Fact },
}

impl Edge {
    /// Extract `from`'s [`Fact`] from the edge, no matter which variant.
    pub fn get_from(&self) -> &Fact {
        match self {
            Edge::Normal {
                from,
                to: _,
                curved: _,
            } => from,
            Edge::Call { from, to: _ } => from,
            Edge::CallToReturn { from, to: _ } => from,
            Edge::Return { from, to: _ } => from,
            Edge::Path { from, to: _ } => from,
            Edge::Summary { from, to: _ } => from,
        }
    }

    /// Extract `to`'s [`Fact`] from the edge, no matter which variant.
    pub fn to(&self) -> &Fact {
        match self {
            Edge::Normal {
                from: _,
                to,
                curved: _,
            } => to,
            Edge::Call { from: _, to } => to,
            Edge::CallToReturn { from: _, to } => to,
            Edge::Return { from: _, to } => to,
            Edge::Path { from: _, to } => to,
            Edge::Summary { from: _, to } => to,
        }
    }
}

impl Graph {
    pub fn new() -> Self {
        Graph::default()
    }
}

/* 

#[cfg(test)]
mod test {
    use super::*;
    use crate::ir::ast::Function as AstFunction;

    #[test]
    fn adding_var_ok() {
        let mut graph = Graph::default();
        graph
            .init_function(
                &AstFunction {
                    name: "main".to_string(),
                    definitions: vec!["%0".to_string()],
                    ..Default::default()
                },
                0,
            )
            .unwrap();

        //assert_eq!(2, graph.facts.len());
        assert_eq!(1, graph.vars.len());
        assert_eq!(2, graph.vars.get(&"main".to_string()).unwrap().len());
    }

    #[test]
    fn adding_global() {
        let mut graph = Graph::default();
        graph
            .init_function(
                &AstFunction {
                    name: "main".to_string(),
                    definitions: vec!["%-1".to_string(), "%0".to_string()],
                    ..Default::default()
                },
                0,
            )
            .unwrap();

        assert_eq!(3, graph.vars.get(&"main".to_string()).unwrap().len());
        assert_eq!(
            &Variable {
                function: "main".to_string(),
                is_global: true,
                is_taut: false,
                name: "%-1".to_string(),
                is_memory: false,
                memory_offset: None,
            },
            graph.vars.get(&"main".to_string()).unwrap().get(1).unwrap()
        );
    }
}
*/