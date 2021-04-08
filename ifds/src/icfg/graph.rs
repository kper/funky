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
    pub vars: HashMap<FunctionName, Vec<Variable>>,
    pub functions: HashMap<FunctionName, Function>,
    //pub facts: Vec<Fact>,
    pub edges: Vec<Edge>,
    pub notes: Vec<Note>,
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

    pub fn is_function_defined(&self, name: &String) -> bool {
        self.functions.get(name).is_some()
    }

    pub fn get_taut(&self, function: &String) -> Option<&Fact> {
        self.edges
            .iter()
            .find(|x| x.get_from().var_is_taut && &x.get_from().function == function)
            .map(|x| x.get_from())
    }

    fn init_function_def(&mut self, function: &AstFunction) -> Result<()> {
        self.functions.insert(
            function.name.clone(),
            Function {
                name: function.name.clone(),
                definitions: function.definitions.len(),
                return_count: function.results_len,
            },
        );

        Ok(())
    }

    /// Add a memory variable to the graph's variables
    pub fn add_memory_var(&mut self, variable: String, function: String, offset: f64) -> Variable {
        let name = format!("{}@{}", variable, offset);
        let var = Variable {
            function: function.clone(),
            is_global: false,
            is_memory: true,
            is_taut: false,
            name,
            memory_offset: Some(offset),
        };

        if let Some(vars) = self.vars.get_mut(&function) {
            if !vars.contains(&var) {
                vars.push(var.clone());
            }
        }
        else {
            self.vars.insert(function, vec![var.clone()]);
        }

        var 
    }

    /// Initialise the function.
    pub fn init_function(&mut self, function: &AstFunction, pc: usize) -> Result<Vec<Fact>> {
        debug!("Adding new function {} to the graph", function.name);

        if let Some(function) = self.functions.get(&function.name) {
            debug!("Function was already initialized");

            // Handle edge case when the smallest fact greater than `pc`.
            // This might happen if you the user starts the analysis from not `0`,
            // but there is a self recursive call.

            let min_pc = self
                .edges
                .iter()
                .filter(|x| x.get_from().function == function.name)
                .map(|x| x.get_from().next_pc)
                .min()
                .context("No facts found")?;

            if min_pc <= pc {
                // no self recursion

                // Return the first facts of the function.
                return Ok(self
                    .init_facts
                    .get(&function.name)
                    .context("Cannot get function's stored init facts")?
                    .clone());
            }

            // else reinitalize the function.
        }

        self.init_function_def(function)?;

        let mut variables = Vec::with_capacity(function.definitions.len() + 1);

        variables.push(Variable {
            name: "taut".to_string(),
            function: function.name.clone(),
            is_global: false,
            is_taut: true,
            is_memory: false,
            memory_offset: None,
        });

        // add definitions
        for reg in function.definitions.iter() {
            debug!("Adding definition {}", reg);

            let reg_num: isize = reg
                .clone()
                .split_off(1)
                .parse()
                .context("Cannot parse reg to number")?;
            let is_global = match reg_num {
                x if x < 0 => true,
                x if x >= 0 => false,
                _ => unreachable!(""),
            };

            variables.push(Variable {
                name: reg.clone(),
                function: function.name.clone(),
                is_global,
                is_taut: false,
                is_memory: false,
                memory_offset: None,
            });
        }

        let facts = self
            .init_facts(function, &mut variables, pc)
            .context("Cannot initialize facts")?;

        self.vars.insert(function.name.clone(), variables);

        // insert the initial facts or update them.
        self.init_facts.insert(function.name.clone(), facts.clone());

        Ok(facts)
    }

    fn init_facts(
        &mut self,
        function: &AstFunction,
        variables: &mut Vec<Variable>,
        pc: usize,
    ) -> Result<Vec<Fact>> {
        debug!("Initializing facts for function {}", function.name);

        let mut index = 0;
        let mut facts = Vec::with_capacity(variables.len());
        for var in variables {
            debug!("Creating fact for var {}", var.name);

            let fact = Fact {
                belongs_to_var: var.name.clone(),
                var_is_global: var.is_global,
                var_is_taut: var.is_taut,
                var_is_memory: var.is_memory,
                next_pc: pc,
                track: index,
                function: function.name.clone(),
                memory_offset: var.memory_offset,
            };

            //self.facts.push(fact.clone());
            facts.push(fact);

            index += 1;
        }

        Ok(facts)
    }

    /// Get the facts at a certain instruction for a given function.
    pub fn get_facts_at(
        &self,
        function: &String,
        pc: usize,
    ) -> Result<impl Iterator<Item = &Fact>> {
        let function = function.clone();

        // changed to `to()` because all edges start usually at root
        Ok(self
            .edges
            .iter()
            .filter(move |x| x.get_from().function == function && x.to().next_pc == pc)
            .map(|x| x.to()))
    }

    /// Get the track by given name and function.
    pub fn get_track(&self, function: &String, variable: &String) -> Option<usize> {
        self.vars
            .get(function)?
            .iter()
            .position(|x| &x.name == variable)
    }

    /// Get a variable by given name and function.
    pub fn get_var(&self, function: &String, variable: &String) -> Option<&Variable> {
        self.vars
            .get(function)?
            .iter()
            .find(|x| &x.name == variable)
    }

    /// Add a new statement to the graph.
    pub fn add_statement(
        &mut self,
        function: &AstFunction,
        instruction: String,
        pc: usize,
        variable: &String,
    ) -> Result<Vec<Fact>> {
        debug!(
            "Adding statement {} at {} for {} ({})",
            instruction, pc, variable, function.name
        );

        let vars = self
            .vars
            .get(&function.name)
            .context("Cannot get functions's vars")?
            .clone();
        let mut vars = vars.iter().enumerate();

        let mut facts = Vec::new();

        for (track, var) in vars.find(|x| &x.1.name == variable) {
            debug!("Adding new fact for {}", var.name);

            facts.push(Fact {
                belongs_to_var: var.name.clone(),
                var_is_global: var.is_global,
                var_is_taut: var.is_taut,
                var_is_memory: var.is_memory,
                track,
                function: function.name.clone(),
                next_pc: pc,
                memory_offset: var.memory_offset,
            });
        }

        Ok(facts)
    }

    /// Add a statement with the instruction with a note [`Note`]. 
    /// The notes has the instruction as content, which makes it easier to read in the
    /// `tikz` representation.
    pub fn add_statement_with_note(
        &mut self,
        function: &AstFunction,
        instruction: String,
        pc: usize,
        variable: &String,
    ) -> Result<Vec<Fact>> {
        let facts = self
            .add_statement(function, instruction.clone(), pc, variable)
            .context("While add statement with note")?;

        let vars = self
            .vars
            .get(&function.name)
            .context("Cannot get functions's vars")?
            .clone();
        let mut vars = vars.iter().enumerate();

        for (_track, var) in vars.find(|x| &x.1.name == variable) {
            debug!("Adding new fact for {}", var.name);

            if var.is_taut && pc < function.instructions.len() {
                self.notes.push(Note {
                    id: self.note_counter.get(),
                    function: function.name.clone(),
                    pc,
                    note: instruction.clone(),
                });
            }
        }

        Ok(facts)
    }

    /// Create new tautological fact by given function and pc.
    pub fn taut(&mut self, function: String, pc: usize) -> Fact {
        Fact {
            belongs_to_var: "taut".to_string(),
            var_is_taut: true,
            var_is_global: false,
            var_is_memory: false,
            function,
            next_pc: pc,
            track: 0,
            memory_offset: None,
        }
    }
}

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
