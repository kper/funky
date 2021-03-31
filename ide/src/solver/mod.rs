use crate::icfg::graph2::Graph;
use anyhow::{bail, Context, Result};

//pub mod bfs;

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
        let function = &req.function;
        let variable = req
            .variable
            .as_ref()
            .context("Request needs to have a specified variable")?;
        let first_pc = graph
            .edges
            .iter()
            .filter(|x| {
                &x.get_from().function == function && &x.get_from().belongs_to_var == variable
            })
            .map(|x| x.get_from().pc)
            .min();

        if let Some(first_pc) = first_pc {
            let start = graph
                .edges
                .iter()
                .map(|x| x.get_from())
                .find(|x| {
                    &x.function == function
                        && x.pc == first_pc
                        && (x.var_is_taut || &x.belongs_to_var == variable)
                })
                .context("Cannot find taut")?;

            let taints = graph
                .edges
                .iter()
                .filter(|x| x.get_from() == start && &x.to().function == function)
                .map(|x| x.to())
                .map(|x| Taint {
                    function: x.function.clone(),
                    pc: x.pc,
                    variable: x.belongs_to_var.clone(),
                })
                .collect();

            Ok(taints)
        } else {
            Ok(vec![])
        }
    }
}

pub trait GraphReachability {
    fn all_sinks(&mut self, graph: &mut Graph, req: &Request) -> Vec<Taint>;
}
