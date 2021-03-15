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

    fn get_vars(&self, function_name: &String) -> Option<&Vec<Variable>> {
        self.vars.get(function_name)
    }

    fn get_vars_mut(&mut self, function_name: &String) -> Option<&mut Vec<Variable>> {
        self.vars.get_mut(function_name)
    }

    fn get_var(&self, function_name: &String, var: &String) -> Option<&Variable> {
        self.get_vars(function_name)?
            .iter()
            .find(|x| &x.name == var)
    }

    fn get_var_mut(&mut self, function_name: &String, var: &String) -> Option<&mut Variable> {
        self.get_vars_mut(function_name)?
            .iter_mut()
            .find(|x| &x.name == var)
    }

    fn new_var(&mut self, function_name: &String, var: Variable) -> Result<()> {
        debug!("Adding new var {} to function {}", var.name, function_name);

        let vars = self.get_vars_mut(function_name).context("Cannot get variables")?;

        if vars.iter().find(|x| x.name == var.name).is_none() {
            // No other variable defined
            vars.push(var);
        }
        else {
            bail!("Variable {} is already defined", var.name);
        }

        Ok(())
    }

    pub fn init_function(&mut self, function: &AstFunction) -> Result<()> {
        debug!("Adding new function {} to the graph", function.name);

        self.functions.insert(
            function.name.clone(),
            Function {
                name: function.name.clone(),
            },
        );

        self.vars.insert(function.name.clone(), vec![Variable {
            name: "taut".to_string(),
            function: function.name.clone(),
        }]);

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
        graph.init_function(&AstFunction {
            name: "main".to_string(),
            ..Default::default()
        }).unwrap();
        
        let var = Variable {
            name: "%0".to_string(),
            function: "main".to_string()
        };

        assert!(graph.new_var(&"main".to_string(), var).is_ok());
        assert_eq!(1, graph.vars.len());
        assert_eq!(2, graph.vars.get(&"main".to_string()).unwrap().len());

    }

    #[test]
    fn adding_duplicated_var() {
        let mut graph = Graph::default();
        graph.init_function(&AstFunction {
            name: "main".to_string(),
            ..Default::default()
        }).unwrap();
        
        let var = Variable {
            name: "taut".to_string(),
            function: "main".to_string()
        };

        assert!(graph.new_var(&"main".to_string(), var).is_err());
    }
}