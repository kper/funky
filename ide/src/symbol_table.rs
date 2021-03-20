#![allow(dead_code)]

use std::fmt;

use crate::counter::Counter;
use anyhow::{bail, Context, Result};
use log::debug;

#[derive(Debug)]
pub struct Variable {
    pub reg: Reg,
    pub is_killed: bool,
    pub is_phi: bool,
}

impl Variable {
    pub fn val(&self) -> Result<usize> {
        self.reg.val()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum Reg {
    Normal(usize),
    Phi(Box<Reg>, Box<Reg>),
}

impl Reg {
    pub fn val(&self) -> Result<usize> {
        Ok(match self {
            &Reg::Normal(x) => x,
            _ => bail!("Register is unexpectedly a phi node"),
        })
    }
}

impl fmt::Display for Reg {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            &Reg::Normal(ref x) => write!(f, "%{}", x),
            &Reg::Phi(ref x, ref y) => write!(f, "phi({}, {})", x, y),
        }
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
        for (index, var) in self.vars.iter().enumerate().rev() {
            if !var.is_killed {
                if !var.is_phi {
                    return Ok(var.reg.clone());
                } else {
                    return Ok(Reg::Phi(
                        Box::new(var.reg.clone()),
                        Box::new(
                            self.vars
                                .get(index - 1)
                                .context("Cannot get other phi register")?
                                .reg
                                .clone(),
                        ),
                    ));
                }
            }
        }

        bail!("No active variable in symbol table");
    }

    /// Mark the last `return_count`-times variables in the symbol table.
    /// This means all marked variables can be also meant.
    pub fn mark_phi(&mut self, return_count: u32) -> Result<()> {
        for var in self.vars.iter_mut().rev().take(return_count as usize) {
            var.is_phi = true;
        }

        Ok(())
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
        let val = Reg::Normal(val);

        debug!("Getting new var {:?}", val);

        self.vars.push(Variable {
            reg: val.clone(),
            is_killed: false,
            is_phi: false,
        });

        Ok(val)
    }

    /// Kill the variable with `reg`.
    /// The search starts at the end, because
    /// it is more likely that the killed variable is there.
    pub fn kill(&mut self, reg: Reg) -> Result<()> {
        for var in self.vars.iter_mut().rev() {
            if var.reg == reg {
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
