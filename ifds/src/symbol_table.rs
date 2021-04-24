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
    pub fn val(&self) -> Result<isize> {
        self.reg.val()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum Reg {
    Normal(usize),
    Global(isize),
}

impl Reg {
    pub fn val(&self) -> Result<isize> {
        Ok(match *self {
            Reg::Normal(x) => x as isize,
            Reg::Global(x) => x,
        })
    }
}

impl fmt::Display for Reg {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match *self {
            Reg::Normal(x) => write!(f, "%{}", x),
            Reg::Global(x) => write!(f, "%{}", x),
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

    pub fn new_reg(&mut self) -> Result<Reg> {
        let val = self.counter.get();
        let val = Reg::Normal(val);

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

    /// Returns how many variables are alive
    pub fn count_alive_vars(&self) -> usize {
        self.vars.iter().filter(|x| !x.is_killed).count()
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_symbol_table_peek_offset() {
        let mut sym_table = SymbolTable::default();

        let mut regs = Vec::new();
        for _i in 0..10 {
            regs.push(sym_table.new_reg().unwrap());
        }

        for i in regs.iter().rev().take(9) {
            sym_table.kill(i).unwrap();
        }

        assert_eq!(1, sym_table.vars.iter().filter(|x| !x.is_killed).count());
        assert_eq!(9, sym_table.vars.iter().filter(|x| x.is_killed).count());

        assert_eq!(&Reg::Normal(0), sym_table.peek_offset(0).unwrap());
        assert_eq!(
            &sym_table.peek().unwrap(),
            sym_table.peek_offset(0).unwrap()
        );
        assert!(sym_table.peek_offset(1).is_err());
    }

    #[test]
    fn test_symbol_table_multiple_peek_offset() {
        let mut sym_table = SymbolTable::default();

        let mut regs = Vec::new();
        for _i in 0..10 {
            regs.push(sym_table.new_reg().unwrap());
        }

        for i in regs.iter().rev().take(3) {
            sym_table.kill(i).unwrap();
        }

        assert_eq!(7, sym_table.vars.iter().filter(|x| !x.is_killed).count());
        assert_eq!(3, sym_table.vars.iter().filter(|x| x.is_killed).count());

        assert_eq!(&Reg::Normal(6), sym_table.peek_offset(0).unwrap());
        assert_eq!(&Reg::Normal(5), sym_table.peek_offset(1).unwrap());
        assert_eq!(&Reg::Normal(4), sym_table.peek_offset(2).unwrap());
    }
}
