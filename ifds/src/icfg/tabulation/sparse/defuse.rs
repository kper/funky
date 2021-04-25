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
    /// Build the defuse chain for the instruction
    /// The precondition is that the function must be already initialized.
    /// Because we need the track of the given variable `var`.
    pub(crate) fn cache<'a>(
        &mut self,
        ctx: &mut Ctx<'a>,
        function: &AstFunction,
        var: &Variable,
        pc: usize,
    ) -> Result<Vec<Fact>> {
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
            .filter(|(_, x)| self.is_used(&var.name, x));

        let mut facts = Vec::new();
        for (pc, _instruction) in instructions {
            let track = ctx
                .state
                .get_track(&function.name, &var.name)
                .context("Cannot find track of var")?;
            let x = ctx
                .state
                .cache_fact(&function.name, Fact::from_var(var, pc, track))?;
            facts.push(x.clone());
        }

        self.inner
            .insert((function.name.clone(), var.name.clone(), pc), facts);

        self.inner
            .get(&(function.name.clone(), var.name.clone(), pc))
            .context("Cannot find chained facts")
            .map(|x| x.clone())
    }

    fn is_used(&self, var: &Var, instruction: &Instruction) -> bool {
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
            .cache(
                &mut ctx,
                &function,
                &Variable {
                    name: "%0".to_string(),
                    function: func_name.clone(),
                    ..Default::default()
                },
                pc,
            )
            .unwrap();
        assert_eq!(2, facts.len());
        assert_eq!(0, facts.get(0).unwrap().next_pc);
        assert_eq!(3, facts.get(1).unwrap().next_pc);
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
            .cache(
                &mut ctx,
                &function,
                &Variable {
                    name: "%1".to_string(),
                    function: func_name.clone(),
                    ..Default::default()
                },
                pc,
            )
            .unwrap();
        assert_eq!(2, facts.len());
        assert_eq!(1, facts.get(0).unwrap().next_pc);
        assert_eq!(4, facts.get(1).unwrap().next_pc);
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
            .cache(
                &mut ctx,
                &function,
                &Variable {
                    name: "%0".to_string(),
                    function: func_name.clone(),
                    ..Default::default()
                },
                pc,
            )
            .unwrap();
        assert_eq!(1, facts.len());

        let facts = chain
            .cache(
                &mut ctx,
                &function,
                &Variable {
                    name: "%0".to_string(),
                    function: func_name.clone(),
                    ..Default::default()
                },
                0,
            )
            .unwrap();
        assert_eq!(2, facts.len());
    }
}
