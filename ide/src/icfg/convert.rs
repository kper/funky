use std::ops::Sub;

use crate::counter::Counter;
use crate::icfg::graph::SubGraph;
use crate::symbol_table::SymbolTable;
use anyhow::{bail, Result};
use funky::engine::Engine;
/// This module is responsible to parse
/// the webassembly AST to a graph
use funky::engine::{func::FuncInstance, FunctionBody, InstructionWrapper};
use log::debug;
use wasm_parser::core::Instruction::*;
use wasm_parser::core::*;

#[derive(Debug)]
pub struct Convert<'a> {
    engine: &'a Engine,
    symbol_table: SymbolTable,
    block_counter: Counter,
}

#[derive(Debug)]
struct Block {
    name: usize,
    is_loop: bool,
}

impl<'a> Convert<'a> {
    pub fn new(engine: &'a Engine) -> Self {
        Self {
            engine,
            symbol_table: SymbolTable::default(),
            block_counter: Counter::default(),
        }
    }

    pub fn visit(&mut self) -> Result<()> {
        for function in self.engine.store.funcs.iter() {
            self.visit_function(function)?;
        }

        Ok(())
    }

    fn visit_function(&mut self, inst: &FuncInstance) -> Result<()> {
        let mut graph = SubGraph::new();

        for (i, _) in inst.ty.param_types.iter().enumerate() {
            //function.locals.insert(i, self.symbol_table.new_var()?);
            graph.add_var();
        }

        self.visit_body(&inst.code)?;

        Ok(())
    }

    fn visit_body(&mut self, body: &FunctionBody) -> Result<()> {
        let code = &body.code;

        let name = self.block_counter.get();
        let then_name = self.block_counter.get();

        let block = Block {
            name: name.clone(),
            is_loop: false,
        };

        let mut blocks = vec![block];

        self.visit_instruction_wrapper(code, &mut blocks)?;

        Ok(())
    }

    fn visit_instruction_wrapper(
        &mut self,
        code: &[InstructionWrapper],
        blocks: &mut Vec<Block>,
    ) -> Result<()> {
        for instr in code.iter() {
            debug!("Instruction {}", instr.get_instruction());

           
        }

        Ok(())
    }
}
