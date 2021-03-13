use log::debug;

use crate::counter::Counter;
use crate::ssa::ast::Function as AstFunction;
use anyhow::{bail, Context, Result};
use std::collections::HashMap;

type VarId = String;
type FunctionName = String;

#[derive(Debug, Default)]
pub struct Graph {
    vars: HashMap<FunctionName, Vec<Variable>>,
    pub functions: HashMap<FunctionName, Function>,
    facts: Vec<Fact>,
    pub edges: Vec<Edge>,
    counter: Counter,
    epoch: Counter,
}

#[derive(Debug, Default)]
pub struct Variable {
    id: VarId,
    /// the predessors
    pub last_fact: Vec<Fact>,
    /// the first fact which defines the var
    first_fact: Option<Fact>,
    killed: bool,
}

#[derive(Debug)]
pub struct Function {
    pub name: String,
    first_facts: Vec<Fact>,
    pub last_facts: Vec<Fact>,
    pub results_len: usize,
}

#[derive(Debug, Default, Clone, PartialEq, PartialOrd, Eq, Ord, Hash)]
pub struct Fact {
    pub id: usize,
    pub note: String,
    pub belongs_to_var: String,
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
        Self::default()
    }

    pub fn init_function(&mut self, function: &AstFunction) {
        // add taut
        let mut vars = vec![Variable {
            id: "taut".to_string(),
            last_fact: vec![],
            ..Default::default()
        }];

        // add variables because parameters
        for param in function.params.iter() {
            vars.push(Variable {
                id: param.clone(),
                last_fact: vec![],
                ..Default::default()
            });
        }

        self.vars.insert(function.name.clone(), vars);

        // init function for tracking

        self.functions.insert(
            function.name.clone(),
            Function {
                name: function.name.clone(),
                first_facts: Vec::new(),
                last_facts: Vec::new(),
                results_len: function.results_len
            },
        );
    }

    fn new_fact(&mut self) -> Fact {
        let fact = Fact {
            id: self.counter.get(),
            note: "taut".to_string(),
            belongs_to_var: "taut".to_string(),
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

    /// Add a variable which is not derived from taut
    pub fn add_empty_vars(&mut self, function_name: &String, regs: &Vec<String>) -> Result<()> {
        for reg in regs.iter() {
            if self.vars.get(reg).is_none() {
                self.vars
                    .get_mut(function_name)
                    .context("Cannot find function's vars")?
                    .push(Variable {
                        id: reg.clone(),
                        last_fact: vec![],
                        killed: true,
                        ..Default::default()
                    });
            } else {
                bail!("Variable does already exist {}", reg);
            }
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

        if let Some(var) = self.get_mut_var(function_name, dest) {
            debug!("Variable is already defined");
            var.killed = true;
        } else {
            bail!("Variable does not exist");
        }

        Ok(())
    }

    fn get_var(&self, function_name: &String, name: &String) -> Option<&Variable> {
        self.vars.get(function_name)?.iter().find(|x| &x.id == name)
    }

    fn get_mut_var(&mut self, function_name: &String, name: &String) -> Option<&mut Variable> {
        self.vars
            .get_mut(function_name)?
            .iter_mut()
            .find(|x| &x.id == name)
    }

    pub fn add_call(
        &mut self,
        function_name: &String, //current function
        function: &AstFunction,
        name: &String,      //name of calling function
        regs: &Vec<String>, //passing arguments
    ) -> Result<()> {
        debug!("Add call {}", name);
        debug!("=> function {:#?}", function);

        let mut params_facts = {
            let mut facts = Vec::new();
            for param in vec!["taut".to_string()]
                .iter()
                .chain(function.params.iter())
            {
                if let Some(param_fact) = self
                    .get_mut_var(&function.name, param)
                    .and_then(|x| x.first_fact.as_ref())
                {
                    facts.push(param_fact.clone());
                } else {
                    bail!("Fact does not exist");
                }
            }

            facts
        };

        debug!("param facts are {:?}", params_facts);

        for (from_var, to_fact) in vec!["taut".to_string()]
            .iter()
            .chain(regs.iter())
            .zip(params_facts.iter())
        {
            if let Some(from) = self.get_var(function_name, from_var) {
                debug!(
                    "Creating call edge from={:?} to={:?}",
                    from.last_fact.get(0),
                    to_fact
                );

                assert!(from.last_fact.len() == 1, "Only one pred is allowed");
                //Call edges
                self.edges.push(Edge::Call {
                    from: from.last_fact.get(0).unwrap().clone(),
                    to: to_fact.clone(),
                });
            } else {
                bail!("Variable does not exist");
            }
        }

        Ok(())
    }

    pub fn add_row(
        &mut self,
        function_name: &String,
        note: String,
        killing_set: &mut Vec<Variable>,
    ) -> Result<()> {
        let mut facts = Vec::new();
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
                    belongs_to_var: format!("{}", var.id),
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

            if var.first_fact.is_none() {
                // Set it as first fact
                var.first_fact = Some(fact.clone());
            }

            var.last_fact = vec![fact.clone()];

            facts.push(fact);
        }

        if let Some(func_ref) = self.functions.get_mut(function_name) {
            func_ref.last_facts = facts;
        }

        Ok(())
    }

    pub fn add_call_to_return(
        &mut self,
        function_name: &String,
        note: String,
        killing_set: &mut Vec<Variable>,
    ) -> Result<Vec<Fact>> {
        let mut facts = Vec::new();
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
                    belongs_to_var: format!("{}", var.id),
                };

                self.facts.push(fact.clone());

                fact
            };

            if !var.killed {
                for node in var.last_fact.iter() {
                    debug!("Creating edge from={:?} to={:?}", node.id, fact.id);
                    //Normal
                    self.edges.push(Edge::CallToReturn {
                        from: node.clone(),
                        to: fact.clone(),
                    });
                }
            } else {
                debug!("Variable {} killed, therefore not creating edges", var.id);
            }

            facts.push(fact);
        }

        Ok(facts)
    }

    pub fn add_return(
        &mut self,
        src_facts: &Vec<Fact>,
        goal_facts: Vec<Fact>,
        dest_regs: &Vec<String>,
    ) -> Result<()> {
        debug!("Add return");

        let mut goals = Vec::with_capacity(dest_regs.len() + 1);

        // Add taut
        let taut = goal_facts.iter().find(|x| x.belongs_to_var == "taut".to_string()).context("Cannot find taut")?;
        goals.push(taut.clone());

        // Match other params
        for fact in goal_facts.into_iter() {
            debug!("Comparing {} with {:?}", fact.belongs_to_var, dest_regs);
            if dest_regs.contains(&fact.belongs_to_var) {
                goals.push(fact);
            }
        }

        assert_eq!(dest_regs.len() + 1, goals.len());

        debug!("The src facts of the callee are {:?}", src_facts);
        debug!("Filtered facts which match caller: {:?}", goals);
        for (src, target) in src_facts.iter().zip(goals) {
            debug!("Creating edge from={:?} to={:?}", src.id, target.id);
            self.edges.push(Edge::Return {
                from: src.clone(),
                to: target,
            });
        }

        Ok(())
    }
}
