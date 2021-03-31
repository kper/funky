#![allow(dead_code)]

use crate::counter::Counter;
use crate::ir::ast::Function as AstFunction;
use crate::solver::Request;
use anyhow::{Context, Result};
use log::debug;
use std::collections::{HashMap};

type VarId = String;
type FunctionName = String;

#[derive(Debug, Default)]
pub struct Graph {
    pub vars: HashMap<FunctionName, Vec<Variable>>,
    pub functions: HashMap<FunctionName, Function>,
    pub facts: Vec<Fact>,
    pub edges: Vec<Edge>,
    pub notes: Vec<Note>,
    fact_counter: Counter,
    note_counter: Counter,
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
    pub pc: usize,
    pub track: usize,
    pub function: FunctionName,
    pub is_return: bool,
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

    /// Query graph by given Request.
    pub fn query(&self, req: &Request) -> Option<&Fact> {
        if let Some(var) = req.variable.as_ref() {
            self.facts
                .iter()
                .find(|x| &x.belongs_to_var == var && x.pc == req.pc && x.function == req.function)
        } else {
            self.facts
                .iter()
                .find(|x| x.pc == req.pc && x.function == req.function)
        }
    }

    /// Query graph by given fact_id.
    pub fn query_by_fact_id(&self, id: usize) -> Option<&Fact> {
        self.facts.iter().find(|x| x.id == id)
    }

    pub fn get_taut(&self, function: &String) -> Option<&Fact> {
        self.facts
            .iter()
            .find(|x| x.var_is_taut && &x.function == function)
    }

    pub fn init_function_fact(&mut self, function: String, pc: usize) -> &Fact {
        let fact = Fact {
            id: self.fact_counter.get(),
            belongs_to_var: "taut".to_string(),
            var_is_taut: true,
            var_is_global: false,
            function,
            pc,
            track: 0,
            is_return: false,
        };

        self.facts.push(fact);
        self.facts.get(self.facts.len() - 1).unwrap()
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
                .facts
                .iter()
                .filter(|x| x.function == function.name)
                .map(|x| x.pc)
                .min()
                .context("No facts found")?;

            if min_pc <= pc {
                // no self recursion

                // Return the first facts of the function.
                return Ok(self
                    .facts
                    .iter()
                    .filter(|x| x.function == function.name && x.pc == pc)
                    .cloned()
                    .collect());
            }
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

        // generate of all facts

        for (i, instruction) in function.instructions.iter().enumerate() {
            self.add_statement(function, format!("{:?}", instruction), i + 1)?;
        }

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
                pc,
                is_return: false,
                track: index,
                function: function.name.clone(),
            };

            self.facts.push(fact.clone());
            facts.push(fact);

            index += 1;
        }

        Ok(facts)
    }

    pub fn get_facts_at(&self, function: &String, pc: usize) -> Result<Vec<&Fact>> {
        let facts = self
            .facts
            .iter()
            .filter(|x| &x.function == function && x.pc == pc)
            .collect::<Vec<_>>();

        Ok(facts)
    }

    pub fn new_fact(&mut self, fact: Fact) -> Result<()> {
        self.facts.push(fact);

        Ok(())
    }

    pub fn add_statement(
        &mut self,
        function: &AstFunction,
        instruction: String,
        pc: usize,
        //variable: &String,
    ) -> Result<()> {
        debug!("Adding statement {:?}", instruction);

        let vars = self
            .vars
            .get(&function.name)
            .context("Cannot get functions's vars")?
            .clone();
        let vars = vars.iter().enumerate();

        //.filter(|x| &x.1.name == variable);

        //let pc = self.pc_counter.get();
        //debug!("New pc {} for {}", pc, function.name);

        for (track, var) in vars {
            debug!("Adding new fact for {}", var.name);

            self.facts.push(Fact {
                id: self.fact_counter.get(),
                belongs_to_var: var.name.clone(),
                var_is_global: var.is_global,
                var_is_taut: var.is_taut,
                track,
                function: function.name.clone(),
                pc,
                is_return: false,
            });
        }

        self.notes.push(Note {
            id: self.note_counter.get(),
            function: function.name.clone(),
            pc,
            note: instruction.clone(),
        });

        Ok(())
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

        assert_eq!(2, graph.facts.len());
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
