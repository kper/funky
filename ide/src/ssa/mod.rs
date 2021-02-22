#![allow(dead_code)]

use funky::engine::func::FuncInstance;
use funky::engine::store::Store;
use std::fmt::Write;
use wasm_parser::core::Instruction::*;
use wasm_parser::core::*;

#[derive(Debug, Default)]
pub struct IR {
    //buffer: String,
    functions: Vec<Function>,
    counter: Counter,
}

#[derive(Debug)]
struct Function {
    blocks: Vec<Block>,
}

#[derive(Debug)]
struct Block {
    name: String,
    instructions: Vec<String>,
}

#[derive(Debug, Default)]
struct Counter {
    counter: usize,
}

impl Counter {
    pub fn get(&mut self) -> usize {
        let counter = self.counter.clone();
        self.counter += 1;
        counter
    }
}

impl IR {
    pub fn buffer(&self) -> String {
        format!("{:#?}", self.functions)
        //&self.buffer
    }

    pub fn visit(&mut self, store: &Store) {
        

        for function in store.funcs.iter() {
            let code = self.get_instructions(&function.code.code);

            for instr in code {

            } 

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
        //writeln!(self.buffer, "define {} {{", self.counter.get()).unwrap();
        let blocks = self.visit_body(&inst.code);

        self.functions.push(Function { blocks });

        //writeln!(self.buffer, "}};").unwrap();
    }

    fn visit_body(&mut self, body: &FunctionBody) -> Vec<Block> {
        let code = &body.code;

        self.visit_instruction_wrapper(code)
    }

    fn visit_instruction_wrapper(&mut self, code: &[InstructionWrapper]) -> Vec<Block> {
        let mut blocks = Vec::new();

        let main_block = Block {
            name: format!("{}", self.counter.get()),
            instructions: Vec::new(),
        };

        blocks.push(main_block);

        let main_index = blocks.len() - 1;

        for instr in code.iter() {
            match instr.get_instruction() {
                OP_BLOCK(_ty, block) => {
                    let mut inner_blocks = self.visit_instruction_wrapper(block.get_instructions());
                    blocks.append(&mut inner_blocks);
                }
                OP_LOOP(_ty, block) => {
                    let mut inner_blocks = self.visit_instruction_wrapper(block.get_instructions());
                    blocks.append(&mut inner_blocks);
                }
                _ => {
                    let main_block = blocks.get_mut(main_index).unwrap();

                    main_block
                        .instructions
                        .push(format!("{}", instr.get_instruction()))
                    //writeln!(self.buffer, "{}", instr.get_instruction()).unwrap();
                }
            }
        }

        blocks
    }
}
