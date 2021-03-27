use crate::icfg::graph2::*;
use crate::ir::ast::Function as AstFunction;
use crate::ir::ast::Instruction;
/// This module is responsible to parse
/// the webassembly AST to a graph
use crate::{counter::Counter, solver::Request};
use anyhow::{bail, Context, Result};
use std::collections::VecDeque;

use log::debug;

use std::collections::HashMap;

use crate::ir::ast::Program;

use crate::icfg::graph2::Edge;

#[derive(Debug)]
pub struct ConvertSummary {
    block_counter: Counter,
}

impl ConvertSummary {
    pub fn new() -> Self {
        Self {
            block_counter: Counter::default(),
        }
    }

    pub fn visit(&mut self, prog: &Program, req: &Request) -> Result<Graph> {
        let mut graph = Graph::new();

        self.tabulate(&mut graph, prog, req)?;

        Ok(graph)
    }

    /// computes all intraprocedural edges
    fn flow(
        &self,
        function: &AstFunction,
        instructions: &Vec<Instruction>,
        graph: &mut Graph,
        pc: usize,
    ) -> Result<Vec<Edge>> {
        let mut edges = Vec::new();
        let mut offset_pc = 0;
        for instruction in instructions.iter() {
            let before: Vec<Fact> = graph
                .get_facts_at(function, pc + offset_pc)?
                .into_iter()
                .cloned()
                .collect();
            offset_pc += 1;
            graph.add_statement(function, instruction)?;
            let after = graph.get_facts_at(function, pc + offset_pc)?;

            for (b, a) in before.into_iter().zip(after) {
                edges.push(Edge::Normal {
                    from: b,
                    to: a.clone().clone(),
                    curved: false,
                });
            }
        }

        Ok(edges)
    }

    /// Computes call-to-start edges
    fn pass_args(
        &self,
        program: &Program,
        caller_function: &AstFunction,
        callee: &String,
        params: &Vec<String>,
        dests: &Vec<String>,
        graph: &mut Graph,
    ) -> Result<Vec<Edge>> {
        let callee_function = program
            .functions
            .iter()
            .find(|x| &x.name == callee)
            .context("Cannot find function")?;

        let facts = graph.init_function(callee_function)?;
        let caller_facts = graph.get_facts_at(caller_function,0)?;

        let mut edges = Vec::new();
        for (caller, callee) in caller_facts.iter().zip(facts) {
            edges.push(Edge::Call {
                from: caller.clone().clone(),
                to: callee,
            })
        }

        Ok(edges)
    }

    /// Computes exit-to-return edges
    fn return_val(
        &self,
        function: &AstFunction,
        instructions: &Vec<Instruction>,
        exit_fact: &Fact,
    ) -> Result<Vec<Edge>> {
        Ok(vec![])
    }

    /// Computes call-to-return
    fn call_flow(
        &self,
        program: &Program,
        caller_function: &AstFunction,
        callee: &String,
        params: &Vec<String>,
        dests: &Vec<String>,
        graph: &mut Graph,
        pc: usize,
    ) -> Result<Vec<Edge>> {
        let before = graph.get_facts_at(caller_function, pc - 1)?;
        let after = graph.get_facts_at(caller_function, pc)?;

        let mut edges = Vec::with_capacity(after.len());
        for fact in after
            .into_iter()
            .filter(|x| !dests.contains(&x.belongs_to_var))
        {
            let b = before
                .iter()
                .find(|x| x.belongs_to_var == fact.belongs_to_var)
                .context("Variable mismatch.")?;
            edges.push(Edge::CallToReturn {
                from: b.clone().clone(),
                to: fact.clone(),
            });
        }

        Ok(edges)
    }

    fn tabulate(&mut self, mut graph: &mut Graph, prog: &Program, req: &Request) -> Result<()> {
        debug!("Convert intermediate repr to graph");
        //let mut path_edges = Vec::new();
        //let mut worklist = Vec::new();

        let function = prog
            .functions
            .iter()
            .find(|x| x.name == req.function)
            .context("Cannot find function")?;

        if graph.is_function_defined(&function.name) {
            debug!("==> Function was already summarised.");
            return Ok(());
        }

        /*
        graph.init_function_def(&function)?;
        let mut vars = vec![Variable {
            function: req.function.clone(),
            is_taut: true,
            is_global: false,
            name: "taut".to_string(),
        }];
        let init_facts = graph.init_facts(&function, &mut vars)?;
        let init = init_facts.get(0).context("Cannot get init fact")?;
        */

        let facts = graph.init_function(&function)?;
        let init = facts.get(0).unwrap().clone();

        let mut path_edge = Vec::new();
        let mut worklist = VecDeque::new();
        let mut summary_edge = Vec::new();

        path_edge.push(Edge::Path {
            from: init.clone(),
            to: init.clone(),
        });
        worklist.push_back(Edge::Path {
            from: init.clone(),
            to: init.clone(),
        });

        self.forwardTabulateSLRPs(
            init,
            &prog,
            &function,
            &mut path_edge,
            &mut worklist,
            &mut summary_edge,
            &function.instructions,
            &mut graph,
        )?;

        Ok(())
    }

    fn propagate(
        &self,
        path_edge: &mut Vec<Edge>,
        worklist: &mut VecDeque<Edge>,
        e: Edge,
    ) -> Result<()> {
        let f = path_edge.iter().rev().find(|x| *x == &e);

        if f.is_none() {
            path_edge.push(e.clone());
            worklist.push_back(e);
        }

        Ok(())
    }

    fn forwardTabulateSLRPs(
        &self,
        init: Fact,
        program: &Program,
        function: &AstFunction,
        path_edge: &mut Vec<Edge>,
        worklist: &mut VecDeque<Edge>,
        summary_edge: &mut Vec<Edge>,
        instructions: &Vec<Instruction>,
        graph: &mut Graph,
    ) -> Result<()> {
        debug!("Function has {} instructions", instructions.len());
        let flow = self.flow(function, instructions, graph, init.pc)?;

        let mut end_summary: HashMap<usize, Vec<Fact>> = HashMap::new();
        let mut incoming: HashMap<usize, Vec<Fact>> = HashMap::new();

        while let Some(edge) = worklist.pop_front() {
            debug!("Popping edge from worklist {:?}", edge);
            let pc = edge.to().pc;
            debug!("Instruction pointer is {}", pc);
            let n = instructions.get(pc);
            debug!("=> Instruction {:?}", n);

            if let Some(n) = n {
                match n {
                    Instruction::Call(callee, params, dest) => {
                        let call_edges =
                            self.pass_args(program, function, callee, params, dest, graph)?;

                        for d3 in call_edges.into_iter() {
                            //graph.add_call_edge(d3.from().clone(), d3.to().clone());
                            self.propagate(
                                path_edge,
                                worklist,
                                Edge::Path {
                                    from: d3.to().clone(),
                                    to: d3.to().clone(),
                                },
                            )?; //self loop

                            //TODO Add incoming

                            for d4 in end_summary.get(&d3.to().id).unwrap() {
                                for d5 in self.return_val(function, instructions, d4)? {}

                                //TODO returns
                                /*
                                summary_edge.push(Edge::Summary {
                                    from: n.from().clone(),
                                    to:
                                });*/
                            }
                        }

                        let call_flow =
                            self.call_flow(program, function, callee, params, dest, graph, pc)?;

                        //TODO `or` with overwritten value
                        for d3 in call_flow {
                            self.propagate(
                                path_edge,
                                worklist,
                                Edge::Path {
                                    from: init.clone(),
                                    to: d3.to().clone(),
                                },
                            )?;
                        }
                    }
                    _ => {
                        for f in flow.iter() {
                            let to = f.to();
                            self.propagate(
                                path_edge,
                                worklist,
                                Edge::Path {
                                    from: path_edge.get(0).unwrap().clone().get_from().clone(),
                                    to: to.clone(),
                                },
                            )?;
                        }
                    }
                }
            } else {
                // this is E_p
                debug!("Instruction does not exist. Therefore breaking");

                // Summary
                if let Some(end_summary) = end_summary.get_mut(&init.pc) {
                    let facts = graph.get_facts_at(function, pc)?.into_iter().map(|x| x.clone());
                    end_summary.extend(facts);
                }
                else {
                    let facts = graph.get_facts_at(function, pc)?.into_iter().map(|x| x.clone()).collect();
                    end_summary.insert(init.pc, facts);
                }

                // Incoming
                if let Some(incoming) = incoming.get_mut(&init.pc) {
                    for d4 in incoming {
                        let ret_vals = self.return_val(function, instructions, d4)?;

                        for d5 in ret_vals.into_iter() {
                            if summary_edge.iter().find(|x| x.get_from() != d4 && x.to() != d5.to()).is_none() {
                                summary_edge.push(Edge::Summary { from: d4.clone(), to: d5.to().clone()});

                                //TODO propagation

                            }
                        }
                    }
                }


                break;
            }
        }

        graph.edges.extend_from_slice(&path_edge);

        Ok(())
    }
}
