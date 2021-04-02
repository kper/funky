use crate::icfg::graph::{Edge, Graph};
use anyhow::{Context, Result};
use std::collections::HashSet;

type PC = usize;

pub struct IfdsSolver;

#[derive(Debug)]
pub struct Taint {
    pub variable: String,
    pub function: String,
    pub pc: PC,
}

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
            .filter(|x| matches!(x, Edge::Path { .. }) && &x.to().function == function)
            .collect();

        let f2: Vec<_> = f1
            .into_iter()
            .map(|x| x.to())
            .filter(|x| (x.var_is_taut && x.next_pc == 0) || !x.var_is_taut)
            .collect();

        let taints = f2
            .into_iter()
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
