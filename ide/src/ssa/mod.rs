#![allow(dead_code)]

use funky::engine::func::FuncInstance;
use funky::engine::store::Store;
use std::fmt::Write;
use wasm_parser::core::Instruction::*;
use wasm_parser::core::*;
use log::debug;

#[derive(Debug, Default)]
pub struct IR {
    //functions: Vec<Function>,
    buffer: String,
    counter: Counter,
    block_counter: Counter,
    function_counter: Counter,
}

#[derive(Debug)]
struct Function {
    name: String,
    blocks: Vec<Block>
}

#[derive(Debug)]
struct Block {
    name: String,
    instructions: Vec<String>
}



#[derive(Debug, Default)]
struct Counter {
    counter: usize,
}

impl Counter {
    pub fn peek(& self) -> usize {
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
    pub fn buffer(&self) -> &str {
        //format!("{:#?}", self.functions)
        &self.buffer
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
        writeln!(self.buffer, "define {} {{", self.function_counter.get()).unwrap();

        debug!("code {:#?}", inst.code);
        self.visit_body(&inst.code);

        writeln!(self.buffer, "}};").unwrap();
    }

    fn visit_body(&mut self, body: &FunctionBody) {
        let code = &body.code;

        //let name= format!("{}", self.block_counter.get());

        //writeln!(self.buffer, "BLOCK {}", name).unwrap();


        let blk = self.visit_instruction_wrapper(code);

        writeln!(self.buffer, "{}", blk).unwrap();
    }

    fn visit_instruction_wrapper(&mut self, code: &[InstructionWrapper]) -> String {
        debug!("Visiting instruction wrapper");

        let mut str_block = String::new();
        writeln!(str_block, "BLOCK {}", self.block_counter.get()).unwrap();
        let mut jumped = false;

        for instr in code.iter() {
            debug!("Instruction {}", instr.get_instruction());

            match instr.get_instruction() {
                OP_BLOCK(_ty, block) => {
                    write!(str_block, "{}" ,self.visit_instruction_wrapper(block.get_instructions()));
                }
                OP_LOOP(_ty, block) => {
                    write!(str_block, "{}", self.visit_instruction_wrapper(block.get_instructions()));
                    writeln!(str_block, "GOTO {}", self.block_counter.peek() + 1).unwrap();
                    jumped = true;
                }
                OP_IF(_ty, block) => {
                    writeln!(str_block, "if %{} THEN GOTO {}", self.counter.peek(), self.block_counter.peek_next()).unwrap();

                    write!(str_block, "{}", self.visit_instruction_wrapper(block.get_instructions()));
                }
                OP_BR(label) => {
                    writeln!(str_block, "GOTO {}", self.block_counter.peek() - *label as usize).unwrap();
                    jumped = true;
                }
                _ => {
                    writeln!(str_block, "{}", instr.get_instruction()).unwrap();
                }
            }
        }

        if !jumped {
            writeln!(str_block, "GOTO {}", self.block_counter.peek()).unwrap();
        }

        writeln!(str_block, "BLOCK {}", self.block_counter.get()).unwrap();

        str_block

        //writeln!(self.buffer, "{}", str_block).unwrap();
    }
}
