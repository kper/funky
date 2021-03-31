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

type FunctionName = String;
type BlockNum = String;

#[derive(Debug)]
pub struct ConvertSummary {
    block_counter: Counter,
    block_resolver: HashMap<(FunctionName, BlockNum), usize>,
}

impl ConvertSummary {
    pub fn new() -> Self {
        Self {
            block_counter: Counter::default(),
            block_resolver: HashMap::new(),
        }
    }

    pub fn visit(&mut self, prog: &Program, req: &Request) -> Result<Graph> {
        let mut graph = Graph::new();

        self.tabulate(&mut graph, prog, req)?;

        Ok(graph)
    }

    fn compute_init_flows(
        &self,
        function: &AstFunction,
        graph: &mut Graph,
        pc: usize,
    ) -> Result<Vec<Edge>> {
        debug!("Calling init flow for {} with pc {}", function.name, pc);

        let mut edges = Vec::new();

        // We need the offset, because not every
        // instruction is taintable. For example BLOCK and JUMP
        // have no registers. That's why skip to the next one.
        let mut offset = 0;
        let instructions = &function.instructions;

        loop {
            let pc = pc + offset;
            let instruction = instructions.get(pc).context("Cannot find instr")?;
            debug!("Next instruction is {:?}", instruction);

            let init_fact = graph
                .facts
                .iter()
                .find(|x| x.function == function.name && x.pc == pc && x.var_is_taut)
                .context("Cannot find taut")?;

            match instruction {
                Instruction::Const(reg, _) => {
                    let before2 = graph
                        .get_facts_at(&function.name, pc)?
                        .into_iter()
                        .filter(|x| x.var_is_taut)
                        .cloned();

                    let after2 = graph
                        .get_facts_at(&function.name, pc + 1)?
                        .into_iter()
                        .filter(|x| &x.belongs_to_var == reg)
                        .cloned();

                    for (b, a) in before2.zip(after2) {
                        edges.push(Edge::Normal {
                            from: b,
                            to: a.clone(),
                            curved: false,
                        });
                        edges.push(Edge::Path {
                            from: init_fact.clone(),
                            to: a,
                        });
                    }
                }
                Instruction::Assign(dest, _src) => {
                    let before2 = graph
                        .get_facts_at(&function.name, pc)?
                        .into_iter()
                        .filter(|x| x.var_is_taut)
                        .cloned();

                    let after2 = graph
                        .get_facts_at(&function.name, pc + 1)?
                        .into_iter()
                        .filter(|x| &x.belongs_to_var == dest)
                        .cloned();

                    for (b, a) in before2.zip(after2) {
                        edges.push(Edge::Normal {
                            from: b,
                            to: a.clone(),
                            curved: false,
                        });
                        edges.push(Edge::Path {
                            from: init_fact.clone(),
                            to: a,
                        });
                    }
                }
                Instruction::Unop(dest, _src) => {
                    let before2 = graph
                        .get_facts_at(&function.name, pc)?
                        .into_iter()
                        .filter(|x| x.var_is_taut)
                        .cloned();

                    let after2 = graph
                        .get_facts_at(&function.name, pc + 1)?
                        .into_iter()
                        .filter(|x| &x.belongs_to_var == dest)
                        .cloned();

                    for (b, a) in before2.zip(after2) {
                        edges.push(Edge::Normal {
                            from: b,
                            to: a.clone(),
                            curved: false,
                        });
                        edges.push(Edge::Path {
                            from: init_fact.clone(),
                            to: a,
                        });
                    }
                }
                Instruction::BinOp(dest, _src, _src2) => {
                    let before2 = graph
                        .get_facts_at(&function.name, pc)?
                        .into_iter()
                        .filter(|x| x.var_is_taut)
                        .cloned();

                    let after2 = graph
                        .get_facts_at(&function.name, pc + 1)?
                        .into_iter()
                        .filter(|x| &x.belongs_to_var == dest)
                        .cloned();

                    for (b, a) in before2.zip(after2) {
                        edges.push(Edge::Normal {
                            from: b,
                            to: a.clone(),
                            curved: false,
                        });
                        edges.push(Edge::Path {
                            from: init_fact.clone(),
                            to: a,
                        });
                    }
                }
                Instruction::Conditional(reg, _) => {
                    let before2 = graph
                        .get_facts_at(&function.name, pc)?
                        .into_iter()
                        .filter(|x| x.var_is_taut)
                        .cloned();

                    let after2 = graph
                        .get_facts_at(&function.name, pc + 1)?
                        .into_iter()
                        .filter(|x| &x.belongs_to_var == reg)
                        .cloned();

                    for (b, a) in before2.zip(after2) {
                        edges.push(Edge::Normal {
                            from: b,
                            to: a.clone(),
                            curved: false,
                        });
                        edges.push(Edge::Path {
                            from: init_fact.clone(),
                            to: a,
                        });
                    }
                }
                Instruction::Block(_) | Instruction::Jump(_) => {
                    let before2 = graph
                        .get_facts_at(&function.name, pc)?
                        .into_iter()
                        .filter(|x| x.var_is_taut)
                        .cloned();

                    let after2 = graph
                        .get_facts_at(&function.name, pc + 1)?
                        .into_iter()
                        .filter(|x| x.var_is_taut)
                        .cloned();

                    for (b, a) in before2.zip(after2) {
                        edges.push(Edge::Normal {
                            from: b,
                            to: a.clone(),
                            curved: false,
                        });
                        edges.push(Edge::Path {
                            from: init_fact.clone(),
                            to: a,
                        });
                    }

                    offset += 1;

                    continue;
                }
                Instruction::Phi(dest, _src1, _src2) => {
                    let before2 = graph
                        .get_facts_at(&function.name, pc)?
                        .into_iter()
                        .filter(|x| x.var_is_taut)
                        .cloned();

                    let after2 = graph
                        .get_facts_at(&function.name, pc + 1)?
                        .into_iter()
                        .filter(|x| &x.belongs_to_var == dest)
                        .cloned();

                    for (b, a) in before2.zip(after2) {
                        edges.push(Edge::Normal {
                            from: b,
                            to: a.clone(),
                            curved: false,
                        });
                        edges.push(Edge::Path {
                            from: init_fact.clone(),
                            to: a,
                        });
                    }
                }
                _ => {}
            }

            break;
        }

        Ok(edges)
    }

    /// computes all intraprocedural edges
    fn flow(
        &self,
        function: &AstFunction,
        graph: &mut Graph,
        pc: usize,
        variable: &String,
    ) -> Result<Vec<Edge>> {
        debug!(
            "Calling flow for {} with var {} with pc {}",
            function.name, variable, pc
        );

        let mut edges = Vec::new();

        let instructions = &function.instructions;

        let instruction = instructions.get(pc).context("Cannot find instr")?;
        debug!("Next instruction is {:?}", instruction);

        match instruction {
            Instruction::Const(reg, _) if reg != variable => {
                // Identity
                let before2 = graph
                    .get_facts_at(&function.name, pc)?
                    .into_iter()
                    .filter(|x| &x.belongs_to_var == variable)
                    .cloned();

                let after2 = graph
                    .get_facts_at(&function.name, pc + 1)?
                    .into_iter()
                    .filter(|x| &x.belongs_to_var == variable)
                    .cloned();

                for (b, a) in before2.zip(after2) {
                    edges.push(Edge::Normal {
                        from: b,
                        to: a,
                        curved: false,
                    });
                }
            }
            Instruction::Const(_reg, _) => {
                //kill
            }
            Instruction::Assign(dest, src) if src == variable => {
                let before2 = graph
                    .get_facts_at(&function.name, pc)?
                    .into_iter()
                    .filter(|x| &x.belongs_to_var == src)
                    .cloned();

                let after2 = graph
                    .get_facts_at(&function.name, pc + 1)?
                    .into_iter()
                    .filter(|x| &x.belongs_to_var == dest || &x.belongs_to_var == src)
                    //.filter(|x| &x.belongs_to_var != dest)
                    .cloned();

                for (b, a) in (before2.clone().chain(before2)).zip(after2) {
                    edges.push(Edge::Normal {
                        from: b,
                        to: a,
                        curved: false,
                    });
                }
            }
            Instruction::Assign(dest, src) if dest != variable && src != variable => {
                // Identity
                let before2 = graph
                    .get_facts_at(&function.name, pc)?
                    .into_iter()
                    .filter(|x| &x.belongs_to_var == variable)
                    .cloned();

                let after2 = graph
                    .get_facts_at(&function.name, pc + 1)?
                    .into_iter()
                    .filter(|x| &x.belongs_to_var == variable)
                    .cloned();

                for (b, a) in before2.zip(after2) {
                    edges.push(Edge::Normal {
                        from: b,
                        to: a,
                        curved: false,
                    });
                }
            }
            Instruction::Assign(_dest, _src) => {
                // kill
            }
            Instruction::Unop(dest, src) if src == variable => {
                let before2 = graph
                    .get_facts_at(&function.name, pc)?
                    .into_iter()
                    .filter(|x| &x.belongs_to_var == src)
                    .cloned();

                let after2 = graph
                    .get_facts_at(&function.name, pc + 1)?
                    .into_iter()
                    .filter(|x| &x.belongs_to_var == dest || &x.belongs_to_var == src)
                    //.filter(|x| &x.belongs_to_var != dest)
                    .cloned();

                for (b, a) in (before2.clone().chain(before2)).zip(after2) {
                    edges.push(Edge::Normal {
                        from: b,
                        to: a,
                        curved: false,
                    });
                }
            }
            Instruction::Unop(dest, src) if dest != variable && src != variable => {
                // Identity
                let before2 = graph
                    .get_facts_at(&function.name, pc)?
                    .into_iter()
                    .filter(|x| &x.belongs_to_var == variable)
                    .cloned();

                let after2 = graph
                    .get_facts_at(&function.name, pc + 1)?
                    .into_iter()
                    .filter(|x| &x.belongs_to_var == variable)
                    .cloned();

                for (b, a) in before2.zip(after2) {
                    edges.push(Edge::Normal {
                        from: b,
                        to: a,
                        curved: false,
                    });
                }
            }
            Instruction::Unop(_dest, _src) => {
                // kill
            }
            Instruction::BinOp(dest, src1, _src2) if src1 == variable => {
                let before2 = graph
                    .get_facts_at(&function.name, pc)?
                    .into_iter()
                    .filter(|x| &x.belongs_to_var == src1)
                    .cloned();

                let after2 = graph
                    .get_facts_at(&function.name, pc + 1)?
                    .into_iter()
                    .filter(|x| &x.belongs_to_var == dest || &x.belongs_to_var == src1)
                    //.filter(|x| &x.belongs_to_var != dest)
                    .cloned();

                for (b, a) in (before2.clone().chain(before2)).zip(after2) {
                    edges.push(Edge::Normal {
                        from: b,
                        to: a,
                        curved: false,
                    });
                }
            }
            Instruction::BinOp(dest, _src1, src2) if src2 == variable => {
                let before2 = graph
                    .get_facts_at(&function.name, pc)?
                    .into_iter()
                    .filter(|x| &x.belongs_to_var == src2)
                    .cloned();

                let after2 = graph
                    .get_facts_at(&function.name, pc + 1)?
                    .into_iter()
                    .filter(|x| &x.belongs_to_var == dest || &x.belongs_to_var == src2)
                    //.filter(|x| &x.belongs_to_var != dest)
                    .cloned();

                for (b, a) in (before2.clone().chain(before2)).zip(after2) {
                    edges.push(Edge::Normal {
                        from: b,
                        to: a,
                        curved: false,
                    });
                }
            }
            Instruction::BinOp(dest, src1, src2)
                if dest != variable && src1 != variable && src2 != variable =>
            {
                // Identity
                let before2 = graph
                    .get_facts_at(&function.name, pc)?
                    .into_iter()
                    .filter(|x| &x.belongs_to_var == variable)
                    .cloned();

                let after2 = graph
                    .get_facts_at(&function.name, pc + 1)?
                    .into_iter()
                    .filter(|x| &x.belongs_to_var == variable)
                    .cloned();

                for (b, a) in before2.zip(after2) {
                    edges.push(Edge::Normal {
                        from: b,
                        to: a,
                        curved: false,
                    });
                }
            }
            Instruction::BinOp(_dest, _src, _src2) => {
                // kill
            }
            Instruction::Kill(reg) if variable == reg => {
                // kill
            }
            Instruction::Jump(block) => {
                let jump_to_pc = self
                    .block_resolver
                    .get(&(function.name.clone(), block.clone()))
                    .context("Cannot find block to jump to")?;

                let before2 = graph
                    .get_facts_at(&function.name, pc)?
                    .into_iter()
                    .filter(|x| &x.belongs_to_var == variable)
                    .cloned();

                let after2 = graph
                    .get_facts_at(&function.name, *jump_to_pc)?
                    .into_iter()
                    .filter(|x| &x.belongs_to_var == variable)
                    .cloned();

                for (b, a) in before2.zip(after2) {
                    edges.push(Edge::Normal {
                        from: b,
                        to: a,
                        curved: true,
                    });
                }
            }
            Instruction::Conditional(_reg, jumps) if jumps.len() == 1 => {
                // edge case for an `if`
                // which continues if the condition is not successful
                for block in jumps.iter() {
                    let jump_to_pc = self
                        .block_resolver
                        .get(&(function.name.clone(), block.clone()))
                        .context("Cannot find block to jump to")?;

                    let before2 = graph
                        .get_facts_at(&function.name, pc)?
                        .into_iter()
                        .filter(|x| &x.belongs_to_var == variable)
                        .cloned();

                    let after2 = graph
                        .get_facts_at(&function.name, *jump_to_pc)?
                        .into_iter()
                        .filter(|x| &x.belongs_to_var == variable)
                        .cloned();

                    for (b, a) in before2.zip(after2) {
                        edges.push(Edge::Normal {
                            from: b,
                            to: a,
                            curved: true,
                        });
                    }
                }

                let before2 = graph
                    .get_facts_at(&function.name, pc)?
                    .into_iter()
                    .filter(|x| &x.belongs_to_var == variable)
                    .cloned();

                let after2 = graph
                    .get_facts_at(&function.name, pc + 1)?
                    .into_iter()
                    .filter(|x| &x.belongs_to_var == variable)
                    .cloned();

                for (b, a) in before2.zip(after2) {
                    edges.push(Edge::Normal {
                        from: b,
                        to: a,
                        curved: true,
                    });
                }
            }
            Instruction::Conditional(_reg, jumps) => {
                for block in jumps.iter() {
                    let jump_to_pc = self
                        .block_resolver
                        .get(&(function.name.clone(), block.clone()))
                        .context("Cannot find block to jump to")?;

                    let before2 = graph
                        .get_facts_at(&function.name, pc)?
                        .into_iter()
                        .filter(|x| &x.belongs_to_var == variable)
                        .cloned();

                    let after2 = graph
                        .get_facts_at(&function.name, *jump_to_pc)?
                        .into_iter()
                        .filter(|x| &x.belongs_to_var == variable)
                        .cloned();

                    for (b, a) in before2.zip(after2) {
                        edges.push(Edge::Normal {
                            from: b,
                            to: a,
                            curved: true,
                        });
                    }
                }
            }
            Instruction::Phi(dest, src1, _src2) if src1 == variable => {
                let before2 = graph
                    .get_facts_at(&function.name, pc)?
                    .into_iter()
                    .filter(|x| &x.belongs_to_var == src1)
                    .cloned();

                let after2 = graph
                    .get_facts_at(&function.name, pc + 1)?
                    .into_iter()
                    .filter(|x| &x.belongs_to_var == dest || &x.belongs_to_var == src1)
                    //.filter(|x| &x.belongs_to_var != dest)
                    .cloned();

                for (b, a) in (before2.clone().chain(before2)).zip(after2) {
                    edges.push(Edge::Normal {
                        from: b,
                        to: a,
                        curved: false,
                    });
                }
            }
            Instruction::Phi(dest, _src1, src2) if src2 == variable => {
                let before2 = graph
                    .get_facts_at(&function.name, pc)?
                    .into_iter()
                    .filter(|x| &x.belongs_to_var == src2)
                    .cloned();

                let after2 = graph
                    .get_facts_at(&function.name, pc + 1)?
                    .into_iter()
                    .filter(|x| &x.belongs_to_var == dest || &x.belongs_to_var == src2)
                    //.filter(|x| &x.belongs_to_var != dest)
                    .cloned();

                for (b, a) in (before2.clone().chain(before2)).zip(after2) {
                    edges.push(Edge::Normal {
                        from: b,
                        to: a,
                        curved: false,
                    });
                }
            }
            Instruction::Phi(dest, src1, src2)
                if dest != variable && src1 != variable && src2 != variable =>
            {
                // Identity
                let before2 = graph
                    .get_facts_at(&function.name, pc)?
                    .into_iter()
                    .filter(|x| &x.belongs_to_var == variable)
                    .cloned();

                let after2 = graph
                    .get_facts_at(&function.name, pc + 1)?
                    .into_iter()
                    .filter(|x| &x.belongs_to_var == variable)
                    .cloned();

                for (b, a) in before2.zip(after2) {
                    edges.push(Edge::Normal {
                        from: b,
                        to: a,
                        curved: false,
                    });
                }
            }
            Instruction::Phi(_dest, _src, _src2) => {
                // kill
            }
            Instruction::Table(jumps) => {
                for block in jumps.iter() {
                    let jump_to_pc = self
                        .block_resolver
                        .get(&(function.name.clone(), block.clone()))
                        .context("Cannot find block to jump to")?;

                    let before2 = graph
                        .get_facts_at(&function.name, pc)?
                        .into_iter()
                        .filter(|x| &x.belongs_to_var == variable)
                        .cloned();

                    let after2 = graph
                        .get_facts_at(&function.name, *jump_to_pc)?
                        .into_iter()
                        .filter(|x| &x.belongs_to_var == variable)
                        .cloned();

                    for (b, a) in before2.zip(after2) {
                        edges.push(Edge::Normal {
                            from: b,
                            to: a,
                            curved: true,
                        });
                    }
                }
            }
            Instruction::Return(dest) if dest.contains(variable) => {
                // Identity
                let before2 = graph
                    .get_facts_at(&function.name, pc)?
                    .into_iter()
                    .filter(|x| &x.belongs_to_var == variable)
                    .cloned();

                let after2 = graph
                    .get_facts_at(&function.name, pc + 1)?
                    .into_iter()
                    .filter(|x| &x.belongs_to_var == variable)
                    .cloned();

                for (b, a) in before2.zip(after2) {
                    edges.push(Edge::Normal {
                        from: b,
                        to: a,
                        curved: false,
                    });
                }
            }
            _ => {
                // Identity
                let before2 = graph
                    .get_facts_at(&function.name, pc)?
                    .into_iter()
                    .filter(|x| &x.belongs_to_var == variable)
                    .cloned();

                let after2 = graph
                    .get_facts_at(&function.name, pc + 1)?
                    .into_iter()
                    .filter(|x| &x.belongs_to_var == variable)
                    .cloned();

                for (b, a) in before2.zip(after2) {
                    edges.push(Edge::Normal {
                        from: b,
                        to: a,
                        curved: false,
                    });
                }
            }
        }

        Ok(edges)
    }

    /// Computes call-to-start edges
    fn pass_args(
        &mut self,
        program: &Program,
        caller_function: &AstFunction,
        callee_function: &String,
        params: &Vec<String>,
        graph: &mut Graph,
        current_pc: usize,
        caller_var: &String,
    ) -> Result<Vec<Edge>> {
        // Why not dests?
        if !params.contains(&caller_var) && caller_var != &"taut".to_string() {
            debug!(
                "Caller's variable is not a parameter {} in {:?} for {}",
                caller_var, params, callee_function
            );
            return Ok(vec![]);
        }

        // After here, checked that the caller_var is relevant

        let callee_function = program
            .functions
            .iter()
            .find(|x| &x.name == callee_function)
            .context("Cannot find function")?;

        // Init facts of the called function
        // Start from the beginning.
        let start_pc = 0;
        let init_facts = graph.init_function(callee_function, start_pc)?;
        self.resolve_block_ids(&callee_function, start_pc)?;

        // Filter by variable
        let callee_fact = init_facts
            .iter()
            .find(|x| &x.belongs_to_var == caller_var) 
            .context("Cannot find callee's fact")?;

        // Last caller facts
        let caller_facts = graph.get_facts_at(&caller_function.name, current_pc)?;

        // Filter by variable
        let caller_fact = caller_facts
            .iter()
            .find(|x| &x.belongs_to_var == caller_var)
            .context("Cannot find caller's fact")?;

        // The corresponding edges have to match now, but filter `dest`.
        // taut -> taut
        // %0   -> %0
        // %1   -> %1

        // Create an edge.
        let edges = vec![Edge::CallToReturn {
            from: caller_fact.clone().clone(),
            to: callee_fact.clone(),
        }];

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
        debug!("Trying to compute return_val");
        debug!("Caller: {} ({})", caller_function, caller_pc);
        debug!("Callee: {} ({})", callee_function, callee_pc);

        let dest = match caller_instructions.get(caller_pc).as_ref() {
            Some(Instruction::Call(_, _params, dest)) => {
                let mut dd = Vec::with_capacity(dest.len());
                dd.push("taut".to_string());
                dd.extend(dest.clone());
                dd
            }
            Some(x) => bail!("Wrong instruction passed to return val. Found {:?}", x),
            None => bail!("Cannot find instruction while trying to compute exit-to-return edges"),
        };

        let caller_facts = graph.get_facts_at(caller_function, caller_pc + 1)?;

        debug!("Caller facts {:#?}", caller_facts);

        let mut caller_facts = caller_facts.into_iter().collect::<Vec<_>>();

        // We are looking for an edge from the beginning to callee_pc
        // But, we don't know if `pc` starts at 0
        let first_statement_pc_callee = graph
            .edges
            .iter()
            .filter(|x| {
                &x.get_from().function == callee_function && &x.to().function == callee_function
            })
            .map(|x| x.get_from().pc)
            .min()
            .context("Cannot find first statement's pc of callee")
            .unwrap_or(0);

        debug!(
            "graph {:#?}",
            graph
                .edges
                .iter()
                .filter(|x| &x.get_from().function == callee_function
                    && &x.to().function == callee_function)
                .collect::<Vec<_>>()
        );

        // Cannot query all facts, because some vars might not exist anymore
        // We want to check the ones, which are still alive.
        let mut callee_facts = graph
            .edges
            .iter()
            .filter(|x| {
                &x.get_from().function == callee_function
                    && &x.to().function == callee_function
                    && x.get_from().pc == first_statement_pc_callee
                    && x.to().pc == callee_pc
            })
            .map(|x| x.to())
            .collect::<Vec<_>>(); //(callee_function, callee_pc)?;

        caller_facts.sort_by(|a, b| a.track.cmp(&b.track));
        callee_facts.sort_by(|a, b| a.track.cmp(&b.track));

        debug!("caller_facts {:#?}", caller_facts);
        debug!("callee_facts {:#?}", callee_facts);

        // Generate edges when for all dest + taut

        let mut edges = Vec::new();

        debug!("=> dest {:?}", dest);

        for (from, to) in callee_facts.iter().zip(
            caller_facts
                .iter()
                .filter(|x| dest.contains(&x.belongs_to_var)),
        ) {
            edges.push(Edge::Return {
                to: to.clone().clone().clone(),
                from: from.clone().clone(),
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
        caller: &String,
    ) -> Result<Vec<Edge>> {
        debug!(
            "Generating call-to-return edges for {} ({})",
            callee, caller
        );

        let before: Vec<_> = graph
            .get_facts_at(&caller_function.name, pc)?
            .into_iter()
            .filter(|x| &x.belongs_to_var == caller)
            .map(|x| x.clone())
            .collect();
        debug!("Facts before statement {}", before.len());

        let after = graph.get_facts_at(&caller_function.name, pc + 1)?;

        debug!("Facts after statement {}", after.len());

        let after: Vec<_> = after
            .into_iter()
            .filter(|x| &x.belongs_to_var == caller)
            .filter(|x| !dests.contains(&x.belongs_to_var))
            .collect();

        debug!("Facts after statement without dests {}", after.len());

        debug!("before {:#?}", before);
        debug!("after {:#?}", after);

        let mut edges = Vec::with_capacity(after.len());
        for fact in before {
            let b = after
                .iter()
                .find(|x| x.belongs_to_var == fact.belongs_to_var);

            if let Some(b) = b {
                edges.push(Edge::CallToReturn {
                    from: fact.clone(),
                    to: b.clone().clone(),
                });
            } else {
                debug!(
                    "Skipping CallToReturn edge for {} because no match",
                    fact.belongs_to_var
                );
            }
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

        let facts = graph.init_function(&function, req.pc)?;
        let init = facts.get(0).unwrap().clone();

        let mut path_edge = Vec::new();
        let mut worklist = VecDeque::new();
        let mut summary_edge = Vec::new();

        self.propagate(graph, &mut path_edge, &mut worklist, Edge::Path {
            from: init.clone(),
            to: init.clone()
        })?;

        // Compute init flows

        let init_normal_flows = self.compute_init_flows(function, graph, req.pc)?;

        for edge in init_normal_flows.into_iter() {
            self.propagate(graph, &mut path_edge, &mut worklist, edge)?;
        }

        self.forward(
            &prog,
            &function,
            &mut path_edge,
            &mut worklist,
            &mut summary_edge,
            &mut graph,
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
        let f = path_edge.iter().rev().find(|x| *x == &e);

        if f.is_none() {
            debug!("Propagate {:#?}", e);
            graph.edges.push(e.clone());
            path_edge.push(e.clone());
            worklist.push_back(e);
        }

        Ok(())
    }

    /// Iterates over all instructions and remembers the pc of a
    /// BLOCK declaration. Then saves it into `block_resolver`.
    /// Those values will be used for JUMP instructions.
    fn resolve_block_ids(&mut self, function: &AstFunction, start_pc: usize) -> Result<()> {
        for (pc, instruction) in function
            .instructions
            .iter()
            .enumerate()
            .skip(start_pc)
            .filter(|x| matches!(x.1, Instruction::Block(_)))
        {
            match instruction {
                Instruction::Block(block) => {
                    self.block_resolver
                        .insert((function.name.clone(), block.clone()), pc);
                }
                _ => {
                    bail!("This code should be unreachable.");
                }
            }
        }

        Ok(())
    }

    fn forward(
        &mut self,
        program: &Program,
        function: &AstFunction,
        path_edge: &mut Vec<Edge>,
        worklist: &mut VecDeque<Edge>,
        summary_edge: &mut Vec<Edge>,
        graph: &mut Graph,
        start_pc: usize,
    ) -> Result<()> {
        let mut end_summary: HashMap<(String, usize, String), Vec<Fact>> = HashMap::new();
        let mut incoming: HashMap<(String, usize, String), Vec<Fact>> = HashMap::new();

        let mut normal_flows_debug = Vec::new();

        self.resolve_block_ids(&function, start_pc)?;

        while let Some(edge) = worklist.pop_front() {
            debug!("Popping edge from worklist {:#?}", edge);

            let d1 = edge.get_from();
            let d2 = edge.to();

            let pc = edge.to().pc;
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
                    Instruction::Call(callee, params, dest) => {
                        // Here becomes the `d2` the caller.
                        // This is so on purpose, because this will become
                        // the parameter and `d1` doesn't matter here
                        let caller_var = &d2.belongs_to_var;
                        let call_edges = self.pass_args(
                            program, function, callee, params, graph, d2.pc, caller_var,
                        )?;

                        for d3 in call_edges.into_iter() {
                            debug!("d3 {:#?}", d3);

                            self.propagate(
                                graph,
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
                                if !incoming.contains(&d2) {
                                    incoming.push(d2.clone());
                                }
                            } else {
                                incoming.insert(
                                    (
                                        d3.to().function.clone(),
                                        d3.to().pc,
                                        d3.to().belongs_to_var.clone(),
                                    ),
                                    vec![d2.clone()],
                                );
                            }

                            debug!("Incoming in call {:#?}", incoming);

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
                                            to: d5.get_from().clone(),
                                        });
                                    }
                                }
                            }

                            debug!("End summary {:#?}", end_summary);
                        }

                        let call_flow = self.call_flow(
                            program,
                            function,
                            callee,
                            params,
                            dest,
                            graph,
                            pc,
                            &d2.belongs_to_var,
                        )?;

                        let return_sites = summary_edge.iter().filter(|x| {
                            x.get_from().belongs_to_var == d2.belongs_to_var
                                && x.get_from().pc == d2.pc
                                && x.to().pc == d2.pc + 1
                        });

                        for d3 in call_flow.iter().chain(return_sites) {
                            let taut = graph.get_taut(&d3.get_from().function).unwrap().clone();
                            self.propagate(
                                graph,
                                path_edge,
                                worklist,
                                Edge::Path {
                                    from: taut,
                                    to: d3.to().clone(),
                                },
                            )?; // adding edges to return site of caller from d1
                        }
                    }
                    Instruction::Return(_dest) => {
                        let new_function = program
                            .functions
                            .iter()
                            .find(|x| x.name == d2.function)
                            .unwrap();
                        for f in self
                            .flow(&new_function, graph, d2.pc, &d2.belongs_to_var)?
                            .iter()
                        {
                            debug!("Normal flow {:?}", f);
                            let to = f.to();

                            //graph.edges.push(f.clone());
                            normal_flows_debug.push(f.clone());

                            self.propagate(
                                graph,
                                path_edge,
                                worklist,
                                Edge::Path {
                                    from: d1.clone(),
                                    to: to.clone(),
                                },
                            )?;
                        }

                        assert_eq!(d1.function, d2.function);

                        let first_statement_pc_callee = graph
                            .edges
                            .iter()
                            .filter(|x| {
                                x.get_from().function == d1.function
                                    && x.to().function == d1.function
                            })
                            .map(|x| x.get_from().pc)
                            .min()
                            .context("Cannot find first statement's pc of callee")
                            .unwrap_or(0);

                        if let Some(end_summary) = end_summary.get_mut(&(
                            d1.function.clone(),
                            first_statement_pc_callee,
                            d1.belongs_to_var.clone(),
                        )) {
                            let facts = graph
                                .get_facts_at(&d2.function.clone(), d2.pc)?
                                .into_iter()
                                .filter(|x| x.belongs_to_var == d2.belongs_to_var)
                                .map(|x| x.clone());
                            end_summary.extend(facts);
                        } else {
                            let facts = graph
                                .get_facts_at(&d2.function.clone(), d2.pc)?
                                .into_iter()
                                .filter(|x| x.belongs_to_var == d2.belongs_to_var)
                                .map(|x| x.clone())
                                .collect();
                            end_summary.insert(
                                (d1.function.clone(), d1.pc, d1.belongs_to_var.clone()),
                                facts,
                            );
                        }

                        debug!("End Summary {:#?}", end_summary);
                    }
                    _ => {
                        let new_function = program
                            .functions
                            .iter()
                            .find(|x| x.name == d2.function)
                            .unwrap();
                        for f in self
                            .flow(&new_function, graph, d2.pc, &d2.belongs_to_var)?
                            .iter()
                        {
                            debug!("Normal flow {:?}", f);
                            let to = f.to();

                            //graph.edges.push(f.clone());
                            normal_flows_debug.push(f.clone());

                            self.propagate(
                                graph,
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
                self.end_procedure(
                    &program,
                    graph,
                    summary_edge,
                    &mut incoming,
                    &mut end_summary,
                    d1,
                    d2,
                    path_edge,
                    worklist,
                )?;
            }
        }

        //graph.edges.extend_from_slice(&path_edge);
        graph.edges.extend_from_slice(&normal_flows_debug);
        //graph.edges.extend_from_slice(&summary_edge);

        Ok(())
    }

    fn end_procedure(
        &mut self,
        program: &Program,
        graph: &mut Graph,
        summary_edge: &mut Vec<Edge>,
        incoming: &mut HashMap<(String, usize, String), Vec<Fact>>,
        end_summary: &mut HashMap<(String, usize, String), Vec<Fact>>,
        d1: &Fact,
        d2: &Fact,
        path_edge: &mut Vec<Edge>,
        worklist: &mut VecDeque<Edge>,
    ) -> Result<()> {
        // this is E_p
        debug!("=> Reached end of procedure");

        assert_eq!(d1.function, d2.function);

        // Summary
        if let Some(end_summary) =
            end_summary.get_mut(&(d1.function.clone(), d1.pc, d1.belongs_to_var.clone()))
        {
            let facts = graph
                .get_facts_at(&d2.function.clone(), d2.pc)?
                .into_iter()
                .filter(|x| x.belongs_to_var == d2.belongs_to_var)
                .map(|x| x.clone());
            end_summary.extend(facts);
        } else {
            let facts = graph
                .get_facts_at(&d2.function.clone(), d2.pc)?
                .into_iter()
                .filter(|x| x.belongs_to_var == d2.belongs_to_var)
                .map(|x| x.clone())
                .collect();
            end_summary.insert(
                (d1.function.clone(), d1.pc, d1.belongs_to_var.clone()),
                facts,
            );
        }

        debug!("End Summary {:#?}", end_summary);

        // Incoming has as key the beginning of procedure
        // The values are the callers of the procedure.
        if let Some(incoming) =
            incoming.get_mut(&(d1.function.clone(), d1.pc, d1.belongs_to_var.clone()))
        {
            for d4 in incoming {
                debug!("Computing return to fact to {:#?}", d4);

                assert!(d4.function != d2.function);

                let instructions = &program
                    .functions
                    .iter()
                    .find(|x| x.name == d4.function)
                    .context("Cannot find function")?
                    .instructions;

                // Get all return edges
                let ret_vals = self.return_val(
                    &d4.function,
                    &d2.function,
                    d4.pc,
                    d2.pc,
                    &instructions,
                    graph,
                )?;

                debug!("Exit-To-Return edges are {:#?}", ret_vals);

                // Use only `d4`'s var
                let ret_vals = ret_vals.into_iter().collect::<Vec<_>>();

                debug!("Exit-To-Return edges (filtered) are {:#?}", ret_vals);

                let ret_vals = ret_vals.iter().map(|x| x.to()).collect::<Vec<_>>();

                debug!("Exit-To-Return vars (filtered) are {:#?}", ret_vals);

                for d5 in ret_vals.into_iter() {
                    debug!("Handling var {:#?}", d5);

                    assert!(d5.function != d2.function);

                    if summary_edge
                        .iter()
                        .find(|x| x.get_from() != d4 && x.to() != d5)
                        .is_none()
                    {
                        summary_edge.push(Edge::Summary {
                            from: d4.clone(),
                            to: d5.clone().clone(),
                        });

                        // Get all path edges
                        // from `d3` to `d4`
                        let edges: Vec<_> = path_edge
                            .iter()
                            .filter(|x| {
                                x.to() == d4
                                    && &x.get_from().function == &d4.function
                                    && x.get_from().pc == 0
                            })
                            .cloned()
                            .collect();

                        for d3 in edges.into_iter() {
                            self.propagate(
                                graph,
                                path_edge,
                                worklist,
                                Edge::Path {
                                    from: d3.get_from().clone(),
                                    to: d5.clone().clone(),
                                },
                            )?;
                        }
                    }
                }
            }
        }

        Ok(())
    }
}
