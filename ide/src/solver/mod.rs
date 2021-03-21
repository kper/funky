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
    /// Return all sinks of the `req`
    fn all_sinks(&mut self, graph: &mut Graph, req: &Request) -> Vec<Taint>;
    /// Ask a query the solver
    fn ask(&mut self, graph: &mut Graph, req: &Request, response: &Request) -> bool;

    /// Ask a query the solver, but doesn't care about the program counter
    fn fast_ask(
        &mut self,
        graph: &mut Graph,
        req: &Request,
        response_var: &String,
        response_function: &String,
    ) -> bool;
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

    fn ask(&mut self, graph: &mut Graph, req: &Request, response: &Request) -> bool {
        // TODO optimize
        // There is no need to run the whole graph reachability algorithm
        let sinks = self.graph_reachability.all_sinks(graph, req);

        sinks
            .iter()
            .find(|x| {
                x.to == response.variable
                    && x.to_pc == response.pc
                    && x.to_function == response.function
            })
            .is_some()
    }

    fn fast_ask(
        &mut self,
        graph: &mut Graph,
        req: &Request,
        response_var: &String,
        response_function: &String,
    ) -> bool {
        // TODO optimize
        // There is no need to run the whole graph reachability algorithm
        let sinks = self.graph_reachability.all_sinks(graph, req);

        sinks
            .iter()
            .find(|x| &x.to == response_var && &x.to_function == response_function)
            .is_some()
    }
}

pub trait GraphReachability {
    fn all_sinks(&mut self, graph: &mut Graph, req: &Request) -> Vec<Taint>;
}
