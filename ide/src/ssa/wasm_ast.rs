#![allow(dead_code)]
use anyhow::{bail, Result};
/// This module is responsible to parse
/// the webassembly AST to an IR
use funky::engine::func::FuncInstance;
use funky::engine::Engine;
use log::debug;
use std::fmt::Write;
use wasm_parser::core::Instruction::*;
use wasm_parser::core::*;
use crate::symbol_table::SymbolTable;

use std::collections::HashMap;

#[derive(Debug)]
pub struct IR<'a> {
    //functions: Vec<Function>,
    buffer: String,
    //counter: Counter,
    symbol_table: SymbolTable,
    block_counter: Counter,
    function_counter: Counter,
    functions: Vec<Function>,
    engine: &'a Engine,
}

#[derive(Debug)]
struct Function {
    name: String,
    locals: HashMap<usize, usize>,  //key to register
    globals: HashMap<usize, usize>, //key to register
}

#[derive(Debug)]
struct Block {
    name: usize,
    is_loop: bool,
}

#[derive(Debug)]
struct JumpList {
    blocks: Vec<Block>,
}

#[derive(Debug, Default)]
struct Counter {
    counter: usize,
}

impl Counter {
    pub fn peek(&self) -> usize {
        self.counter
    }

    pub fn get(&mut self) -> usize {
        let counter = self.counter.clone();
        self.counter += 1;
        counter
    }

    pub fn peek_next(&self) -> usize {
        self.counter + 1
    }
}


impl<'a> IR<'a> {
    pub fn new(engine: &'a Engine) -> Self {
        Self {
            engine,
            buffer: String::new(),
            //counter: Counter::default(),
            symbol_table: SymbolTable::default(),
            block_counter: Counter::default(),
            function_counter: Counter::default(),
            functions: Vec::new(),
        }
    }

    pub fn buffer(&self) -> String {
        self.buffer.clone()
    }

    pub fn visit(&mut self) -> Result<()> {
        for function in self.engine.store.funcs.iter() {
            self.visit_function(function)?;
        }

        Ok(())
    }

    fn visit_function(&mut self, inst: &FuncInstance) -> Result<()> {
        let name = format!("{}", self.function_counter.get());
        writeln!(self.buffer, "define {} {{", name).unwrap();

        let mut function = Function {
            name,
            locals: HashMap::new(),
            globals: HashMap::new(),
        };

        for (i, _) in inst.ty.param_types.iter().enumerate() {
            function.locals.insert(i, self.symbol_table.new_var()?);
        }

        self.functions.push(function);
        let func_index = self.functions.len() - 1;

        debug!("code {:#?}", inst.code);
        self.visit_body(&inst.code, func_index, inst.ty.return_types.len())?;

        writeln!(self.buffer, "}};").unwrap();

        Ok(())
    }

    fn visit_body(&mut self, body: &FunctionBody, function_index: usize, return_count: usize) -> Result<()> {
        let code = &body.code;

        let name = self.block_counter.get();
        let then_name = self.block_counter.get();

        let block = Block {
            name: name.clone(),
            is_loop: false,
        };

        let mut blocks = vec![block];

        writeln!(self.buffer, "BLOCK {}", name).unwrap();

        self.visit_instruction_wrapper(code, function_index, &mut blocks, return_count)?;

        writeln!(self.buffer, "BLOCK {}", then_name).unwrap();

        Ok(())
    }

    /// If the execution exits a block (with no jump),
    /// then kill all variables which are not returned.
    fn exit_block(&mut self, block_ty: &BlockType, old_state: usize) -> Result<()> {
        let arity = self.engine.get_return_count_block(block_ty)?;

        for reg in self.symbol_table.vars.iter_mut().rev().skip(arity as usize) {
            if reg.val() <= old_state {
                break;
            }

            if !reg.is_killed {
                reg.is_killed = true;
                writeln!(self.buffer, "KILL %{}", reg.val()).unwrap();
            }
        }

        Ok(())
    }

    fn visit_instruction_wrapper(
        &mut self,
        code: &[InstructionWrapper],
        function_index: usize,
        blocks: &mut Vec<Block>,
        return_arity: usize,
    ) -> Result<()> {
        debug!("Visiting instruction wrapper");

        //let mut str_block = String::new();
        //writeln!(str_block, "BLOCK {}", self.block_counter.get()).unwrap();

        let blocks_len = blocks.len();

        for instr in code.iter() {
            debug!("Instruction {}", instr.get_instruction());

            match instr.get_instruction() {
                OP_BLOCK(ty, code) => {
                    let name = self.block_counter.get();
                    let then_name = self.block_counter.get();

                    let block = Block {
                        name: name.clone(),
                        is_loop: false,
                    };

                    let _tblock = Block {
                        name: then_name.clone(),
                        is_loop: false,
                    };

                    blocks.push(block);

                    writeln!(self.buffer, "BLOCK {}", name.clone()).unwrap();

                    // If the block exits, then kill all variable to `current_reg`
                    let current_reg = self.symbol_table.peek()?;

                    let arity = self.engine.get_return_count_block(ty)?;
                    self.visit_instruction_wrapper(
                        code.get_instructions(),
                        function_index,
                        blocks,
                        arity as usize
                    )?;

                    self.exit_block(ty, current_reg)?;

                    blocks.pop();

                    writeln!(
                        self.buffer,
                        "GOTO {} // BLOCK ended for {}",
                        then_name,
                        name.clone()
                    )
                    .unwrap();
                    writeln!(
                        self.buffer,
                        "BLOCK {} // THEN block for {}",
                        then_name, name
                    )
                    .unwrap();
                }
                /*
                OP_LOOP(ty, code) => {
                    let name = self.block_counter.get();
                    let then_name = self.block_counter.get();

                    let block = Block {
                        name: name.clone(),
                        is_loop: true,
                    };

                    let _tblock = Block {
                        name: then_name.clone(),
                        is_loop: false,
                    };

                    blocks.push(block);

                    writeln!(self.buffer, "BLOCK {} // LOOP", name.clone()).unwrap();

                    let arity = self.engine.get_return_count_block(ty)?;
                    self.visit_instruction_wrapper(
                        code.get_instructions(),
                        function_index,
                        blocks,
                        arity as usize,
                    )?;

                    blocks.pop();

                    writeln!(
                        self.buffer,
                        "GOTO {} // BLOCK ended for {}",
                        then_name,
                        name.clone()
                    )
                    .unwrap();
                    writeln!(
                        self.buffer,
                        "BLOCK {} // THEN block for {}",
                        then_name, name
                    )
                    .unwrap();
                }
                OP_IF(ty, code) => {
                    let name = self.block_counter.get();
                    let then_name = self.block_counter.get();

                    let block = Block {
                        name: name.clone(),
                        is_loop: false,
                    };

                    let _tblock = Block {
                        name: then_name.clone(),
                        is_loop: false,
                    };

                    blocks.push(block);

                    writeln!(
                        self.buffer,
                        "IF %{} THEN GOTO {} ELSE GOTO {}",
                        self.symbol_table.peek()?,
                        name.clone(),
                        then_name.clone()
                    )
                    .unwrap();
                    writeln!(self.buffer, "BLOCK {} ", name.clone()).unwrap();

                    let arity = self.engine.get_return_count_block(ty)?;
                    self.visit_instruction_wrapper(
                        code.get_instructions(),
                        function_index,
                        blocks,
                        arity as usize,
                    )?;

                    blocks.pop();

                    writeln!(
                        self.buffer,
                        "GOTO {} // BLOCK ended for {}",
                        then_name,
                        name.clone()
                    )
                    .unwrap();
                    writeln!(
                        self.buffer,
                        "BLOCK {} // THEN block for {}",
                        then_name, name
                    )
                    .unwrap();
                }
                OP_IF_AND_ELSE(ty, code1, code2) => {
                    let name = self.block_counter.get();
                    let then_name = self.block_counter.get();
                    let done_name = self.block_counter.get();

                    let block = Block {
                        name: name.clone(),
                        is_loop: false,
                    };

                    let tblock = Block {
                        name: then_name.clone(),
                        is_loop: false,
                    };

                    let _done_block = Block {
                        name: done_name.clone(),
                        is_loop: false,
                    };

                    blocks.push(block);

                    writeln!(
                        self.buffer,
                        "IF %{} THEN GOTO {} ELSE GOTO {}",
                        self.symbol_table.peek()?,
                        name.clone(),
                        then_name.clone()
                    )
                    .unwrap();
                    writeln!(self.buffer, "BLOCK {} ", name.clone()).unwrap();

                    let arity = self.engine.get_return_count_block(ty)?;
                    self.visit_instruction_wrapper(
                        code1.get_instructions(),
                        function_index,
                        blocks,
                        arity as usize,
                    )?;

                    writeln!(
                        self.buffer,
                        "GOTO {} // BLOCK ended for {}",
                        done_name,
                        name.clone()
                    )
                    .unwrap();
                    writeln!(
                        self.buffer,
                        "BLOCK {} // THEN block for {}",
                        then_name, name
                    )
                    .unwrap();

                    blocks.pop();

                    blocks.push(tblock);

                    self.visit_instruction_wrapper(
                        code2.get_instructions(),
                        function_index,
                        blocks,
                        arity as usize,
                    )?;

                    blocks.pop();

                    writeln!(
                        self.buffer,
                        "GOTO {} // BLOCK ended for {}",
                        done_name,
                        name.clone()
                    )
                    .unwrap();

                    writeln!(
                        self.buffer,
                        "BLOCK {} // THEN block for {}",
                        done_name, then_name
                    )
                    .unwrap();
                }*/
                OP_BR(label) => {
                    let jmp_index = blocks_len - 1 - *label as usize;

                    let block = blocks.get(jmp_index).unwrap();

                    if block.is_loop {
                        writeln!(self.buffer, "GOTO {}", block.name).unwrap();
                    } else {
                        writeln!(self.buffer, "GOTO {}", block.name + 1).unwrap();
                    }
                }
                OP_BR_IF(label) => {
                    let jmp_index = blocks_len - 1 - *label as usize;

                    let block = blocks.get(jmp_index).unwrap();

                    if block.is_loop {
                        writeln!(
                            self.buffer,
                            "IF %{} THEN GOTO {}",
                            self.symbol_table.peek()?,
                            block.name
                        )
                        .unwrap();
                    } else {
                        writeln!(
                            self.buffer,
                            "IF %{} THEN GOTO {}",
                            self.symbol_table.peek()?,
                            block.name + 1
                        )
                        .unwrap();
                    }
                }
                OP_BR_TABLE(labels, else_lb) => {
                    let indices: Vec<_> = labels
                        .iter()
                        .map(|x| {
                            let i = blocks_len - 1 - *x as usize;

                            let block = blocks.get(i).unwrap();
                            if block.is_loop {
                                block.name
                            } else {
                                block.name + 1
                            }
                        })
                        .map(|x| format!("{}", x))
                        .collect();

                    let jmp_index = blocks_len - 1 - *else_lb as usize;
                    let block = blocks.get(jmp_index).unwrap();

                    let jmp_index = match block.is_loop {
                        true => block.name,
                        false => block.name + 1,
                    };

                    writeln!(
                        self.buffer,
                        "BR TABLE GOTO {} ELSE GOTO {}",
                        indices.join(" "),
                        jmp_index
                    )
                    .unwrap();
                }
                OP_LOCAL_GET(index) => {
                    writeln!(
                        self.buffer,
                        "%{} = %{}",
                        self.symbol_table.new_var()?,
                        self.functions
                            .get(function_index)
                            .unwrap()
                            .locals
                            .get(&(*index as usize))
                            .unwrap()
                    )
                    .unwrap();
                }
                OP_LOCAL_SET(index) => {
                    writeln!(
                        self.buffer,
                        "%{} = %{}",
                        self.functions
                            .get(function_index)
                            .unwrap()
                            .locals
                            .get(&(*index as usize))
                            .unwrap(),
                        self.symbol_table.peek()?
                    )
                    .unwrap();
                }
                OP_LOCAL_TEE(index) => {
                    println!("{:?}", self.symbol_table);
                    let peek = self.symbol_table.peek()?;
                    // Push only once because the old still lives
                    writeln!(self.buffer, "%{} = %{}", self.symbol_table.new_var()?, peek).unwrap();
                    writeln!(
                        self.buffer,
                        "%{} = %{}",
                        self.functions
                            .get(function_index)
                            .unwrap()
                            .locals
                            .get(&(*index as usize))
                            .unwrap(),
                        peek
                    )
                    .unwrap();
                }
                OP_CALL(func) => {
                    let addr = self.engine.module.lookup_function_addr(*func)?;
                    let instance = self.engine.store.get_func_instance(&addr)?;

                    let num_params = instance.ty.param_types.len();

                    let mut param_regs = Vec::new();

                    for i in 0..num_params {
                        param_regs.push(format!("{}", self.symbol_table.peek_offset(i)?));
                    }

                    writeln!(self.buffer, "CALL {}({})", func, param_regs.join(",")).unwrap();
                }
                OP_I32_CONST(a) => {
                    writeln!(self.buffer, "%{} = {}", self.symbol_table.new_var()?, a).unwrap();
                }
                OP_I64_CONST(a) => {
                    writeln!(self.buffer, "%{} = {}", self.symbol_table.new_var()?, a).unwrap();
                }
                OP_F32_CONST(a) => {
                    writeln!(self.buffer, "%{} = {}", self.symbol_table.new_var()?, a).unwrap();
                }
                OP_F64_CONST(a) => {
                    writeln!(self.buffer, "%{} = {}", self.symbol_table.new_var()?, a).unwrap();
                }
                OP_RETURN => {
                    let mut regs = Vec::new(); 

                    for i in 0..return_arity {
                        regs.push(format!("%{}", self.symbol_table.peek_offset(i)?));
                    }

                    writeln!(self.buffer, "return {}", regs.join(" ")).unwrap();
                }
                OP_I32_CLZ
                | OP_I32_CTZ
                | OP_I32_POPCNT
                | OP_I64_CLZ
                | OP_I64_CTZ
                | OP_I64_POPCNT
                | OP_F32_ABS
                | OP_F32_NEG
                | OP_F32_CEIL
                | OP_F32_FLOOR
                | OP_F32_TRUNC
                | OP_F32_NEAREST
                | OP_F32_SQRT
                | OP_F64_ABS
                | OP_F64_NEG
                | OP_F64_CEIL
                | OP_F64_FLOOR
                | OP_F64_TRUNC
                | OP_F64_NEAREST
                | OP_F64_SQRT
                | OP_I32_WRAP_I64
                | OP_I32_TRUNC_F32_S
                | OP_I32_TRUNC_F32_U
                | OP_I32_TRUNC_F64_S
                | OP_I32_TRUNC_F64_U
                | OP_I64_EXTEND_I32_U
                | OP_I64_EXTEND_I32_S
                | OP_I64_TRUNC_F32_S
                | OP_I64_TRUNC_F32_U
                | OP_I64_TRUNC_F64_S
                | OP_I64_TRUNC_F64_U
                | OP_F32_CONVERT_I32_S
                | OP_F32_CONVERT_I32_U
                | OP_F32_CONVERT_I64_S
                | OP_F32_CONVERT_I64_U
                | OP_F32_DEMOTE_F64
                | OP_F64_CONVERT_I32_S
                | OP_F64_CONVERT_I32_U
                | OP_F64_CONVERT_I64_S
                | OP_F64_CONVERT_I64_U
                | OP_F64_PROMOTE_F32
                | OP_I32_REINTERPRET_F32
                | OP_I64_REINTERPRET_F64
                | OP_F32_REINTERPRET_I32
                | OP_F64_REINTERPRET_I64
                | OP_I32_EXTEND8_S
                | OP_I32_EXTEND16_S
                | OP_I64_EXTEND8_S
                | OP_I64_EXTEND16_S
                | OP_I64_EXTEND32_S
                | OP_I32_TRUNC_SAT_F32_S
                | OP_I32_TRUNC_SAT_F32_U
                | OP_I32_TRUNC_SAT_F64_S
                | OP_I32_TRUNC_SAT_F64_U
                | OP_I64_TRUNC_SAT_F32_S
                | OP_I64_TRUNC_SAT_F32_U
                | OP_I64_TRUNC_SAT_F64_S
                | OP_I64_TRUNC_SAT_F64_U => {
                    writeln!(
                        self.buffer,
                        "%{} = {} %{}",
                        self.symbol_table.new_var()?,
                        "op",
                        self.symbol_table.peek_offset(2)?, // TODO Check why `2`
                    )
                    .unwrap();
                }
                OP_I32_ADD | OP_I32_SUB | OP_I32_MUL | OP_I32_DIV_S | OP_I32_DIV_U
                | OP_I32_REM_S | OP_I32_REM_U | OP_I32_AND | OP_I32_OR | OP_I32_XOR
                | OP_I32_SHL | OP_I32_SHR_S | OP_I32_SHR_U | OP_I32_ROTL | OP_I32_ROTR
                | OP_I64_ADD | OP_I64_SUB | OP_I64_MUL | OP_I64_DIV_S | OP_I64_DIV_U
                | OP_I64_REM_S | OP_I64_REM_U | OP_I64_AND | OP_I64_OR | OP_I64_XOR
                | OP_I64_SHL | OP_I64_SHR_S | OP_I64_SHR_U | OP_I64_ROTL | OP_I64_ROTR
                | OP_I32_EQZ | OP_I32_EQ | OP_I32_NE | OP_I32_LT_S | OP_I32_LT_U | OP_I32_GT_S
                | OP_I32_GT_U | OP_I32_LE_S | OP_I32_LE_U | OP_I32_GE_S | OP_I32_GE_U
                | OP_I64_EQZ | OP_I64_EQ | OP_I64_NE | OP_I64_LT_S | OP_I64_LT_U | OP_I64_GT_S
                | OP_I64_GT_U | OP_I64_LE_S | OP_I64_LE_U | OP_I64_GE_S | OP_I64_GE_U
                | OP_F32_EQ | OP_F32_NE | OP_F32_LT | OP_F32_GT | OP_F32_LE | OP_F32_GE
                | OP_F64_EQ | OP_F64_NE | OP_F64_LT | OP_F64_GT | OP_F64_LE | OP_F64_GE
                | OP_F32_ADD | OP_F32_SUB | OP_F32_MUL | OP_F32_DIV | OP_F64_ADD | OP_F64_SUB
                | OP_F64_MUL | OP_F64_DIV | OP_F32_MIN | OP_F32_MAX | OP_F32_COPYSIGN
                | OP_F64_MIN | OP_F64_MAX | OP_F64_COPYSIGN => {
                    writeln!(
                        self.buffer,
                        "%{} = %{} {} %{}",
                        self.symbol_table.new_var()?,
                        self.symbol_table.peek_offset(1)?,
                        "op",
                        self.symbol_table.peek_offset(2)?
                    )
                    .unwrap();
                }
                _ => {
                    writeln!(self.buffer, "{}", instr.get_instruction()).unwrap();
                }
            }
        }

        Ok(())

        /*
        if !jumped {
            writeln!(str_block, "GOTO {}", self.block_counter.peek()).unwrap();
        }*/

        //writeln!(str_block, "BLOCK {}", self.block_counter.get()).unwrap();

        //writeln!(self.buffer, "{}", str_block).unwrap();
    }
}
