#![allow(dead_code)]

use funky::engine::func::{self, FuncInstance};
use funky::engine::store::Store;
use log::debug;
use std::fmt::Write;
use wasm_parser::core::Instruction::*;
use wasm_parser::core::*;

use std::collections::HashMap;

#[derive(Debug, Default)]
pub struct IR {
    //functions: Vec<Function>,
    buffer: String,
    counter: Counter,
    block_counter: Counter,
    function_counter: Counter,
    functions: Vec<Function>,
}

#[derive(Debug)]
struct Function {
    name: String,
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
    pub fn buffer(&self) -> String {
        //format!("{:#?}", self.functions)

        /*
         let mut buffer = String::new();
         for func in self.functions.iter() {
             writeln!(buffer, "define {} {{", func.name).unwrap();

             for block in func.blocks.iter() {
                 writeln!(buffer, "BLOCK {}", block.name).unwrap();

                 for instr in block.instructions.iter() {
                     writeln!(buffer, "{}", instr).unwrap();
                 }
             }

             writeln!(buffer, "}};").unwrap();
        }*/

        self.buffer.clone()
    }

    pub fn visit(&mut self, store: &Store) {
        for function in store.funcs.iter() {
            self.visit_function(function);
        }
    }

    fn get_instructions<'a>(
        &self,
        instructions: &'a [InstructionWrapper],
    ) -> Vec<&'a InstructionWrapper> {
        let mut result = Vec::new();

        for i in instructions {
            match i.get_instruction() {
                Instruction::OP_BLOCK(_, block) => {
                    result.push(i);
                    result.extend(&self.get_instructions(block.get_instructions()));
                }
                Instruction::OP_LOOP(_, block) => {
                    result.push(i);
                    result.extend(&self.get_instructions(block.get_instructions()));
                }
                Instruction::OP_IF(_, block) => {
                    result.push(i);
                    result.extend(&self.get_instructions(block.get_instructions()));
                }
                Instruction::OP_IF_AND_ELSE(_, block1, block2) => {
                    result.push(i);
                    result.extend(&self.get_instructions(block1.get_instructions()));
                    result.extend(&self.get_instructions(block2.get_instructions()));
                }
                _ => {
                    result.push(i);
                }
            }
        }

        result
    }

    fn visit_function(&mut self, inst: &FuncInstance) {
        let name = format!("{}", self.function_counter.get());
        writeln!(self.buffer, "define {} {{", name).unwrap();

        let function = Function { name };

        self.functions.push(function);
        let func_index = self.functions.len() - 1;

        debug!("code {:#?}", inst.code);
        self.visit_body(&inst.code, func_index);

        writeln!(self.buffer, "}};").unwrap();
    }

    fn visit_body(&mut self, body: &FunctionBody, function_index: usize) {
        let code = &body.code;

        let name = self.block_counter.get();
        let then_name = self.block_counter.get();

        let block = Block {
            name: name.clone(),
            is_loop: false,
        };

        let mut blocks = vec![block];

        writeln!(self.buffer, "BLOCK {}", name).unwrap();

        self.visit_instruction_wrapper(code, function_index, &mut blocks);

        writeln!(self.buffer, "BLOCK {}", then_name).unwrap();
    }

    fn visit_instruction_wrapper(
        &mut self,
        code: &[InstructionWrapper],
        function_index: usize,
        blocks: &mut Vec<Block>,
    ) {
        debug!("Visiting instruction wrapper");

        //let mut str_block = String::new();
        //writeln!(str_block, "BLOCK {}", self.block_counter.get()).unwrap();

        let blocks_len = blocks.len();

        for instr in code.iter() {
            debug!("Instruction {}", instr.get_instruction());

            match instr.get_instruction() {
                OP_BLOCK(_ty, code) => {
                    let name = self.block_counter.get();
                    let then_name = self.block_counter.get();

                    let block = Block {
                        name: name.clone(),
                        is_loop: false,
                    };

                    let tblock = Block {
                        name: then_name.clone(),
                        is_loop: false,
                    };

                    blocks.push(block);

                    writeln!(self.buffer, "BLOCK {}", name.clone()).unwrap();

                    self.visit_instruction_wrapper(code.get_instructions(), function_index, blocks);

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
                OP_LOOP(_ty, code) => {
                    let name = self.block_counter.get();
                    let then_name = self.block_counter.get();

                    let block = Block {
                        name: name.clone(),
                        is_loop: true,
                    };

                    let tblock = Block {
                        name: then_name.clone(),
                        is_loop: false,
                    };

                    blocks.push(block);

                    writeln!(self.buffer, "BLOCK {} // LOOP", name.clone()).unwrap();

                    self.visit_instruction_wrapper(code.get_instructions(), function_index, blocks);

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
                OP_IF(_ty, code) => {
                    let name = self.block_counter.get();
                    let then_name = self.block_counter.get();

                    let block = Block {
                        name: name.clone(),
                        is_loop: false,
                    };

                    let tblock = Block {
                        name: then_name.clone(),
                        is_loop: false,
                    };

                    blocks.push(block);

                    writeln!(
                        self.buffer,
                        "IF %{} THEN GOTO {} ELSE GOTO {}",
                        self.counter.peek(),
                        name.clone(),
                        then_name.clone()
                    )
                    .unwrap();
                    writeln!(self.buffer, "BLOCK {} ", name.clone()).unwrap();

                    self.visit_instruction_wrapper(code.get_instructions(), function_index, blocks);

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
                OP_IF_AND_ELSE(_ty, code1, code2) => {
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

                    let done_block = Block {
                        name: done_name.clone(),
                        is_loop: false,
                    };

                    blocks.push(block);

                    writeln!(
                        self.buffer,
                        "IF %{} THEN GOTO {} ELSE GOTO {}",
                        self.counter.peek(),
                        name.clone(),
                        then_name.clone()
                    )
                    .unwrap();
                    writeln!(self.buffer, "BLOCK {} ", name.clone()).unwrap();

                    self.visit_instruction_wrapper(
                        code1.get_instructions(),
                        function_index,
                        blocks,
                    );

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
                    );

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
                }
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
                            self.counter.peek(),
                            block.name
                        )
                        .unwrap();
                    } else {
                        writeln!(
                            self.buffer,
                            "IF %{} THEN GOTO {}",
                            self.counter.peek(),
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
                        "BR TABLE GOTO %{} ELSE GOTO {}",
                        indices.join(" "),
                        jmp_index
                    )
                    .unwrap();
                }
                OP_I32_CONST(a) => {
                    writeln!(self.buffer, "%{} = {}", self.counter.get(), a).unwrap();
                }
                OP_I64_CONST(a) => {
                    writeln!(self.buffer, "%{} = {}", self.counter.get(), a).unwrap();
                }
                OP_F32_CONST(a) => {
                    writeln!(self.buffer, "%{} = {}", self.counter.get(), a).unwrap();
                }
                OP_F64_CONST(a) => {
                    writeln!(self.buffer, "%{} = {}", self.counter.get(), a).unwrap();
                }
                _ => {
                    writeln!(self.buffer, "{}", instr.get_instruction()).unwrap();
                }
            }
        }

        /*
        if !jumped {
            writeln!(str_block, "GOTO {}", self.block_counter.peek()).unwrap();
        }*/

        //writeln!(str_block, "BLOCK {}", self.block_counter.get()).unwrap();

        //writeln!(self.buffer, "{}", str_block).unwrap();
    }
}
