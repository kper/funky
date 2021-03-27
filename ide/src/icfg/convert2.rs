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
        variable: &String,
    ) -> Result<Vec<Edge>> {
        debug!(
            "Calling flow for {} with var {} with pc {}",
            function.name, variable, pc
        );

        let mut edges = Vec::new();

        let instruction = instructions.get(pc).context("Cannot find instr")?;
        debug!("Next instruction is {:?}", instruction);

        //graph.add_statement(function, instruction, pc + 1)?;
        let before: Vec<Fact> = graph
            .get_facts_at(&function.name, pc)?
            .into_iter()
            //.filter(|x| &x.belongs_to_var == variable)
            .cloned()
            .collect();

        debug!("Facts before statement {}", before.len());

        graph.add_statement(function, format!("{:?}", instruction), pc + 1, variable)?;

        let after: Vec<_> = graph
            .get_facts_at(&function.name, pc + 1)?
            .into_iter()
            //.filter(|x| &x.belongs_to_var == variable)
            .collect();

        debug!("Facts after statement {}", after.len());

        for (b, a) in before.into_iter().zip(after) {
            edges.push(Edge::Normal {
                from: b,
                to: a.clone().clone(),
                curved: false,
            });
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
        current_pc: usize,
    ) -> Result<Vec<Edge>> {
        let callee_function = program
            .functions
            .iter()
            .find(|x| &x.name == callee)
            .context("Cannot find function")?;

        let facts = graph.init_function(callee_function)?;
        let caller_facts = graph.get_facts_at(&caller_function.name, current_pc)?;

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
        caller_function: &String,
        callee_function: &String,
        caller_pc: usize,
        callee_pc: usize,
        //dest: &Vec<String>,
        caller_instructions: &Vec<Instruction>,
        graph: &mut Graph,
    ) -> Result<Vec<Edge>> {
        let dest = match caller_instructions.get(caller_pc).as_ref() {
            Some(Instruction::Call(_, _params, dest)) => dest,
            Some(_) => bail!("Wrong instruction passed to return val"),
            None => bail!("Cannot find instruction"),
        };

        let caller_facts = graph.get_facts_at(caller_function, caller_pc)?;
        let callee_facts = graph.get_facts_at(callee_function, callee_pc)?;

        // Generate edges when for all dest + taut

        let mut edges = Vec::new();

        // taut
        edges.push(Edge::CallToReturn {
            from: caller_facts.get(0).unwrap().clone().clone(),
            to: callee_facts.get(0).unwrap().clone().clone(),
        });

        for (i, d) in dest.iter().enumerate() {
            let caller_fact = caller_facts
                .iter()
                .find(|x| &x.belongs_to_var == d)
                .context("Cannot find var")?;
            let callee_fact = callee_facts.get(i + 1).context("Cannot find var")?;

            edges.push(Edge::CallToReturn {
                from: caller_fact.clone().clone(),
                to: callee_fact.clone().clone(),
            });
        }

        Ok(edges)
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
        debug!("Generating call-to-return edges for {}", callee);

        graph.add_statement(
            caller_function,
            format!("{:?}", "call"),
            pc + 1,
            &"taut".to_string(),
        )?;
        for dest in dests.iter() {
            graph.add_statement(caller_function, format!("{:?}", "call"), pc, dest)?;
            graph.add_statement(caller_function, format!("{:?}", "call"), pc + 1, dest)?;
        }

        let before: Vec<_> = graph
            .get_facts_at(&caller_function.name, pc)?
            .into_iter()
            .map(|x| x.clone())
            .collect();
        debug!("Facts before statement {}", before.len());

        let after = graph.get_facts_at(&caller_function.name, pc + 1)?;

        debug!("Facts after statement {}", after.len());

        let after: Vec<_> = after
            .into_iter()
            .filter(|x| !dests.contains(&x.belongs_to_var))
            .collect();

        debug!("Facts after statement without dests {}", after.len());

        debug!("before {:#?}", before);
        debug!("after {:#?}", after);

        let mut edges = Vec::with_capacity(after.len());
        for fact in after {
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

        let function = prog
            .functions
            .iter()
            .find(|x| x.name == req.function)
            .context("Cannot find function")?;

        if graph.is_function_defined(&function.name) {
            debug!("==> Function was already summarised.");
            return Ok(());
        }

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
            &prog,
            &function,
            &mut path_edge,
            &mut worklist,
            &mut summary_edge,
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
        program: &Program,
        function: &AstFunction,
        path_edge: &mut Vec<Edge>,
        worklist: &mut VecDeque<Edge>,
        summary_edge: &mut Vec<Edge>,
        graph: &mut Graph,
    ) -> Result<()> {
        let mut end_summary: HashMap<(String, usize, String), Vec<Fact>> = HashMap::new();
        let mut incoming: HashMap<(String, usize, String), Vec<Fact>> = HashMap::new();

        while let Some(edge) = worklist.pop_front() {
            debug!("Popping edge from worklist {:#?}", edge);
            let pc = edge.to().pc;
            debug!("Instruction pointer is {}", pc);

            let d1 = edge.get_from();
            let d2 = edge.to();

            let instructions = &program
                .functions
                .iter()
                .find(|x| x.name == d1.function)
                .context("Cannot find function")?
                .instructions;
            let n = instructions.get(pc);
            debug!("=> Instruction {:?}", n);

            if let Some(n) = n {
                match n {
                    Instruction::Call(callee, params, dest) => {
                        let call_edges =
                            self.pass_args(program, function, callee, params, dest, graph, d2.pc)?;

                        for d3 in call_edges.into_iter() {
                            self.propagate(
                                path_edge,
                                worklist,
                                Edge::Path {
                                    from: d3.to().clone(),
                                    to: d3.to().clone(),
                                },
                            )?; //self loop

                            debug!(
                                "Propagate {:?}",
                                Edge::Path {
                                    from: d3.to().clone(),
                                    to: d3.to().clone(),
                                }
                            );

                            //Add incoming

                            if let Some(incoming) = incoming.get_mut(&(
                                d3.to().function.clone(),
                                d3.to().pc,
                                d3.to().belongs_to_var.clone(),
                            )) {
                                incoming.push(d2.clone());
                            } else {
                                incoming.insert(
                                    (
                                        d3.to().function.clone(),
                                        d3.to().id,
                                        d3.to().belongs_to_var.clone(),
                                    ),
                                    vec![d2.clone()],
                                );
                            }

                            debug!("Incoming {:#?}", incoming);

                            if let Some(end_summary) = end_summary.get(&(
                                d3.to().function.clone(),
                                d3.to().pc,
                                d3.to().belongs_to_var.clone(),
                            )) {
                                for d4 in end_summary.iter() {
                                    for d5 in self.return_val(
                                        &function.name,
                                        &d4.function,
                                        d2.pc,
                                        d4.pc,
                                        &instructions,
                                        graph,
                                    )? {
                                        summary_edge.push(Edge::Summary {
                                            from: d2.clone(),
                                            to: d5.get_from().clone(), //return_site is d5.get_from?
                                        });
                                    }
                                }
                            }

                            debug!("End summary {:#?}", end_summary);
                        }

                        let call_flow =
                            self.call_flow(program, function, callee, params, dest, graph, pc)?;

                        let new_function = program
                            .functions
                            .iter()
                            .find(|x| x.name == d2.function)
                            .unwrap();

                        //TODO `or` with overwritten value
                        for d3 in call_flow {
                            let taut = graph.get_taut(&d3.get_from().function).unwrap().clone();
                            self.propagate(
                                path_edge,
                                worklist,
                                Edge::Path {
                                    from: taut,
                                    to: d3.to().clone(),
                                },
                            )?; // adding edges to return site of caller from d1
                        }
                    }
                    _ => {
                        let new_function = program
                            .functions
                            .iter()
                            .find(|x| x.name == d2.function)
                            .unwrap();
                        for f in self
                            .flow(
                                &new_function,
                                &new_function.instructions,
                                graph,
                                d2.pc,
                                &d2.belongs_to_var,
                            )?
                            .iter()
                        {
                            debug!("Normal flow {:?}", f);
                            let to = f.to();
                            self.propagate(
                                path_edge,
                                worklist,
                                Edge::Path {
                                    from: d1.clone(),
                                    to: to.clone(),
                                },
                            )?;
                        }
                    }
                }
            } else {
                // this is E_p
                debug!("=> Reached end of procedure");

                // Summary
                if let Some(end_summary) =
                    end_summary.get_mut(&(d1.function.clone(), d1.pc, d1.belongs_to_var.clone()))
                {
                    let facts = graph
                        .get_facts_at(&function.name, pc)?
                        .into_iter()
                        .map(|x| x.clone());
                    end_summary.extend(facts);
                } else {
                    let facts = graph
                        .get_facts_at(&function.name, pc)?
                        .into_iter()
                        .map(|x| x.clone())
                        .collect();
                    end_summary.insert(
                        (d1.function.clone(), d1.pc, d1.belongs_to_var.clone()),
                        facts,
                    );
                }

                debug!("End Summary {:#?}", end_summary);

                // Incoming
                if let Some(incoming) =
                    incoming.get_mut(&(d1.function.clone(), d1.pc, d1.belongs_to_var.clone()))
                {
                    for d4 in incoming {
                        let ret_vals = self.return_val(
                            &d2.function,
                            &d4.function,
                            d2.pc,
                            d4.pc,
                            &instructions,
                            graph,
                        )?;

                        for d5 in ret_vals.into_iter() {
                            if summary_edge
                                .iter()
                                .find(|x| x.get_from() != d4 && x.to() != d5.to())
                                .is_none()
                            {
                                summary_edge.push(Edge::Summary {
                                    from: d4.clone(),
                                    to: d5.to().clone(),
                                });

                                let edges: Vec<_> =
                                    path_edge.iter().filter(|x| x.to() == d4).cloned().collect();
 
                                for d3 in edges.into_iter() {
                                    self.propagate(
                                        path_edge,
                                        worklist,
                                        Edge::Path {
                                            from: d3.get_from().clone(),
                                            to: d5.to().clone(),
                                        },
                                    )?;
                                }
                            }
                        }
                    }
                }
            }
        }

        graph.edges.extend_from_slice(&path_edge);
        graph.edges.extend_from_slice(&summary_edge);

        Ok(())
    }
}
