/// This module is responsible to parse
/// the webassembly AST to a graph
use crate::icfg::graph::*;
use crate::icfg::state::State;
use crate::ir::ast::Function as AstFunction;
use crate::ir::ast::Instruction;

use crate::solver::Request;
use anyhow::{Context, Result};
use std::collections::VecDeque;

use log::debug;

use crate::ir::ast::Program;

use crate::icfg::naive::convert::Convert as NaiveConvert;
use std::collections::HashMap;

type CallerFunction = String;
type PC = usize;
type Function = String;
type CallResolver = HashMap<Function, Vec<(CallerFunction, PC, Vec<String>)>>;
pub(crate) struct Ctx<'a> {
    pub graph: &'a mut Graph,
    pub new_graph: &'a mut Graph,
    pub state: &'a mut State,
    pub call_resolver: &'a CallResolver,
}

/// Central datastructure for the computation of the IFDS problem.
#[derive(Debug, Default)]
pub struct OriginalConvert;

impl OriginalConvert {
    /// Computes a graph by a given program and `req` ([`Request`]).
    /// The `variable` in `req` doesn't matter. It only matters the `function` and `pc`.
    pub fn visit(&mut self, prog: &Program, req: &Request) -> Result<(Graph, State)> {
        let mut fact_gen = NaiveConvert::default();
        let (mut graph, mut state, call_resolver) = fact_gen
            .visit(prog)
            .context("Naive fact generation failed")?;

        let mut new_graph = Graph::default();

        let mut ctx = Ctx {
            graph: &mut graph,
            state: &mut state,
            new_graph: &mut new_graph,
            call_resolver: &call_resolver,
        };

        self.tabulate(&mut ctx, prog, req)?;

        Ok((new_graph, state))
    }

    fn tabulate<'a>(&mut self, ctx: &mut Ctx<'a>, prog: &Program, req: &Request) -> Result<()> {
        debug!("Convert intermediate repr to graph");

        let function = prog
            .functions
            .iter()
            .find(|x| x.name == req.function)
            .context("Cannot find function")?;

        let facts = ctx
            .state
            .get_facts_at(&function.name, req.pc)?
            .cloned()
            .collect::<Vec<_>>();

        let mut path_edge = Vec::new();
        let mut worklist = VecDeque::new();
        let mut summary_edge = Vec::new();
        let mut normal_flows_debug = Vec::new();

        let init = facts.get(0).unwrap().clone();

        // self loop for taut
        self.propagate(
            &mut ctx.new_graph,
            &mut path_edge,
            &mut worklist,
            Edge::Path {
                from: init.clone(),
                to: init.clone(),
            },
        )?;

        self.forward(
            &prog,
            &function,
            &mut path_edge,
            &mut worklist,
            &mut summary_edge,
            &mut normal_flows_debug,
            ctx,
            req.pc,
        )?;

        Ok(())
    }

    fn propagate(
        &self,
        graph: &mut Graph,
        path_edge: &mut Vec<Edge>,
        worklist: &mut VecDeque<Edge>,
        e: Edge,
    ) -> Result<()> {
        let from = e.get_from();
        let to = e.to();

        let f = path_edge
            .iter()
            .find(|x| x.get_from() == from && x.to() == to);

        if f.is_none() {
            debug!("Propagate {:#?}", e);
            graph.edges.push(e.clone());
            path_edge.push(e.clone());
            worklist.push_back(e);
        }

        Ok(())
    }

    fn forward<'a>(
        &mut self,
        program: &Program,
        _function: &AstFunction,
        path_edge: &mut Vec<Edge>,
        worklist: &mut VecDeque<Edge>,
        summary: &mut Vec<Edge>,
        normal_flows_debug: &mut Vec<Edge>,
        ctx: &mut Ctx<'a>,
        start_pc: usize,
    ) -> Result<()> {
        while let Some(edge) = worklist.pop_front() {
            debug!("Popping edge from worklist {:#?}", edge);

            assert!(
                matches!(edge, Edge::Path { .. }),
                "Edge in the worklist has wrong type"
            );

            let d1 = edge.get_from();
            let d2 = edge.to();

            let pc = edge.to().next_pc;
            debug!("Instruction pointer is {}", pc);

            let instructions = &program
                .functions
                .iter()
                .find(|x| x.name == d2.function)
                .context("Cannot find function")?
                .instructions;
            let n = instructions.get(pc);
            debug!("=> Instruction {:?}", n);

            if let Some(n) = n {
                match n {
                    Instruction::Call(callee, _params, _dest) => {
                        // handle_call

                        let caller_function = program
                            .functions
                            .iter()
                            .find(|x| x.name == d1.function)
                            .unwrap();

                        let callee_function = program
                            .functions
                            .iter()
                            .find(|x| &x.name == callee)
                            .unwrap();

                        let flow_edges = ctx.graph.edges.iter().filter(|x| {
                            x.get_from().function == caller_function.name
                                && x.get_from().belongs_to_var == d2.belongs_to_var
                                && x.get_from().next_pc == d2.next_pc
                                && x.to().function == callee_function.name
                                && x.to().next_pc == 0
                        });

                        for f in flow_edges.into_iter() {
                            let to = f.to();
                            self.propagate(
                                &mut ctx.new_graph,
                                path_edge,
                                worklist,
                                Edge::Path {
                                    from: to.clone(),
                                    to: to.clone(),
                                },
                            )?;
                        }

                        let flow_edges = ctx
                            .graph
                            .edges
                            .iter()
                            .chain(summary.iter())
                            .filter(|x| {
                                x.get_from().function == caller_function.name
                                    && x.get_from().belongs_to_var == d2.belongs_to_var
                                    && x.get_from().next_pc == d2.next_pc
                                    && x.to().function == caller_function.name
                                    //&& x.to().belongs_to_var == x.get_from().belongs_to_var
                                    && x.to().next_pc == x.get_from().next_pc + 1
                            })
                            .map(|x| x.to());

                        for to in flow_edges.into_iter() {
                            assert_eq!(d1.function, to.function);

                            self.propagate(
                                &mut ctx.new_graph,
                                path_edge,
                                worklist,
                                Edge::Path {
                                    from: d1.clone(),
                                    to: to.clone(),
                                },
                            )?;
                        }
                    }
                    _ => {
                        let new_function = program
                            .functions
                            .iter()
                            .find(|x| x.name == d2.function)
                            .unwrap();

                        let flow_edges =
                            ctx.graph
                                .edges
                                .iter()
                                .filter(|x| x.is_normal())
                                .filter(|x| {
                                    x.get_from().function == new_function.name
                                        && x.get_from().belongs_to_var == d2.belongs_to_var
                                        && x.get_from().next_pc == d2.next_pc
                                });

                        for f in flow_edges.into_iter() {
                            debug!("Normal flow {:#?}", f);
                            let to = f.to();

                            normal_flows_debug.push(f.clone());

                            self.propagate(
                                &mut ctx.new_graph,
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
                // end procedure

                if let Some(callers) = ctx.call_resolver.get(&d1.function) {
                    let callers = callers
                        .iter()
                        .map(|(caller, _pc, _dests)| (caller, &d1.function));

                    for (caller, callee) in callers {
                        assert_eq!(callee, &d1.function);

                        let d4 = ctx
                            .graph
                            .edges
                            .iter()
                            .filter(|x| x.is_call())
                            .filter(|x| {
                                &x.get_from().function == caller
                                    && x.to().function == d1.function
                                    && x.to().belongs_to_var == d1.belongs_to_var
                                    && x.to().next_pc == d1.next_pc
                            })
                            .map(|x| x.get_from());

                        for d4 in d4.into_iter() {
                            let d5 = ctx
                                .graph
                                .edges
                                .iter()
                                .filter(|x| x.is_return())
                                .filter(|x| {
                                    x.get_from().belongs_to_var == d2.belongs_to_var
                                    && x.get_from().function == d2.function
                                    && x.get_from().next_pc == d2.next_pc - 1 //because applied before
                                    && &x.to().function == &d4.function
                                    // && x.to().next_pc == pc + 1
                                })
                                .map(|x| x.to());

                            for d5 in d5 {
                                if summary
                                    .iter()
                                    .find(|x| {
                                        &x.get_from().function == caller
                                            && x.get_from().belongs_to_var == d4.belongs_to_var
                                            && &x.to().function == caller
                                            && x.to().next_pc == x.get_from().next_pc + 1
                                            && x.to().belongs_to_var == d5.belongs_to_var
                                    })
                                    .is_none()
                                {
                                    let ret = d4.clone();
                                    summary.push(Edge::Summary {
                                        from: d4.clone(),
                                        to: ret.clone(),
                                    });

                                    let edges = ctx
                                        .graph
                                        .edges
                                        .iter()
                                        .filter(|x| {
                                            &x.get_from().function == caller
                                                && &x.to().function == caller
                                                && x.get_from().next_pc == start_pc
                                                && x.to().belongs_to_var == d4.belongs_to_var
                                                && x.to().next_pc == d4.next_pc
                                        })
                                        .map(|x| x.get_from());

                                    for d3 in edges {
                                        let mut ret = ret.clone();
                                        ret.belongs_to_var = d5.belongs_to_var.clone();

                                        let init = ctx
                                            .state
                                            .get_facts_at(caller, 0)?
                                            .find(|x| x.belongs_to_var == d3.belongs_to_var)
                                            .context("Cannot find fact")?
                                            .clone();

                                        self.propagate(
                                            &mut ctx.new_graph,
                                            path_edge,
                                            worklist,
                                            Edge::Path {
                                                from: init,
                                                to: ret,
                                            },
                                        )?;
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }

        //graph.edges.extend_from_slice(&path_edge);
        //graph.edges.extend_from_slice(&normal_flows_debug);
        //graph.edges.extend_from_slice(&summary_edge);

        Ok(())
    }
}
