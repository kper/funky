use crate::counter::Counter;
use anyhow::{bail, Result};
use log::debug;

#[derive(Debug, Default)]
pub struct Variable {
    pub reg: usize,
    pub is_killed: bool,
}

impl Variable {
    pub fn val(&self) -> usize {
        self.reg
    }
}

#[derive(Debug, Default)]
pub struct SymbolTable {
    pub vars: Vec<Variable>,
    counter: Counter,
}

impl SymbolTable {
    pub fn peek(&self) -> Result<usize> {
        debug!("Peeking symbol");
        for var in self.vars.iter().rev() {
            if !var.is_killed {
                return Ok(var.reg);
            }
        }

        bail!("No active variable in symbol table");
    }

    /// Peek the last variable, but skip `offset` variables.
    pub fn peek_offset(&self, offset: usize) -> Result<usize> {
        debug!("Peeking symbol with offset {}", offset);

        for var in self.vars.iter().filter(|x| !x.is_killed).rev().skip(offset) {
            return Ok(var.reg);
        }

        bail!("No active variable in symbol table");
    }

    pub fn new_var(&mut self) -> Result<usize> {
        let val = self.counter.get();

        debug!("Getting new var {}", val);

        self.vars.push(Variable {
            reg: val,
            is_killed: false,
        });

        Ok(val)
    }

    /// Kill the variable with `reg`.
    /// The search starts at the end, because
    /// it is more likely that the killed variable is there.
    pub fn kill(&mut self, reg: usize) -> Result<()> {
        for var in self.vars.iter_mut().rev() {
            if var.reg == reg {
                if !var.is_killed {
                    var.is_killed = true;
                    return Ok(());
                } else {
                    bail!("Variable %{} was already killed", reg);
                }
            }
        }

        bail!("Variable %{} was not found", reg);
    }
}