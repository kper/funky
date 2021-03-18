use crate::icfg::graph2::Graph;
use crate::solver::*;
use std::collections::VecDeque;

use log::debug;

pub struct BFS;

impl GraphReachability for BFS {
    fn all_sinks(&mut self, graph: &mut Graph, req: Request) -> Vec<Taint> {
        let mut queue: VecDeque<usize> = VecDeque::new();
        let mut seen = Vec::new();

        if let Some(start) = graph.query(&req) {
            debug!("Adding start {:#?}", start);
            queue.push_back(start.id);
        } else {
            return Vec::new();
        }

        while let Some(node) = queue.pop_front() {
            debug!("Popping node {}", node);
            seen.push(node);
            for child in graph.get_neighbours(node) {
                debug!("Adding child {:#?}", graph.query_by_fact_id(child));

                if !seen.contains(&child) {
                    debug!("queue child {}", child);
                    queue.push_back(child);
                }
            }
        }

        seen.into_iter()
            .map(|x| graph.query_by_fact_id(x).unwrap())
            .map(|x| Taint {
                from: req.variable.clone(),
                from_function: req.function.clone(),
                from_pc: req.pc,
                to: x.belongs_to_var.clone(),
                to_pc: x.pc,
                to_function: x.function.clone(),
            })
            .collect()
    }
}
