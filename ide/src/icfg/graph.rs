use log::debug;
use std::ops::DerefMut;

use crate::counter::Counter;
use crate::ssa::ast::Function;
use anyhow::{bail, private::kind, Context, Result};
use std::collections::HashMap;

type VarId = String;

#[derive(Debug, Default)]
pub struct Graph {
    vars: HashMap<String, Vec<Variable>>,
    facts: Vec<Fact>,
    pub edges: Vec<Edge>,
    counter: Counter,
    epoch: Counter,
}

#[derive(Debug, Default)]
pub struct Variable {
    id: VarId,
    last_fact: Vec<Fact>,
    killed: bool,
}

#[derive(Debug, Default, Clone, PartialEq, PartialOrd, Eq, Ord)]
pub struct Fact {
    pub id: usize,
    pub note: String,
}

#[derive(Debug, Clone)]
pub enum Edge {
    Normal { from: Fact, to: Fact },
}

impl Graph {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn init_function(&mut self, function: &Function) {
        let fact = self.new_fact();
        let vars = vec![Variable {
            id: "taut".to_string(),
            last_fact: vec![fact],
            ..Default::default()
        }];
        self.vars.insert(function.name.clone(), vars);
    }

    fn new_fact(&mut self) -> Fact {
        let fact = Fact {
            id: self.counter.get(),
            note: "taut".to_string(),
        };

        self.facts.push(fact.clone());

        fact
    }

    fn get_taut_id(&self, function_name: &String) -> Result<Vec<Fact>> {
        let taut = self
            .vars
            .get(function_name)
            .context("Cannot find function's vars")?
            .get(0)
            .unwrap();
        assert_eq!(taut.id, "taut".to_string(), "Expected to be tautology");

        Ok(taut.last_fact.clone())
    }

    fn get_fact(&self, function_name: &String, val: &String) -> Result<Vec<Fact>> {
        let nodes = self
            .vars
            .get(function_name)
            .context("Cannot find function's vars")?
            .iter()
            .rev()
            .filter(|x| &x.id == val)
            .collect::<Vec<_>>();

        if let Some(node) = nodes.get(0) {
            return Ok(node.last_fact.clone());
        }

        bail!("Fact for {} not found", val);
    }

    /// add a new node in the graph from taut
    pub fn add_var(
        &mut self,
        function_name: &String,
        reg: &String,
        killing_set: &mut Vec<Variable>,
    ) -> Result<()> {
        let len = self.vars.len();
        let fact = self.get_taut_id(function_name)?;

        if let Some(var) = self
            .vars
            .get_mut(function_name)
            .context("Cannot find function's vars")?
            .iter_mut()
            .filter(|x| &x.id == reg)
            .collect::<Vec<_>>()
            .get_mut(0)
        {
            //killing_set.push(reg.clone());
            var.last_fact = fact;
        } else {
            // Get the last tautology fact
            let fact = self.get_taut_id(function_name)?;
            self.vars
                .get_mut(function_name)
                .context("Cannot find function's vars")?
                .push(Variable {
                    id: reg.clone(),
                    last_fact: fact,
                    ..Default::default()
                });
        }

        Ok(())
    }

    /// add assignment
    pub fn add_assignment(
        &mut self,
        function_name: &String,
        dest: &String,
        src: &String,
        killing_set: &mut Vec<Variable>,
    ) -> Result<()> {
        debug!("add assignment src={} dest={}", src, dest);
        let src_node = self
            .get_fact(function_name, src)
            .context("Could not add assignment")?;

        if let Some(var) = self
            .vars
            .get_mut(function_name)
            .context("Cannot find function's vars")?
            .iter_mut()
            .filter(|x| &x.id == dest)
            .collect::<Vec<_>>()
            .get_mut(0)
        {
            debug!("Variable is already defined");
            var.last_fact = src_node;
        } else {
            debug!("Variable does not exist");
            // dest does not exist
            self.vars
                .get_mut(function_name)
                .context("Cannot find function's vars")?
                .push(Variable {
                    id: dest.clone(),
                    last_fact: src_node,
                    ..Default::default()
                });
        }

        Ok(())
    }

    /// add unop
    pub fn add_unop(
        &mut self,
        function_name: &String,
        dest: &String,
        src: &String,
        killing_set: &mut Vec<Variable>,
    ) -> Result<()> {
        debug!("Unop src={} dest={}", src, dest);

        let src_node = self
            .get_fact(function_name, src)
            .context("Could not unop assignment")?;

        if let Some(var) = self
            .vars
            .get_mut(function_name)
            .context("Cannot find function's vars")?
            .iter_mut()
            .filter(|x| &x.id == dest)
            .collect::<Vec<_>>()
            .get_mut(0)
        {
            debug!("Variable is already defined");
            var.last_fact = src_node;
        } else {
            debug!("Variable does not exist");
            // dest does not exist
            self.vars
                .get_mut(function_name)
                .context("Cannot find function's vars")?
                .push(Variable {
                    id: dest.clone(),
                    last_fact: src_node,
                    ..Default::default()
                });
        }

        Ok(())
    }

    /// add binop
    pub fn add_binop(
        &mut self,
        function_name: &String,
        dest: &String,
        src1: &String,
        src2: &String,
        killing_set: &mut Vec<Variable>,
    ) -> Result<()> {
        debug!("Binop src1={} src2={} dest={}", src1, src2, dest);

        let mut src_node = self
            .get_fact(function_name, src1)
            .context("Could not binop assignment")?;
        let mut src_node2 = self
            .get_fact(function_name, src2)
            .context("Could not binop assignment")?;

        src_node.append(&mut src_node2);

        debug!("src nodes are {:?}", src_node);

        if let Some(var) = self
            .vars
            .get_mut(function_name)
            .context("Cannot find function's vars")?
            .iter_mut()
            .filter(|x| &x.id == dest)
            .collect::<Vec<_>>()
            .get_mut(0)
        {
            debug!("Variable is already defined");
            var.last_fact = src_node;
        } else {
            debug!("Variable does not exist");
            // dest does not exist
            self.vars
                .get_mut(function_name)
                .context("Cannot find function's vars")?
                .push(Variable {
                    id: dest.clone(),
                    last_fact: src_node,
                    ..Default::default()
                });
        }

        Ok(())
    }

    pub fn kill_var(
        &mut self,
        function_name: &String,
        dest: &String,
        killing_set: &mut Vec<Variable>,
    ) -> Result<()> {
        debug!("Killing var={}", dest);

        if let Some(var) = self
            .vars
            .get_mut(function_name)
            .context("Cannot find function's vars")?
            .iter_mut()
            .filter(|x| &x.id == dest)
            .collect::<Vec<_>>()
            .get_mut(0)
        {
            debug!("Variable is already defined");
            var.killed = true;
        } else {
            bail!("Variable does not exist");
        }

        Ok(())
    }

    pub fn add_call(&mut self, name: &String) -> Result<()> {
        Ok(())
    }

    pub fn add_row(
        &mut self,
        function_name: &String,
        note: String,
        killing_set: &mut Vec<Variable>,
    ) -> Result<()> {
        let epoch = self.epoch.get();
        for var in self
            .vars
            .get_mut(function_name)
            .context("Cannot find function's vars")?
            .iter_mut()
        {
            // Create a new fact
            let fact = {
                let fact = Fact {
                    id: self.counter.get(),
                    note: format!("<b>{}</b> at {}<br/>{}", var.id, epoch, note),
                };

                self.facts.push(fact.clone());

                fact
            };

            if !var.killed {
                for node in var.last_fact.iter() {
                    debug!("Creating edge from={:?} to={:?}", node.id, fact.id);
                    //Normal
                    self.edges.push(Edge::Normal {
                        from: node.clone(),
                        to: fact.clone(),
                    });
                }
            } else {
                debug!("Variable {} killed, therefore not creating edges", var.id);
            }

            var.last_fact = vec![fact];
        }

        Ok(())
    }
}
