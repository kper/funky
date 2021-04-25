#![allow(dead_code)]

use crate::icfg::graph::*;
use crate::ir::ast::Function as AstFunction;
use crate::ir::ast::Instruction;
use anyhow::{Context, Result};
use std::collections::HashMap;

use crate::icfg::tabulation::sparse::Ctx;

type Function = String;
type Var = String;
type StartPC = usize;

#[derive(Debug, Default)]
pub struct DefUseChain {
    inner: HashMap<(Function, Var, StartPC), Vec<Fact>>,
}

impl DefUseChain {
    /// Get the facts
    pub fn get_facts_at(
        &self,
        function: &String,
        var: &String,
        start_pc: usize,
    ) -> Option<&Vec<Fact>> {
        self.inner.get(&(function.clone(), var.clone(), start_pc))
    }

    /// Get the facts after given `pc`
    pub fn get_facts_at_after_pc(
        &self,
        function: &String,
        var: &String,
        start_pc: usize,
        pc: usize,
    ) -> Option<Vec<&Fact>> {
        self.inner
            .get(&(function.clone(), var.clone(), start_pc))
            .map(|x| x.iter().filter(|x| x.next_pc >= pc).collect::<Vec<_>>())
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
    ) -> Result<Vec<Fact>> {
        let var = ctx
            .state
            .get_var(&function.name, var)
            .context("Variable is not defined in the state")?
            .clone();

        // already exists, therefore returning
        if self
            .inner
            .contains_key(&(function.name.clone(), var.name.clone(), pc))
        {
            return self
                .inner
                .get(&(function.name.clone(), var.name.clone(), pc))
                .context("Cannot find chained facts")
                .map(|x| x.clone());
        }

        let instructions = function
            .instructions
            .iter()
            .enumerate()
            .skip(pc) //TODO maybe start from the start, because recursion?
            .filter(|(_, x)| self.is_used(&var, x));

        let track = ctx
            .state
            .get_track(&function.name, &var.name)
            .context("Cannot find track of var")?;

        let mut facts = Vec::new();
        for (pc, _instruction) in instructions {
            let x = ctx
                .state
                .cache_fact(&function.name, Fact::from_var(&var, pc + 1, track))?;
            facts.push(x.clone());
        }

        // Add last fact for the end of the procedure

        let x = ctx.state.cache_fact(
            &function.name,
            Fact::from_var(&var, function.instructions.len(), track),
        )?;
        facts.push(x.clone());

        // end

        self.inner
            .insert((function.name.clone(), var.name.clone(), pc), facts);

        self.inner
            .get(&(function.name.clone(), var.name.clone(), pc))
            .context("Cannot find chained facts")
            .map(|x| x.clone())
    }

    fn is_used(&self, variable: &Variable, instruction: &Instruction) -> bool {
        if variable.is_memory {
            // We cannot check memory vars by name, because they might differ
            // But by semantics, every memory should be considered
            return self.is_used_mem(variable, instruction);
        }

        if variable.is_taut {
            // Like mem

            return self.is_used_taut(variable, instruction);
        }

        let var = &variable.name;
        match instruction {
            Instruction::Unop(dest, src) => var == dest || var == src,
            Instruction::BinOp(dest, src1, src2) => var == dest || var == src1 || var == src2,
            Instruction::Const(dest, _) => var == dest,
            Instruction::Assign(dest, src) => var == dest || var == src,
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
    }

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
        };

        let pc = 0;

        // fullfilling precondition of `chain.cache()`
        ctx.state.init_function(&function, pc).unwrap();

        let mut chain = DefUseChain::default();
        let facts = chain
            .cache(&mut ctx, &function, &"%0".to_string(), pc)
            .unwrap();
        assert_eq!(3, facts.len());
        assert_eq!(1, facts.get(0).unwrap().next_pc);
        assert_eq!(4, facts.get(1).unwrap().next_pc);
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
        };

        let pc = 0;

        // fullfilling precondition of `chain.cache()`
        ctx.state.init_function(&function, pc).unwrap();

        let mut chain = DefUseChain::default();
        let facts = chain
            .cache(&mut ctx, &function, &"%1".to_string(), pc)
            .unwrap();
        assert_eq!(3, facts.len());
        assert_eq!(2, facts.get(0).unwrap().next_pc);
        assert_eq!(5, facts.get(1).unwrap().next_pc);
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
        };

        let pc = 2;

        // fullfilling precondition of `chain.cache()`
        ctx.state.init_function(&function, pc).unwrap();

        let mut chain = DefUseChain::default();
        let facts = chain
            .cache(&mut ctx, &function, &"%0".to_string(), pc)
            .unwrap();
        assert_eq!(2, facts.len());

        let facts = chain
            .cache(&mut ctx, &function, &"%0".to_string(), 0)
            .unwrap();
        assert_eq!(3, facts.len());
    }
}
