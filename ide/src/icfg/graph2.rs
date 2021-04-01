#![allow(dead_code)]

use crate::counter::Counter;
use crate::ir::ast::Function as AstFunction;
use anyhow::{Context, Result};
use log::debug;
use std::collections::HashMap;

type VarId = String;
type FunctionName = String;

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

#[derive(Debug, Clone)]
pub struct Note {
    pub id: usize,
    pub function: String,
    pub pc: usize,
    pub note: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Fact {
    pub id: usize,
    pub belongs_to_var: VarId,
    pub var_is_global: bool,
    pub var_is_taut: bool,
    pub next_pc: usize,
    pub track: usize,
    pub function: FunctionName,
}

#[derive(Debug)]
pub struct Function {
    pub name: FunctionName,
    pub definitions: usize,
    pub return_count: usize,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Variable {
    pub name: FunctionName,
    pub function: FunctionName,
    pub is_global: bool,
    pub is_taut: bool,
}

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

    pub fn init_function_fact(&mut self, function: String, pc: usize) -> Fact {
        let fact = Fact {
            id: self.fact_counter.get(),
            belongs_to_var: "taut".to_string(),
            var_is_taut: true,
            var_is_global: false,
            function,
            next_pc: pc,
            track: 0,
        };

        fact

        //self.facts.push(fact);
        //self.facts.get(self.facts.len() - 1).unwrap()
    }

    pub fn init_function_def(&mut self, function: &AstFunction) -> Result<()> {
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

    pub fn init_facts(
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
                id: self.fact_counter.get(),
                belongs_to_var: var.name.clone(),
                var_is_global: var.is_global,
                var_is_taut: var.is_taut,
                next_pc: pc,
                track: index,
                function: function.name.clone(),
            };

            //self.facts.push(fact.clone());
            facts.push(fact);

            index += 1;
        }

        Ok(facts)
    }

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

    pub fn get_track(&self, function: &String, variable: &String) -> Option<usize> {
        self.vars.get(function)?.iter().position(|x| &x.name == variable)
    }

    pub fn get_facts_at2(
        &self,
        function: &String,
        pc: usize,
    ) -> Result<impl Iterator<Item = &Fact>> {
        let function = function.clone();

        Ok(self
            .edges
            .iter()
            .filter(move |x| x.get_from().function == function && x.get_from().next_pc == pc)
            .map(|x| x.to()))
    }

    pub fn get_first_facts(&self, function: &String) -> Result<impl Iterator<Item = &Fact>> {
        let function = function.clone();
        let min = self
            .edges
            .iter()
            .filter(|x| x.get_from().function == function)
            .map(|x| x.get_from().next_pc)
            .min()
            .context("No minimum found")?;

        self.get_facts_at(&function, min)
    }

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
                id: self.fact_counter.get(),
                belongs_to_var: var.name.clone(),
                var_is_global: var.is_global,
                var_is_taut: var.is_taut,
                track,
                function: function.name.clone(),
                next_pc: pc,
            });

            if var.is_taut && pc < function.instructions.len(){
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
            id: self.fact_counter.get(),
            belongs_to_var: "taut".to_string(),
            var_is_taut: true,
            var_is_global: false,
            function,
            next_pc: pc,
            track: 0,
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
                name: "%-1".to_string()
            },
            graph.vars.get(&"main".to_string()).unwrap().get(1).unwrap()
        );
    }
}
