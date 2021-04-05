//! This project allows to do taint analysis for web assembly.

#[macro_use]
extern crate lalrpop_util;

lalrpop_mod!(pub grammar);

pub mod counter;
pub mod icfg;
pub mod ir;
pub mod solver;
pub mod symbol_table;

#[cfg(test)]
mod tests;