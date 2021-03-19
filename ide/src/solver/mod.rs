use crate::icfg::graph2::Graph;

pub mod bfs;

type PC = usize;

pub struct IfdsSolver<T>
where
    T: GraphReachability,
{
    graph_reachability: T,
}

#[derive(Debug)]
pub struct Taint {
    pub from: String,
    pub from_function: String,
    pub from_pc: PC,
    pub to: String,
    pub to_pc: PC,
    pub to_function: String,
}

#[derive(Debug, Clone)]
pub struct Request {
    pub variable: String,
    pub function: String,
    pub pc: PC,
}

pub trait Solver {
    fn all_sinks(&mut self, graph: &mut Graph, req: &Request) -> Vec<Taint>;
}

impl<T> IfdsSolver<T>
where
    T: GraphReachability,
{
    pub fn new(algorithm: T) -> Self {
        Self {
            graph_reachability: algorithm,
        }
    }
}

impl<T> Solver for IfdsSolver<T>
where
    T: GraphReachability,
{
    fn all_sinks(&mut self, graph: &mut Graph, req: &Request) -> Vec<Taint> {
        self.graph_reachability.all_sinks(graph, req)
    }
}

pub trait GraphReachability {
    fn all_sinks(&mut self, graph: &mut Graph, req: &Request) -> Vec<Taint>;
}
