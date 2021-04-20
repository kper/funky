use crate::icfg::graph::{Fact, Graph};
use crate::icfg::state::State;
use crate::solver::*;
use std::collections::VecDeque;

use log::debug;

pub struct Bfs;

impl GraphReachability for Bfs {
    fn all_sinks(&mut self, graph: &mut Graph, state: &State, req: &Request) -> Vec<Taint> {
        let mut queue: VecDeque<&Fact> = VecDeque::new();
        let mut seen = Vec::new();

        if let Some(start) = state
            .get_facts_at(&req.function, req.pc)
            .unwrap()
            .find(|x| &x.belongs_to_var == req.variable.as_ref().unwrap())
        {
            debug!("Adding start {:#?}", start);
            queue.push_back(start);
        } else {
            return Vec::new();
        }

        while let Some(node) = queue.pop_front() {
            debug!("Popping node {:?}", node);
            seen.push(node);
            for child in graph
                .edges
                .iter()
                .filter(|x| x.get_from() == node)
                .map(|x| x.to())
            {
                //debug!("Adding child {:#?}", graph.query_by_fact_id(child));

                if !seen.contains(&child) {
                    debug!("queue child {:?}", child);
                    queue.push_back(child);
                }
            }
        }

        seen.into_iter()
            .map(|x| Taint {
                variable: x.belongs_to_var.clone(),
                pc: x.next_pc,
                function: x.function.clone(),
            })
            .collect()
    }
}
