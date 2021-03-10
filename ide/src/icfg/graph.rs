use crate::counter::Counter;
use anyhow::{bail, Result, Context};

type VarId = String;

#[derive(Debug, Default)]
pub struct SubGraph {
    vars: Vec<Variable>,
    facts: Vec<Fact>,
    pub edges: Vec<Edge>,
    counter: Counter,
    epoch: Counter,
}

#[derive(Debug, Default)]
pub struct Variable {
    id: VarId,
    last_fact: Fact,
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

impl SubGraph {
    pub fn new() -> Self {
        let mut graph = Self::default();
        let fact = graph.new_fact();
        graph.vars.push(Variable {
            id: "taut".to_string(),
            last_fact: fact,
        });
        graph
    }

    fn new_fact(&mut self) -> Fact {
        let fact = Fact {
            id: self.counter.get(),
            note: "taut".to_string(),
        };

        self.facts.push(fact.clone());

        fact
    }

    fn get_taut_id(&self) -> Fact {
        let taut = self.vars.get(0).unwrap();
        assert_eq!(taut.id, "taut".to_string(), "Expected to be tautology");

        taut.last_fact.clone()
    }

    fn get_fact(&self, val: &String) -> Result<Fact> {
        let nodes = self
            .vars
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
    pub fn add_var(&mut self, reg: &String) -> &mut Variable {
        let len = self.vars.len();

        // Get the last tautology fact
        let fact = self.get_taut_id();
        self.vars.push(Variable {
            id: reg.clone(),
            last_fact: fact,
        });
        self.vars.get_mut(len).unwrap()
    }

    /// add assignment
    pub fn add_assignment(&mut self, dest: &String, src: &String) -> Result<()> {
        let src_node = self.get_fact(src).context("Could not add assignment")?;

        self.vars.push(Variable {
            id: dest.clone(),
            last_fact: src_node,
        });

        Ok(())
    }

    pub fn add_row(&mut self, note: String) {
        let epoch = self.epoch.get();
        for var in self.vars.iter_mut() {
            // Create a new fact
            let fact = {
                let fact = Fact {
                    id: self.counter.get(),
                    note: format!("<b>{}</b> at {}<br/>{}", var.id, epoch, note),
                };

                self.facts.push(fact.clone());

                fact
            };

            //Normal
            self.edges.push(Edge::Normal {
                from: var.last_fact.clone(),
                to: fact.clone(),
            });

            var.last_fact = fact;
        }
    }
}
