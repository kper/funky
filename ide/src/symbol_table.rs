#![allow(dead_code)]

use std::fmt;

use crate::counter::Counter;
use anyhow::{bail, Result};
use log::debug;

#[derive(Debug, Clone)]
pub struct Variable {
    pub reg: Reg,
    pub is_killed: bool,
}

impl Variable {
    pub fn val(&self) -> Result<usize> {
        self.reg.val()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct Reg(usize);

impl Reg {
    pub fn val(&self) -> Result<usize> {
        Ok(self.0)
    }
}

impl fmt::Display for Reg {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "%{}", self.0)
    }
}

#[derive(Debug, Default)]
pub struct SymbolTable {
    pub vars: Vec<Variable>,
    counter: Counter,
}

impl SymbolTable {
    pub fn len(&self) -> usize {
        self.counter.peek()
    }

    pub fn clear(&mut self) {
        self.vars.clear();
        self.counter = Counter::default();
    }

    pub fn peek(&self) -> Result<Reg> {
        debug!("Peeking symbol");
        for var in self.vars.iter().rev() {
            if !var.is_killed {
                return Ok(var.reg.clone());
            }
        }

        bail!("No active variable in symbol table");
    }

    /// Summarise the last 2 * `return_count` register together with the last `return_count`
    pub fn summarise_phi(&mut self, return_count: u32) -> Result<Vec<(Variable, Variable)>> {
        let first_block = self
            .vars
            .iter()
            .rev()
            .skip(return_count as usize)
            .skip_while(|x| x.is_killed)
            .take((return_count * 2) as usize)
            .cloned();

        let second_block = self
            .vars
            .iter()
            .rev()
            .skip_while(|x| x.is_killed)
            .take(return_count as usize)
            .cloned();

        Ok(first_block.zip(second_block).collect())
    }

    /// Peek the last variable, but skip `offset` variables.
    pub fn peek_offset(&self, offset: usize) -> Result<&Reg> {
        debug!("Peeking symbol with offset {}", offset);

        for var in self.vars.iter().filter(|x| !x.is_killed).rev().skip(offset) {
            return Ok(&var.reg);
        }

        bail!("No active variable in symbol table");
    }

    pub fn new_var(&mut self) -> Result<Reg> {
        let val = self.counter.get();
        let val = Reg(val);

        debug!("Getting new var {:?}", val);

        self.vars.push(Variable {
            reg: val.clone(),
            is_killed: false,
        });

        Ok(val)
    }

    /// Kill the variable with `reg`.
    /// The search starts at the end, because
    /// it is more likely that the killed variable is there.
    pub fn kill(&mut self, reg: &Reg) -> Result<()> {
        for var in self.vars.iter_mut().rev() {
            if &var.reg == reg {
                if !var.is_killed {
                    var.is_killed = true;
                    return Ok(());
                } else {
                    bail!("Variable {:?} was already killed", reg);
                }
            }
        }

        bail!("Variable {:?} was not found", reg);
    }
}
