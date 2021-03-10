use crate::counter::Counter;

type VarId = usize;

#[derive(Debug, Default)]
pub struct SubGraph {
    vars: Vec<Variable>,
    edges: Vec<Edge>,
    counter: Counter,
}

#[derive(Debug, Default)]
pub struct Variable {
    id: VarId,
}

#[derive(Debug, Default)]
pub struct Fact;

#[derive(Debug)]
pub enum Edge {
    Normal { from: VarId, to: VarId },
}

impl SubGraph {
    pub fn new() -> Self {
        let mut graph = Self::default();
        graph.add_var(); // tautology

        graph
    }

    /// add a new node in the graph
    pub fn add_var(&mut self) -> &mut Variable {
        let counter = self.counter.get();

        self.vars.push(Variable { id: counter });

        let len = self.vars.len() - 1;
        self.vars.get_mut(len).unwrap()
    }
}

impl Variable {
    pub fn normal(&self, graph: &mut SubGraph, var: &mut Variable) {
        graph.edges.push(Edge::Normal {
            from: self.id,
            to: var.id,
        })
    }
}
