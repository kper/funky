use crate::counter::Counter;
use anyhow::{bail, Result};

type VarId = String;

#[derive(Debug, Default)]
pub struct SubGraph {
    vars: Vec<Variable>,
    facts: Vec<Fact>,
    pub edges: Vec<Edge>,
    counter: Counter,
}

#[derive(Debug, Default)]
pub struct Variable {
    id: VarId,
    last_fact: usize,
}

#[derive(Debug, Default, Clone, PartialEq, PartialOrd, Eq, Ord)]
pub struct Fact {
    pub id: usize,
}

#[derive(Debug, Clone)]
pub enum Edge {
    Normal { from: usize, to: usize },
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

    fn new_fact(&mut self) -> usize {
        let fact = Fact {
            id: self.counter.get(),
        };

        self.facts.push(fact);

        self.counter.peek() - 1
    }

    fn get_taut_id(&self) -> usize {
        let taut = self.vars.get(0).unwrap();
        assert_eq!(taut.id, "taut".to_string(), "Expected to be tautology");

        taut.last_fact
    }

    fn get_fact(&self, val: &String) -> Result<usize> {
        let nodes = self
            .vars
            .iter()
            .rev()
            .filter(|x| &x.id == val)
            .collect::<Vec<_>>();

        if let Some(node) = nodes.get(0) {
            return Ok(node.last_fact);
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
        let src_node = self.get_fact(src)?;

        self.vars.push(Variable {
            id: dest.clone(),
            last_fact: src_node,
        });

        Ok(())
    }

    pub fn add_row(&mut self) {
        for var in self.vars.iter_mut() {
            // Create a new fact
            let fact = {
                let fact = Fact {
                    id: self.counter.get(),
                };

                self.facts.push(fact);

                self.counter.peek() - 1
            };

            //Normal
            self.edges.push(Edge::Normal {
                from: var.last_fact,
                to: fact,
            });

            var.last_fact = fact;
        }
    }
}
