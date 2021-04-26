#![allow(dead_code)]

use crate::icfg::graph::*;
use crate::ir::ast::Function as AstFunction;
use crate::ir::ast::Instruction;
use anyhow::{Context, Result};
use log::debug;
use std::collections::HashMap;

use crate::icfg::tabulation::sparse::Ctx;

type Function = String;
type Var = String;
type StartPC = usize;

#[derive(Debug, Default)]
pub struct DefUseChain {
    inner: HashMap<(Function, Var), (StartPC, Graph)>,
}

impl DefUseChain {
    /// Get the DefUseChain for function and variable
    pub fn get_graph(&self, function: &String, var: &String) -> Option<&Graph> {
        self.inner
            .get(&(function.clone(), var.clone()))
            .map(|(_, x)| x)
    }

    /// Get the facts in the graph.
    pub fn get_facts_at(&self, function: &String, var: &String, pc: usize) -> Result<Vec<&Fact>> {
        let graph = self.get_graph(function, var).context("Cannot find graph")?;
        let facts = graph
            .flatten()
            .into_iter()
            .filter(|x| x.pc == pc)
            .collect::<Vec<_>>();

        Ok(facts)
    }

    /// Cache and get next
    pub fn demand<'a>(
        &mut self,
        ctx: &mut Ctx<'a>,
        function: &AstFunction,
        var: &String,
        pc: usize,
    ) -> Result<Vec<Fact>> {
        let graph = self.cache(ctx, function, var, pc)?;

        let x = graph
            .flatten()
            .into_iter()
            .filter(|x| x.pc > pc)
            .map(|x| x.clone())
            .collect::<Vec<_>>();

        Ok(x)
    }

    pub fn get_next<'a>(
        &mut self,
        ctx: &mut Ctx<'a>,
        function: &AstFunction,
        var: &String,
        old_pc: usize,
    ) -> Result<Vec<Fact>> {
        let graph = self.cache(ctx, function, var, old_pc)?;

        let facts = graph
            .flatten()
            .into_iter()
            .filter(|x| x.pc == old_pc)
            .map(|x| x.clone())
            .collect::<Vec<_>>();

        Ok(facts)
    }

    // nodes which point to (var, pc)
    pub fn points_to<'a>(
        &mut self,
        ctx: &mut Ctx<'a>,
        function: &AstFunction,
        var: &String,
        pc: usize,
    ) -> Result<Vec<Fact>> {
        let graph = self.cache(ctx, function, var, pc)?;

        let all_facts = graph.flatten().collect::<Vec<_>>();

        let facts = all_facts
            .into_iter()
            .filter(|x| x.next_pc <= pc)
            .collect::<Vec<_>>();

        let next_pc = facts.iter().map(|x| x.next_pc).max().unwrap_or(0);

        // Get all next nodes, because there might be multiple
        let x: Vec<_> = facts
            .into_iter()
            .filter(|x| x.next_pc == next_pc)
            .map(|x| x.clone())
            .collect();

        Ok(x)
    }

    pub fn get_start_pc(&self, function: &AstFunction, var: &String) -> Option<usize> {
        self.get_start_pc_by_name(&function.name, var)
    }

    pub fn get_start_pc_by_name(&self, function: &String, var: &String) -> Option<usize> {
        self.inner
            .get(&(function.clone(), var.clone()))
            .map(|(pc, _)| *pc)
    }

    /// Build the defuse chain for the instruction
    /// The precondition is that the function must be already initialized.
    /// Because we need the track of the given variable `var`.
    pub(crate) fn cache<'a>(
        &mut self,
        ctx: &mut Ctx<'a>,
        function: &AstFunction,
        var: &String,
        pc: usize,
    ) -> Result<&Graph> {
        let var = ctx
            .state
            .get_var(&function.name, var)
            .context("Variable is not defined in the state")?
            .clone();

        // already exists

        if self
            .inner
            .contains_key(&(function.name.clone(), var.name.clone()))
        {
            let start_pc = self
                .get_start_pc(function, &var.name)
                .context("Cannot get start_pc")?;

            // If `pc` is lower than `start_pc`, then delete and continue
            if pc >= start_pc {
                let x = self
                    .inner
                    .get(&(function.name.clone(), var.name.clone()))
                    .map(|(_, x)| x)
                    .context("Cannot get graph")?;

                return Ok(x);
            } else {
                self.inner
                    .remove(&(function.name.clone(), var.name.clone()));
            }
        }

        /*
        let instructions = function
            .instructions
            .iter()
            .enumerate()
            .skip(pc)
            .filter(|(_, x)| self.is_used(&var, x))
            .collect::<Vec<_>>();
            */

        let track = ctx
            .state
            .get_track(&function.name, &var.name)
            .context("Cannot find track of var")?;

        let instructions = function.instructions.iter().enumerate();
        let mut graph = Graph::default();

        let mut facts = Vec::new();
        let _ = self.build_next(
            &var,
            track,
            &mut facts,
            function.instructions.len(),
            instructions.skip(pc),
            &mut graph,
            false,
        )?;

        let init = Fact::from_var(
            &var,
            0,
            facts
                .last()
                .map(|x| x.pc)
                .unwrap_or(function.instructions.len()),
            track,
        );
        graph.add_normal(init, facts.last().context("Cannot get last fact")?.clone())?;

        {
            self.inner
                .insert((function.name.clone(), var.name.clone()), (pc, graph));
        }

        let (_, ref graph) = self
            .inner
            .get(&(function.name.clone(), var.name.clone()))
            .context("Cannot find chained facts")?;

        Ok(graph)
    }

    /// Recursive function for constructing graph
    fn build_next<'a>(
        &'a self,
        var: &Variable,
        track: usize,
        facts: &mut Vec<Fact>,
        max_len: usize,
        mut instructions: impl Iterator<Item = (usize, &'a Instruction)>,
        graph: &mut Graph,
        init_def: bool,
    ) -> Result<Fact> {
        let instruction = instructions.next();

        if let Some((pc, instruction)) = instruction {
            let is_lhs = self.is_lhs_used(&var, instruction);
            let is_rhs = self.is_rhs_used(&var, instruction);

            if !is_lhs || !init_def {
                let next_fact =
                    self.build_next(var, track, facts, max_len, instructions, graph, true)?;
                if is_rhs {
                    let x = Fact::from_var(&var, pc, next_fact.pc, track);
                    facts.push(x.clone());

                    graph.add_normal(x.clone(), next_fact)?;

                    return Ok(x);
                } else {
                    return Ok(next_fact);
                }
            } else {
                let next_fact =
                    self.build_next(var, track, facts, max_len, instructions, graph, init_def)?;
                let x = Fact::from_var(&var, pc, next_fact.pc, track);
                facts.push(x.clone());
                graph.add_normal(x.clone(), next_fact)?;
                return Ok(x); //redefinition, therefore not tainting anymore
            }
        } else {
            // Create end
            let x = Fact::from_var(&var, max_len, max_len, track);
            facts.push(x.clone());
            return Ok(x);
        }
    }

    fn is_lhs_used(&self, variable: &Variable, instruction: &Instruction) -> bool {
        let var = &variable.name;

        match instruction {
            Instruction::Const(dest, _) if dest == var => true,
            Instruction::Assign(dest, _src) if dest == var => true,
            Instruction::BinOp(dest, _, _) if dest == var => true,
            _ => false,
        }
    }

    fn is_rhs_used(&self, variable: &Variable, instruction: &Instruction) -> bool {
        let var = &variable.name;

        match instruction {
            Instruction::Assign(_dest, src) if src == var => true,
            Instruction::BinOp(_, src1, src2) if src1 == var || src2 == var => true,
            _ => false,
        }
    }

    /*
    fn is_used(&self, variable: &Variable, instruction: &Instruction) -> bool {
        /*
        if variable.is_memory {
            // We cannot check memory vars by name, because they might differ
            // But by semantics, every memory should be considered
            return self.is_used_mem(variable, instruction);
        }

        if variable.is_taut {
            // Like mem

            return self.is_used_taut(variable, instruction);
        }*/

        let var = &variable.name;
        match instruction {
            Instruction::Unop(dest, _src) if var == dest => false,
            Instruction::Unop(_dest, src) if var == src => true,
            Instruction::BinOp(dest, _src1, _src2) if dest == var => false,
            Instruction::BinOp(_dest, src1, src2) if src1 == var || src2 == var => true,
            Instruction::Const(dest, _) => false,
            Instruction::Assign(dest, src) if var == dest=> var == dest || var == src,
            Instruction::Call(_callee, params, dest) => params.contains(var) || dest.contains(var),
            Instruction::CallIndirect(_callee, params, dest) => {
                params.contains(var) || dest.contains(var)
            }
            Instruction::Kill(dest) => var == dest,
            Instruction::Conditional(dest, _) => var == dest,
            Instruction::Return(dest) => dest.contains(var),
            Instruction::Phi(dest, src1, src2) => var == dest || var == src1 || var == src2,
            Instruction::Store(src1, _, src2) => var == src1 || var == src2,
            Instruction::Load(dest, _, src) => var == dest || var == src,
            _ => false,
        }
    }*/

    fn is_used_mem(&self, _variable: &Variable, instruction: &Instruction) -> bool {
        macro_rules! is_mem {
            ($e:ident) => {
                $e.starts_with("mem")
            };
        }

        match instruction {
            Instruction::Unop(dest, src) => is_mem!(dest) || is_mem!(src),
            Instruction::BinOp(dest, src1, src2) => is_mem!(dest) || is_mem!(src1) || is_mem!(src2),
            Instruction::Const(dest, _) => is_mem!(dest),
            Instruction::Assign(dest, src) => is_mem!(dest) || is_mem!(src),
            Instruction::Call(_callee, params, dest) => {
                params.iter().any(|x| is_mem!(x)) || dest.iter().any(|x| is_mem!(x))
            }
            Instruction::CallIndirect(_callee, params, dest) => {
                params.iter().any(|x| is_mem!(x)) || dest.iter().any(|x| is_mem!(x))
            }
            Instruction::Kill(dest) => is_mem!(dest),
            Instruction::Conditional(dest, _) => is_mem!(dest),
            Instruction::Return(dest) => dest.iter().any(|x| is_mem!(x)),
            Instruction::Phi(dest, src1, src2) => is_mem!(dest) || is_mem!(src1) || is_mem!(src2),
            Instruction::Store(src1, _, src2) => is_mem!(src1) || is_mem!(src2),
            Instruction::Load(dest, _, src) => is_mem!(dest) || is_mem!(src),
            _ => false,
        }
    }

    fn is_used_taut(&self, _variable: &Variable, instruction: &Instruction) -> bool {
        match instruction {
            Instruction::Const(_dest, _) => true,
            _ => false,
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::icfg::state::State;
    use crate::ir::ast::Program;
    use insta::assert_debug_snapshot as assert_snapshot;

    #[test]
    fn test_building_scfg() {
        /*
            0 - %0 = 1
            1 - %1 = 1
            2 - %2 = %0 op %1
            3 - %2 = %1 op %0

            %0:
                0 -> 2
                2 -> 3

            %1:
                0 -> 1
                1 -> 2
                2 -> 3
        */
        let func_name = "main".to_string();
        let function = AstFunction {
            name: func_name.clone(),
            definitions: vec!["%0".to_string(), "%1".to_string(), "%2".to_string()],
            instructions: vec![
                Instruction::Const("%0".to_string(), 1.0),
                Instruction::Const("%1".to_string(), 1.0),
                Instruction::BinOp("%2".to_string(), "%0".to_string(), "%1".to_string()),
                Instruction::BinOp("%2".to_string(), "%1".to_string(), "%0".to_string()),
            ],
            ..Default::default()
        };

        let mut graph = Graph::default();
        let mut state = State::default();

        let mut ctx = Ctx {
            graph: &mut graph,
            state: &mut state,
            prog: &Program {
                functions: vec![function.clone()],
            },
        };

        let pc = 0;

        // fullfilling precondition of `chain.cache()`
        ctx.state.init_function(&function, pc).unwrap();

        let mut chain = DefUseChain::default();
        let facts = chain
            .cache(&mut ctx, &function, &"%0".to_string(), pc)
            .unwrap()
            .flatten()
            .collect::<Vec<_>>();

        assert_snapshot!("building_defuse_reg_0_scfg", facts);

        let facts = chain
            .cache(&mut ctx, &function, &"%1".to_string(), pc)
            .unwrap()
            .flatten()
            .collect::<Vec<_>>();

        assert_snapshot!("building_defuse_reg_1_scfg", facts);
    }

    #[test]
    fn testing_caching_first_var() {
        let func_name = "main".to_string();
        let function = AstFunction {
            name: func_name.clone(),
            definitions: vec!["%0".to_string(), "%1".to_string(), "%2".to_string()],
            instructions: vec![
                Instruction::Const("%0".to_string(), 1.0),
                Instruction::Const("%1".to_string(), 1.0),
                Instruction::Const("%2".to_string(), 1.0),
                Instruction::Const("%0".to_string(), 1.0),
                Instruction::Const("%1".to_string(), 1.0),
            ],
            ..Default::default()
        };

        let mut graph = Graph::default();
        let mut state = State::default();

        let mut ctx = Ctx {
            graph: &mut graph,
            state: &mut state,
            prog: &Program {
                functions: vec![function.clone()],
            },
        };

        let pc = 0;

        // fullfilling precondition of `chain.cache()`
        ctx.state.init_function(&function, pc).unwrap();

        let mut chain = DefUseChain::default();
        let facts = chain
            .cache(&mut ctx, &function, &"%0".to_string(), pc)
            .unwrap()
            .flatten()
            .collect::<Vec<_>>();

        assert_snapshot!("defuse_reg_0_scfg", facts);

        assert_eq!(3, facts.len());
        assert_eq!(3, facts.get(1).unwrap().next_pc);
        assert_eq!(5, facts.get(2).unwrap().next_pc);

        let before = chain
            .points_to(&mut ctx, &function, &"%0".to_string(), 3)
            .unwrap();
        assert_eq!(1, before.len());
        assert_eq!(3, before.get(0).unwrap().next_pc);

        let after = chain
            .demand(&mut ctx, &function, &"%0".to_string(), 0)
            .unwrap();
        assert_eq!(2, after.len());
        assert_eq!(3, after.get(0).unwrap().pc);
        assert_eq!(5, after.get(0).unwrap().next_pc);
    }

    #[test]
    fn testing_caching_second_var() {
        let func_name = "main".to_string();
        let function = AstFunction {
            name: func_name.clone(),
            definitions: vec!["%0".to_string(), "%1".to_string(), "%2".to_string()],
            instructions: vec![
                Instruction::Const("%0".to_string(), 1.0),
                Instruction::Const("%1".to_string(), 1.0),
                Instruction::Const("%2".to_string(), 1.0),
                Instruction::Const("%0".to_string(), 1.0),
                Instruction::Const("%1".to_string(), 1.0),
            ],
            ..Default::default()
        };

        let mut graph = Graph::default();
        let mut state = State::default();

        let mut ctx = Ctx {
            graph: &mut graph,
            state: &mut state,
            prog: &Program {
                functions: vec![function.clone()],
            },
        };

        let pc = 0;

        // fullfilling precondition of `chain.cache()`
        ctx.state.init_function(&function, pc).unwrap();

        let mut chain = DefUseChain::default();
        let facts = chain
            .cache(&mut ctx, &function, &"%1".to_string(), pc)
            .unwrap()
            .flatten()
            .collect::<Vec<_>>();

        assert_snapshot!("defuse_reg_1_scfg", facts);

        assert_eq!(4, facts.len());
        assert_eq!(4, facts.get(1).unwrap().next_pc);
        assert_eq!(1, facts.get(2).unwrap().next_pc);

        let before = chain
            .points_to(&mut ctx, &function, &"%1".to_string(), 1)
            .unwrap();
        assert_eq!(1, before.len());
        assert_eq!(1, before.get(0).unwrap().next_pc);

        let after = chain
            .demand(&mut ctx, &function, &"%1".to_string(), 1)
            .unwrap();
        assert_eq!(2, after.len());
        assert_eq!(4, after.get(0).unwrap().pc);
        assert_eq!(5, after.get(0).unwrap().next_pc);
    }

    #[test]
    fn testing_caching_recursion() {
        let func_name = "main".to_string();
        let function = AstFunction {
            name: func_name.clone(),
            definitions: vec!["%0".to_string(), "%1".to_string(), "%2".to_string()],
            instructions: vec![
                Instruction::Const("%0".to_string(), 1.0),
                Instruction::Const("%1".to_string(), 1.0),
                Instruction::Const("%2".to_string(), 1.0),
                Instruction::Const("%0".to_string(), 1.0),
                Instruction::Const("%1".to_string(), 1.0),
            ],
            ..Default::default()
        };

        let mut graph = Graph::default();
        let mut state = State::default();

        let mut ctx = Ctx {
            graph: &mut graph,
            state: &mut state,
            prog: &Program {
                functions: vec![function.clone()],
            },
        };

        let pc = 2;

        // fullfilling precondition of `chain.cache()`
        ctx.state.init_function(&function, pc).unwrap();

        let mut chain = DefUseChain::default();
        let facts = chain
            .cache(&mut ctx, &function, &"%0".to_string(), pc)
            .unwrap()
            .flatten()
            .collect::<Vec<_>>();
        assert_eq!(3, facts.len());

        let facts = chain
            .cache(&mut ctx, &function, &"%0".to_string(), 0)
            .unwrap()
            .flatten()
            .collect::<Vec<_>>();
        assert_eq!(4, facts.len());
    }
}
