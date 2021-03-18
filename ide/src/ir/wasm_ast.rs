#![allow(dead_code)]
use crate::symbol_table::SymbolTable;
use anyhow::{Context, Result};
/// This module is responsible to parse
/// the webassembly AST to an IR
use funky::engine::func::FuncInstance;
use funky::engine::Engine;
use log::debug;
use std::fmt::Write;
use wasm_parser::core::Instruction::*;
use wasm_parser::core::*;

use std::collections::HashMap;

#[derive(Debug)]
pub struct IR {
    //functions: Vec<Function>,
    buffer: String,
    //counter: Counter,
    symbol_table: SymbolTable,
    block_counter: Counter,
    function_counter: Counter,
    functions: Vec<Function>,
}

#[derive(Debug)]
struct Function {
    name: String,
    return_count: usize,
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

impl IR {
    pub fn new() -> Self {
        Self {
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

    pub fn visit(&mut self, engine: &Engine) -> Result<()> {
        for function in engine.store.funcs.iter() {
            debug!("Visiting function");

            self.visit_function(function, engine)?;
            self.symbol_table.clear();
        }

        Ok(())
    }

    fn visit_function(&mut self, inst: &FuncInstance, engine: &Engine) -> Result<()> {
        let name = format!("{}", self.function_counter.get());

        let mut function_buffer = String::new();

        write!(function_buffer, "define {} ", name).unwrap();

        let mut function = Function {
            name,
            locals: HashMap::new(),
            globals: HashMap::new(),
            return_count: inst.ty.return_types.len(),
        };

        let mut params = Vec::new();

        debug!("Signature {:?}", inst.ty.param_types);
        debug!("Body code {:?}", inst.code.locals);

        for (i, _) in inst.ty.param_types.iter().enumerate() {
            let var = self.symbol_table.new_var()?;
            debug!("Adding parameter %{}", var);
            function.locals.insert(i, var);
            params.push(var);
        }

        debug!("Adding additional locals");
        for local in inst.code.locals.iter() {
            for _ in 0..local.count {
                match local.ty {
                    _ => {
                        let var = self.symbol_table.new_var()?;
                        debug!("Adding local %{}", var);
                        function.locals.insert(function.locals.len(), var);
                    }
                }
            }
        }

        debug!("locals are {:?}", function.locals);

        if inst.ty.param_types.len() > 0 {
            let params = params
                .into_iter()
                .map(|x| format!("%{}", x))
                .collect::<Vec<String>>()
                .join(" ");
            let param_str = format!("(param {})", params);
            write!(function_buffer, "{} ", param_str).unwrap();
        }

        let result_str = format!("(result {})", inst.ty.return_types.len());
        write!(function_buffer, "{} ", result_str).unwrap();

        self.functions.push(function);
        let func_index = self.functions.len() - 1;

        let mut body = String::new();
        self.visit_body(
            &inst.code,
            func_index,
            inst.ty.return_types.len(),
            engine,
            &mut body,
            inst.ty.param_types.len(),
        )?;

        let nums = (0..self.symbol_table.len())
            .map(|x| format!("%{}", x))
            .collect::<Vec<_>>()
            .join(" ");
        let defs = format!("(define {}) ", nums);
        function_buffer.push_str(&defs);

        // Instructions

        writeln!(function_buffer, "{{").unwrap();
        function_buffer.push_str(&body);

        writeln!(function_buffer, "}};").unwrap();

        self.buffer.push_str(&function_buffer);

        Ok(())
    }

    fn visit_body(
        &mut self,
        body: &FunctionBody,
        function_index: usize,
        return_count: usize,
        engine: &Engine,
        function_buffer: &mut String,
        params_count: usize,
    ) -> Result<()> {
        let code = &body.code;

        let name = self.block_counter.get();
        //let then_name = self.block_counter.get();

        let block = Block {
            name: name.clone(),
            is_loop: false,
        };

        let mut blocks = vec![block];

        writeln!(function_buffer, "BLOCK {}", name).unwrap();

        let current_reg = self.symbol_table.peek().ok();
        self.visit_instruction_wrapper(
            code,
            function_index,
            &mut blocks,
            return_count,
            current_reg,
            engine,
            function_buffer,
            params_count,
        )?;

        {
            // Add last return
            let mut regs = Vec::new();

            //self.exit_block(return_count, current_reg, function_buffer)?;

            for i in 0..return_count {
                regs.push(format!("%{}", self.symbol_table.peek_offset(i)?));
            }

            writeln!(function_buffer, "RETURN {};", regs.join(" ")).unwrap();
        }

        //writeln!(function_buffer, "BLOCK {}", then_name).unwrap();

        Ok(())
    }

    /// If the execution exits a block (with no jump),
    /// then kill all variables which are not returned.
    fn exit_block(
        &mut self,
        arity: usize,
        old_state: Option<usize>,
        function_buffer: &mut String,
        parameters: usize,
    ) -> Result<()> {
        for reg in self
            .symbol_table
            .vars
            .iter_mut()
            .rev()
            .skip(arity + parameters)
        {
            // we must offset parameters
            if reg.val() <= old_state.unwrap_or(0) {
                break;
            }

            if !reg.is_killed {
                reg.is_killed = true;
                writeln!(function_buffer, "KILL %{}", reg.val()).unwrap();
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
        // reg number of the start current_reg: usize,
        current_reg: Option<usize>,
        engine: &Engine,
        function_buffer: &mut String,
        params_count: usize,
    ) -> Result<()> {
        debug!("Visiting instruction wrapper");

        //let mut str_block = String::new();
        //writeln!(str_block, "BLOCK {}", self.block_counter.get()).unwrap();

        let blocks_len = blocks.len();

        for instr in code.iter() {
            debug!("Instruction {}", instr.get_instruction());

            match instr.get_instruction() {
                OP_DROP => {
                    for reg in self.symbol_table.vars.iter_mut().rev() {
                        if !reg.is_killed {
                            reg.is_killed = true;
                            writeln!(function_buffer, "KILL %{}", reg.val()).unwrap();
                            break;
                        }
                    }
                }
                OP_NOP => {
                    // Skip it
                }
                OP_BLOCK(ty, code) => {
                    debug!("Block ty is {:?}", ty);

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

                    writeln!(function_buffer, "BLOCK {}", name.clone()).unwrap();

                    // If the block exits, then kill all variable to `current_reg`
                    let current_reg = self.symbol_table.peek().ok();

                    let arity = engine.get_return_count_block(ty)?;
                    self.visit_instruction_wrapper(
                        code.get_instructions(),
                        function_index,
                        blocks,
                        arity as usize,
                        current_reg,
                        engine,
                        function_buffer,
                        params_count,
                    )?;

                    self.exit_block(arity as usize, current_reg, function_buffer, params_count)?;

                    blocks.pop();

                    writeln!(function_buffer, "GOTO {}", then_name,).unwrap();
                    writeln!(function_buffer, "BLOCK {}", then_name,).unwrap();
                }
                OP_LOOP(ty, code) => {
                    debug!("Block ty is {:?}", ty);

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

                    writeln!(function_buffer, "BLOCK {}", name.clone()).unwrap();

                    let arity = engine.get_return_count_block(ty)?;

                    // If the block exits, then kill all variable to `current_reg`
                    let current_reg = self.symbol_table.peek().ok();

                    self.visit_instruction_wrapper(
                        code.get_instructions(),
                        function_index,
                        blocks,
                        arity as usize,
                        current_reg,
                        &engine,
                        function_buffer,
                        params_count,
                    )?;

                    blocks.pop();

                    writeln!(function_buffer, "GOTO {} ", then_name,).unwrap();
                    writeln!(function_buffer, "BLOCK {}", then_name,).unwrap();
                }
                OP_IF(ty, code) => {
                    debug!("Block ty is {:?}", ty);

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
                        function_buffer,
                        "IF %{} THEN GOTO {} ELSE GOTO {}",
                        self.symbol_table.peek()?,
                        name.clone(),
                        then_name.clone()
                    )
                    .unwrap();
                    writeln!(function_buffer, "BLOCK {} ", name.clone()).unwrap();

                    // If the block exits, then kill all variable to `current_reg`
                    let current_reg = self.symbol_table.peek().ok();

                    let arity = engine.get_return_count_block(ty)?;
                    self.visit_instruction_wrapper(
                        code.get_instructions(),
                        function_index,
                        blocks,
                        arity as usize,
                        current_reg,
                        engine,
                        function_buffer,
                        params_count,
                    )?;

                    blocks.pop();

                    writeln!(function_buffer, "GOTO {}", then_name,).unwrap();
                    writeln!(function_buffer, "BLOCK {}", then_name,).unwrap();
                    //writeln!(function_buffer, "}}").unwrap();
                }
                OP_IF_AND_ELSE(ty, code1, code2) => {
                    debug!("Block ty is {:?}", ty);
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
                        function_buffer,
                        "IF %{} THEN GOTO {} ELSE GOTO {}",
                        self.symbol_table.peek()?,
                        name.clone(),
                        then_name.clone()
                    )
                    .unwrap();
                    writeln!(function_buffer, "BLOCK {} ", name.clone()).unwrap();

                    // If the block exits, then kill all variable to `current_reg`
                    let current_reg = self.symbol_table.peek().ok();

                    let arity = engine.get_return_count_block(ty)?;
                    self.visit_instruction_wrapper(
                        code1.get_instructions(),
                        function_index,
                        blocks,
                        arity as usize,
                        current_reg,
                        engine,
                        function_buffer,
                        params_count,
                    )?;

                    writeln!(function_buffer, "GOTO {}", done_name,).unwrap();
                    writeln!(function_buffer, "BLOCK {}", then_name,).unwrap();

                    blocks.pop();

                    blocks.push(tblock);

                    self.visit_instruction_wrapper(
                        code2.get_instructions(),
                        function_index,
                        blocks,
                        arity as usize,
                        current_reg,
                        engine,
                        function_buffer,
                        params_count,
                    )?;

                    blocks.pop();

                    writeln!(function_buffer, "GOTO {}", done_name,).unwrap();

                    writeln!(function_buffer, "BLOCK {}", done_name,).unwrap();
                }
                OP_BR(label) => {
                    let jmp_index = blocks_len - 1 - *label as usize;

                    let block = blocks.get(jmp_index).unwrap();

                    self.exit_block(return_arity, current_reg, function_buffer, params_count)?;

                    if block.is_loop {
                        writeln!(function_buffer, "GOTO {}", block.name).unwrap();
                    } else {
                        writeln!(function_buffer, "GOTO {}", block.name + 1).unwrap();
                    }
                }
                OP_BR_IF(label) => {
                    let jmp_index = blocks_len - 1 - *label as usize;

                    let block = blocks.get(jmp_index).unwrap();

                    if block.is_loop {
                        writeln!(
                            function_buffer,
                            "IF %{} THEN GOTO {}",
                            self.symbol_table.peek()?,
                            block.name
                        )
                        .unwrap();
                    } else {
                        writeln!(
                            function_buffer,
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
                        function_buffer,
                        "TABLE GOTO {} ELSE GOTO {}",
                        indices.join(" "),
                        jmp_index
                    )
                    .unwrap();
                }
                OP_LOCAL_GET(index) => {
                    let locals = &self
                        .functions
                        .get(function_index)
                        .with_context(|| format!("Cannot find function at {}", function_index))?
                        .locals;

                    writeln!(
                        function_buffer,
                        "%{} = %{}",
                        self.symbol_table.new_var()?,
                        locals.get(&(*index as usize)).unwrap()
                    )
                    .unwrap();
                }
                OP_LOCAL_SET(index) => {
                    let locals = &self
                        .functions
                        .get(function_index)
                        .with_context(|| format!("Cannot find function at {}", function_index))?
                        .locals;

                    debug!("locals {:?}", locals);

                    writeln!(
                        function_buffer,
                        "%{} = %{}",
                        locals.get(&(*index as usize)).unwrap(),
                        self.symbol_table.peek()?
                    )
                    .unwrap();
                }
                OP_LOCAL_TEE(index) => {
                    let peek = self.symbol_table.peek()?;
                    // Push only once because the old still lives
                    writeln!(
                        function_buffer,
                        "%{} = %{}",
                        self.symbol_table.new_var()?,
                        peek
                    )
                    .unwrap();
                    let locals = &self
                        .functions
                        .get(function_index)
                        .with_context(|| format!("Cannot find function at {}", function_index))?
                        .locals;

                    debug!("locals {:?}", locals);

                    writeln!(
                        function_buffer,
                        "%{} = %{}",
                        locals.get(&(*index as usize)).with_context(|| format!(
                            "Cannot local at {} when locals length is {}",
                            index,
                            locals.values().count()
                        ))?,
                        peek
                    )
                    .unwrap();
                }
                OP_I32_STORE(arg) | OP_I64_STORE(arg) | OP_F32_STORE(arg) | OP_F64_STORE(arg)
                | OP_I32_STORE_8(arg) | OP_I32_STORE_16(arg) | OP_I64_STORE_8(arg)
                | OP_I64_STORE_16(arg) | OP_I64_STORE_32(arg) => {
                    debug!("Ignoring memory store {:?}", arg);
                }
                OP_MEMORY_SIZE | OP_MEMORY_GROW => {
                    debug!("Ignoring memory");
                }
                OP_CALL(func) => {
                    let addr = engine.module.lookup_function_addr(*func)?;
                    let instance = engine.store.get_func_instance(&addr)?;

                    let num_params = instance.ty.param_types.len();
                    let num_results = instance.ty.return_types.len();

                    let mut param_regs = Vec::new();

                    for i in 0..num_params {
                        param_regs.push(format!("{}", self.symbol_table.peek_offset(i)?));
                    }

                    // Function returns no variables
                    if num_results == 0 {
                        writeln!(
                            function_buffer,
                            "CALL {}({})",
                            func,
                            param_regs
                                .into_iter()
                                .map(|x| format!("%{}", x))
                                .collect::<Vec<_>>()
                                .join(" ")
                        )
                        .unwrap();
                    } else {
                        let return_regs: Vec<_> = (0..num_results)
                            .map(|_| {
                                format!(
                                    "%{}",
                                    self.symbol_table.new_var().expect("Cannot get new var")
                                )
                            })
                            .collect();

                        writeln!(
                            function_buffer,
                            "{} <- CALL {}({})",
                            return_regs.join(" "),
                            func,
                            param_regs
                                .into_iter()
                                .map(|x| format!("%{}", x))
                                .collect::<Vec<_>>()
                                .join(" ")
                        )
                        .unwrap();
                    }
                }
                OP_I32_CONST(a) => {
                    writeln!(function_buffer, "%{} = {}", self.symbol_table.new_var()?, a).unwrap();
                }
                OP_I64_CONST(a) => {
                    writeln!(function_buffer, "%{} = {}", self.symbol_table.new_var()?, a).unwrap();
                }
                OP_F32_CONST(a) => {
                    writeln!(function_buffer, "%{} = {}", self.symbol_table.new_var()?, a).unwrap();
                }
                OP_F64_CONST(a) => {
                    writeln!(function_buffer, "%{} = {}", self.symbol_table.new_var()?, a).unwrap();
                }
                OP_RETURN => {
                    let function_return_arity = self
                        .functions
                        .get(function_index)
                        .context("Cannot find function")?
                        .return_count;

                    debug!("RETURN {}", function_return_arity);
                    let mut regs = Vec::new();

                    self.exit_block(return_arity, current_reg, function_buffer, params_count)?;

                    for i in 0..function_return_arity {
                        regs.push(format!("%{}", self.symbol_table.peek_offset(i)?));
                    }

                    writeln!(function_buffer, "RETURN {};", regs.join(" ")).unwrap();
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
                        function_buffer,
                        "%{} = {} %{}",
                        self.symbol_table.new_var()?,
                        "op",
                        self.symbol_table.peek_offset(2)?, // TODO Check why `2`
                    )
                    .unwrap();
                }
                OP_SELECT | OP_I32_ADD | OP_I32_SUB | OP_I32_MUL | OP_I32_DIV_S | OP_I32_DIV_U
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
                        function_buffer,
                        "%{} = %{} {} %{}",
                        self.symbol_table.new_var()?,
                        self.symbol_table.peek_offset(1)?,
                        "op",
                        self.symbol_table.peek_offset(2)?
                    )
                    .unwrap();
                }
                _ => {
                    writeln!(function_buffer, "{}", instr.get_instruction()).unwrap();
                }
            }
        }

        Ok(())
    }
}