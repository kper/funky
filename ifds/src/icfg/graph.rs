#![allow(dead_code)]

use anyhow::Result;

type VarId = String;
type FunctionName = String;

/// The datastructure for the graph.
#[derive(Debug, Default)]
pub struct Graph {
    pub edges: Vec<Edge>,
}

impl Graph {
    /// Adding a normal edge to the graph
    pub fn add_normal(&mut self, from: Fact, to: Fact) -> Result<()> {
        self.edges.push(Edge::Normal {
            curved: false,
            from,
            to,
        });

        Ok(())
    }

    /// Adding a normal edge to the graph, which is curved.
    /// The curving indicates that it is a jump.
    pub fn add_normal_curved(&mut self, from: Fact, to: Fact) -> Result<()> {
        self.edges.push(Edge::Normal {
            curved: true,
            from,
            to,
        });

        Ok(())
    }

    /// Adding a call-to-return edge between the given facts.
    pub fn add_call_to_return_edge(&mut self, from: Fact, to: Fact) -> Result<()> {
        self.edges.push(Edge::CallToReturn { from, to });

        Ok(())
    }

    /// Adding a call edge between the given facts.
    pub fn add_call(&mut self, from: Fact, to: Fact) -> Result<()> {
        self.edges.push(Edge::Call { from, to });

        Ok(())
    }

    /// Adding a return edge between the given facts.
    pub fn add_return(&mut self, from: Fact, to: Fact) -> Result<()> {
        self.edges.push(Edge::Return { from, to });

        Ok(())
    }
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
#[derive(Debug, Clone, PartialEq, PartialOrd, Default)]
pub struct Fact {
    pub belongs_to_var: VarId,
    pub var_is_global: bool,
    pub var_is_taut: bool,
    pub var_is_memory: bool,
    pub pc: usize,
    /// determine what the next pc is
    pub next_pc: usize,
    pub track: usize,
    pub function: FunctionName,
    /// if the fact saves a memory variable
    /// then save the offset.
    pub memory_offset: Option<f64>,
}

impl Fact {
    /// Build a new fact from a given variable
    pub fn from_var(var: &Variable, pc: usize, next_pc: usize, track: usize) -> Fact {
        Fact {
            belongs_to_var: var.name.clone(),
            function: var.function.clone(),
            pc,
            next_pc,
            track,
            memory_offset: var.memory_offset.clone(),
            var_is_global: var.is_global,
            var_is_taut: var.is_taut,
            var_is_memory: var.is_memory,
        }
    }
}

/// An IFDS representation for a function.
#[derive(Debug)]
pub struct Function {
    pub name: FunctionName,
    pub definitions: usize,
    pub return_count: usize,
}

/// The register which will be used at some point in the module.
#[derive(Debug, Clone, PartialEq, Default)]
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

    /// Checks if edge is a normal edge
    pub fn is_normal(&self) -> bool {
        match self {
            Edge::Normal {
                from: _,
                to:_,
                curved: _,
            } => true,
            _ => false,
        }
    }

    /// Checks if edge is a call edge
    pub fn is_call(&self) -> bool {
        match self {
            Edge::Call {
                from: _,
                to: _,
            } => true,
            _ => false,
        }
    }

    /// Checks if edge is a return edge
    pub fn is_return(&self) -> bool {
        match self {
            Edge::Return {
                from: _,
                to: _,
            } => true,
            _ => false,
        }
    }
}
