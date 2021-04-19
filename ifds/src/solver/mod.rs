//! The implementation for extracting the taints from the graph.

use crate::icfg::graph::{Edge, Graph};
use anyhow::{Context, Result};
use std::collections::HashSet;

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
    /// Get all taints for the graph. The edges must be already computed.
    /// The result depends on the `req` [`Request`]. Only the requested function and instruction which have
    /// an higher or equal program counter.
    fn all_sinks(&mut self, graph: &mut Graph, req: &Request) -> Result<Vec<Taint>>;

    /// Check if the statement at `req` and the statement `resp` have a taint.
    fn is_taint(&mut self, graph: &mut Graph, req: &Request, resp: &Request) -> Result<bool>;

    /// Return all tainted variables names at any statement for the graph. The edges must be already computed.
    /// The result depends on the `req` [`Request`]. Only the requested function and instruction which have
    /// an higher or equal program counter.
    fn sinks_var(&mut self, graph: &mut Graph, req: &Request) -> Result<HashSet<String>>;
}

impl Solver for IfdsSolver {
    fn all_sinks(&mut self, graph: &mut Graph, req: &Request) -> Result<Vec<Taint>> {
        assert!(req.variable.is_some());
        assert!(req.variable.as_ref().unwrap().starts_with("%"));

        let function = &req.function;

        let taints: Vec<_> = graph
            .edges
            .iter()
            .filter(|x| {
                matches!(x, Edge::Path { .. })
                    && &x.to().function == function
                    //&& x.get_from().var_is_taut
                    && x.get_from().next_pc == req.pc
            })
            .map(|x| x.to())
            .filter(|x| !x.var_is_taut)
            .map(|x| Taint {
                function: x.function.clone(),
                pc: x.next_pc.checked_sub(1).unwrap_or(0),
                variable: x.belongs_to_var.clone(),
            })
            .collect();

        Ok(taints)
    }

    fn sinks_var(&mut self, graph: &mut Graph, req: &Request) -> Result<HashSet<String>> {
        assert!(req.variable.is_some());
        assert!(req.variable.as_ref().unwrap().starts_with("%"));

        let function = &req.function;

        let taints = graph
            .edges
            .iter()
            .filter(|x| {
                matches!(x, Edge::Path { .. })
                    && &x.to().function == function
                    && x.get_from().next_pc == req.pc
            })
            .map(|x| x.to())
            .filter(|x| !x.var_is_taut)
            .map(|x| x.belongs_to_var.clone())
            .collect::<HashSet<_>>();

        Ok(taints)
    }

    fn is_taint(&mut self, graph: &mut Graph, req: &Request, resp: &Request) -> Result<bool> {
        let all_sinks = self
            .all_sinks(graph, req)
            .context("Cannot compute the sinks")?;

        let var = resp
            .variable
            .as_ref()
            .context("Please specify a variable for the response")?;

        Ok(all_sinks
            .into_iter()
            .find(|x| x.function == resp.function && x.pc == resp.pc && &x.variable == var)
            .is_some())
    }
}

pub trait GraphReachability {
    /// Get all taints for the graph. The edges must be already computed.
    /// The result depends on the `req` [`Request`]. Only the requested function and instruction which have
    /// an higher or equal program counter.
    fn all_sinks(&mut self, graph: &mut Graph, req: &Request) -> Vec<Taint>;
}
