use crate::counter::{Counter, StackedCounter};
use crate::ssa::ast::Function as AstFunction;
use crate::ssa::ast::Instruction;
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
    pub pc_counter: Counter,
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

#[derive(Debug, Clone)]
pub struct Fact {
    pub id: usize,
    pub belongs_to_var: VarId,
    pub pc: usize,
    pub track: usize,
    pub function: FunctionName,
}

#[derive(Debug)]
pub struct Function {
    pub name: FunctionName,
    pub definitions: usize,
    pub return_count: usize,
}

#[derive(Debug, Clone)]
pub struct Variable {
    pub name: FunctionName,
    pub function: FunctionName,
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

    pub fn get_vars(&self, function_name: &String) -> Option<&Vec<Variable>> {
        self.vars.get(function_name)
    }

    fn get_vars_mut(&mut self, function_name: &String) -> Option<&mut Vec<Variable>> {
        self.vars.get_mut(function_name)
    }

    pub fn get_var(&self, function_name: &String, var: &String) -> Option<&Variable> {
        self.get_vars(function_name)?
            .iter()
            .find(|x| &x.name == var)
    }

    fn get_var_mut(&mut self, function_name: &String, var: &String) -> Option<&mut Variable> {
        self.get_vars_mut(function_name)?
            .iter_mut()
            .find(|x| &x.name == var)
    }

    pub fn get_first_fact_of_var(&self, variable: &Variable) -> Option<&Fact> {
        self.facts
            .iter()
            .find(|x| x.belongs_to_var == variable.name && x.function == variable.function)
    }

    pub fn get_last_fact_of_var(&self, variable: &Variable) -> Option<&Fact> {
        self.facts
            .iter()
            .rev()
            .find(|x| x.belongs_to_var == variable.name && x.function == variable.function)
    }

    pub fn init_function(&mut self, function: &AstFunction) -> Result<()> {
        debug!("Adding new function {} to the graph", function.name);

        self.functions.insert(
            function.name.clone(),
            Function {
                name: function.name.clone(),
                definitions: function.definitions.len(),
                return_count: function.results_len,
            },
        );

        let mut variables = Vec::with_capacity(function.definitions.len() + 1);

        variables.push(Variable {
            name: "taut".to_string(),
            function: function.name.clone(),
        });

        // add definitions
        for reg in function.definitions.iter() {
            debug!("Adding definition {}", reg);
            variables.push(Variable {
                name: reg.clone(),
                function: function.name.clone(),
            });
        }

        self.init_facts(function, &mut variables)
            .context("Cannot initialize facts")?;

        self.vars.insert(function.name.clone(), variables);

        Ok(())
    }

    fn init_facts(&mut self, function: &AstFunction, variables: &mut Vec<Variable>) -> Result<()> {
        debug!("Initializing facts for function {}", function.name);

        let mut index = 0;
        for var in variables {
            debug!("Creating fact for var {}", var.name);

            self.facts.push(Fact {
                id: self.fact_counter.get(),
                belongs_to_var: var.name.clone(),
                pc: 0,
                track: index,
                function: function.name.clone(),
            });

            index += 1;
        }

        self.pc_counter.get();

        Ok(())
    }

    pub fn add_statement(
        &mut self,
        function: &AstFunction,
        instruction: &Instruction,
    ) -> Result<()> {
        debug!("Adding statement");

        let vars = self
            .get_vars(&function.name)
            .context("Cannot get functions's vars")?
            .clone();
        let vars = vars.iter().enumerate();

        let pc = self.pc_counter.get();
        debug!("New pc {} for {}", pc, function.name);

        for (track, var) in vars {
            debug!("Adding new fact for {}", var.name);

            self.facts.push(Fact {
                id: self.fact_counter.get(),
                belongs_to_var: var.name.clone(),
                track,
                function: function.name.clone(),
                pc,
            });
        }

        // Adding stmt note

        self.notes.push(Note {
            id: self.note_counter.get(),
            function: function.name.clone(),
            pc,
            note: format!("{:?}", instruction),
        });

        Ok(())
    }

    /// Add a normal edge from the fact `from` to the fact `to`.
    pub fn add_normal(&mut self, from: Fact, to: Fact) -> Result<()> {
        self.edges.push(Edge::Normal { from, to });

        Ok(())
    }

    /// Add a call edge from the fact `from` to the fact `to`.
    pub fn add_call_edge(&mut self, from: Fact, to: Fact) -> Result<()> {
        self.edges.push(Edge::Call { from, to });

        Ok(())
    }

    /// Add a return edge from the fact `from` to the fact `to`.
    pub fn add_return_edge(&mut self, from: Fact, to: Fact) -> Result<()> {
        self.edges.push(Edge::Return { from, to });

        Ok(())
    }

    /// Add a call-to-return edge from the fact `from` to the fact `to`.
    pub fn add_call_to_return_edge(&mut self, from: Fact, to: Fact) -> Result<()> {
        self.edges.push(Edge::CallToReturn { from, to });

        Ok(())
    }
}



#[cfg(test)]
mod test {
    use super::*;
    use crate::ssa::ast::Function as AstFunction;

    #[test]
    fn adding_var_ok() {
        let mut graph = Graph::default();
        graph
            .init_function(&AstFunction {
                name: "main".to_string(),
                definitions: vec!["%0".to_string()],
                ..Default::default()
            })
            .unwrap();

        assert_eq!(2, graph.facts.len());
        assert_eq!(1, graph.vars.len());
        assert_eq!(2, graph.vars.get(&"main".to_string()).unwrap().len());
    }
}
