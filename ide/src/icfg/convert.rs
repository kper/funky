use std::ops::Sub;

use crate::counter::Counter;
use crate::icfg::graph::SubGraph;
use crate::symbol_table::SymbolTable;
use anyhow::{bail, Result, Context};
use funky::engine::Engine;
/// This module is responsible to parse
/// the webassembly AST to a graph
use funky::engine::{func::FuncInstance, FunctionBody, InstructionWrapper};
use log::debug;
use wasm_parser::core::*;
use crate::ssa::ast::Instruction;
use crate::ssa::ast::Instruction::*;

use crate::grammar::*;

#[derive(Debug)]
pub struct Convert {
    symbol_table: SymbolTable,
    block_counter: Counter,
}

#[derive(Debug)]
struct Block {
    name: usize,
    is_loop: bool,
}

impl Convert {
    pub fn new() -> Self {
        Self {
            symbol_table: SymbolTable::default(),
            block_counter: Counter::default(),
        }
    }

    pub fn visit(&mut self, ir: &'static str) -> Result<SubGraph> {
        debug!("Convert intermediate repr to graph");

        let prog = ProgramParser::new().parse(ir).context("Parsing IR failed")?;

        for function in prog.functions.iter() {
            debug!("Creating graph from function {}", function.name);

            let mut graph = SubGraph::new();

            for instruction in function.instructions.iter() {
                match instruction {
                    Instruction::Const(reg, _val) => {
                        debug!("Adding const");
                        graph.add_var(reg);
                    }
                    Instruction::Assign(dest, src) => {
                        debug!("Assignment");
                        graph.add_assignment(dest, src);
                    }
                    _ => {}
                }

                graph.add_row();
            }

            return Ok(graph);
        }


        bail!("Nothing")
    }
}
