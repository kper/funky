//! The implementation for extracting the taints from the graph.

use crate::icfg::graph::{Edge, Graph};
use anyhow::Result;

type PC = usize;

pub struct IfdsSolver;

/// Represents the taints from the execution of the solver.
#[derive(Debug)]
pub struct Taint {
    pub variable: String,
    pub function: String,
    pub pc: PC,
}

/// Request is the datastructure which defines at what instruction the [`Graph`] should be build from.
#[derive(Debug, Clone)]
pub struct Request {
    pub variable: Option<String>,
    pub function: String,
    pub pc: PC,
}

pub trait Solver {
    /// Return all sinks of the `req`
    fn all_sinks(&mut self, graph: &mut Graph, req: &Request) -> Result<Vec<Taint>>;
}

impl Solver for IfdsSolver {
    fn all_sinks(&mut self, graph: &mut Graph, req: &Request) -> Result<Vec<Taint>> {
        assert!(req.variable.is_some());
        assert!(req.variable.as_ref().unwrap().starts_with("%"));

        let function = &req.function;

        let f1: Vec<_> = graph
            .edges
            .iter()
            .filter(|x| {
                matches!(x, Edge::Path { .. })
                    && &x.to().function == function
                    //&& x.get_from().var_is_taut
                    && x.get_from().next_pc == req.pc
            })
            .collect();

        let f2: Vec<_> = f1
            .iter()
            .map(|x| x.to())
            .filter(|x| !x.var_is_taut)
            .collect();

        let taints = f2
            .iter()
            .map(|x| Taint {
                function: x.function.clone(),
                pc: x.next_pc,
                variable: x.belongs_to_var.clone(),
            })
            .collect();

        Ok(taints)
    }
}

pub trait GraphReachability {
    fn all_sinks(&mut self, graph: &mut Graph, req: &Request) -> Vec<Taint>;
}
