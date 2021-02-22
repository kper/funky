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
    blocks: HashMap<String, Block>,
}

#[derive(Debug)]
struct Block {
    name: String,
    is_loop: bool,
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

        let function = Function {
            name,
            blocks: HashMap::new(),
        };

        self.functions.push(function);
        let func_index = self.functions.len() - 1;

        debug!("code {:#?}", inst.code);
        self.visit_body(&inst.code, func_index);

        writeln!(self.buffer, "}};").unwrap();
    }

    fn visit_body(&mut self, body: &FunctionBody, function_index: usize) {
        let code = &body.code;

        let name = format!("{}", self.block_counter.get());
        let block = Block {
            name: name.clone(),
            is_loop: false,
        };

        writeln!(self.buffer, "BLOCK {}", name).unwrap();

        self.visit_instruction_wrapper(code, function_index);

    }

    /*
    fn push_block(&mut self, function_index: usize, block: Block) -> usize {
        let function = self.functions.get_mut(function_index).unwrap();
        function.blocks.push(block);

        debug!("New block index's is {}", function.blocks.len() - 1);

        let index = function.blocks.len() - 1;

        function.blocks.push(Block {
            name: format!("{} // THEN BLOCK", self.block_counter.get()),
            instructions: Vec::new(),
            is_loop: false,
        });

        index
    }

    fn goto_next(&mut self, function_index: usize, block_index: usize) {
        debug!("GOTO next block {}", block_index + 1);

        let function = self.functions.get_mut(function_index).unwrap();
        function.blocks[block_index]
            .instructions
            .push(format!("GOTO {} // BLOCK ENDED for {}", block_index + 1, block_index));
    }

    fn push_instr(&mut self, function_index: usize, block_index: usize, instr: String) {
        let function = self.functions.get_mut(function_index).unwrap();
        function.blocks[block_index]
            .instructions
            .push(format!("{}", instr));
    }

    fn br(&mut self, function_index: usize, block_index: usize, label: usize) {
        let function = self.functions.get_mut(function_index).unwrap();

        let jmp_index = function.blocks.len() - 1 - label;
        let block = &function.blocks[jmp_index];

        if block.is_loop {
            self.push_instr(
                function_index,
                block_index,
                format!("GOTO {} // REPEAT", jmp_index),
            );
        } else {
            self.push_instr(
                function_index,
                block_index,
                format!("GOTO {} // BREAK", jmp_index + 1),
            );
        }
    }*/

    fn visit_instruction_wrapper(
        &mut self,
        code: &[InstructionWrapper],
        function_index: usize,
    ) {
        debug!("Visiting instruction wrapper");

        //let mut str_block = String::new();
        //writeln!(str_block, "BLOCK {}", self.block_counter.get()).unwrap();

        for instr in code.iter() {
            debug!("Instruction {}", instr.get_instruction());

            match instr.get_instruction() {
                OP_BLOCK(_ty, code) => {
                    let name = format!("{}", self.block_counter.get());
                    let then_name = format!("{}", self.block_counter.get());

                    let block = Block {
                        name: name.clone(),
                        is_loop: false,

                    };

                    let tblock = Block {
                        name: then_name.clone(),
                        is_loop: false,
                    };

                    self.functions[function_index].blocks.insert(name.clone(), block);
                    self.functions[function_index].blocks.insert(then_name.clone(), tblock);


                    writeln!(self.buffer, "BLOCK {}", name.clone()).unwrap();

                    self.visit_instruction_wrapper(
                        code.get_instructions(),
                        function_index,
                    );

                    writeln!(self.buffer, "GOTO {} // BLOCK ended for {}", then_name, name.clone()).unwrap();
                    writeln!(self.buffer, "BLOCK {} // THEN block for {}", then_name, name).unwrap();

                    /*
                    write!(
                        str_block,
                        "{}",
                        self.visit_instruction_wrapper(block.get_instructions())
                    )
                    .unwrap();
                    writeln!(str_block, "GOTO {}", self.block_counter.peek()).unwrap();*/
                }
                OP_LOOP(_ty, code) => {
                    let name = format!("{} // LOOP", self.block_counter.get());
                    let then_name = format!("{}", self.block_counter.get());

                    let block = Block {
                        name: name.clone(),
                        is_loop: true,

                    };

                    let tblock = Block {
                        name: then_name.clone(),
                        is_loop: false,
                    };

                    self.functions[function_index].blocks.insert(name.clone(), block);
                    self.functions[function_index].blocks.insert(then_name.clone(), tblock);


                    writeln!(self.buffer, "BLOCK {}", name.clone()).unwrap();

                    self.visit_instruction_wrapper(
                        code.get_instructions(),
                        function_index,
                    );

                    writeln!(self.buffer, "GOTO {} // BLOCK ended for {}", then_name, name.clone()).unwrap();
                    writeln!(self.buffer, "BLOCK {} // THEN block for {}", then_name, name).unwrap();


                    /*
                    write!(
                        str_block,
                        "{}",
                        self.visit_instruction_wrapper(block.get_instructions())
                    )
                    .unwrap();
                    writeln!(str_block, "GOTO {}", self.block_counter.peek()).unwrap();*/
                }
                OP_IF(_ty, code) => {
                    /*
                    writeln!(
                        str_block,
                        "if %{} THEN GOTO {}",
                        self.counter.peek(),
                        self.block_counter.peek_next()
                    )
                    .unwrap();
                    write!(
                        str_block,
                        "{}",
                        self.visit_instruction_wrapper(block.get_instructions())
                    )
                    .unwrap();
                    writeln!(str_block, "GOTO {}", self.block_counter.peek()).unwrap();*/
                }
                OP_IF_AND_ELSE(_ty, code1, code2) => {
                    /*
                    writeln!(
                        str_block,
                        "if %{} THEN GOTO {} ELSE GOTO {}",
                        self.counter.peek(),
                        self.block_counter.peek(),
                        self.block_counter.peek_next() + 1
                    )
                    .unwrap();
                    write!(
                        str_block,
                        "{}",
                        self.visit_instruction_wrapper(code1.get_instructions())
                    )
                    .unwrap();
                    writeln!(str_block, "GOTO {}", self.block_counter.peek_next()).unwrap();

                    write!(
                        str_block,
                        "{}",
                        self.visit_instruction_wrapper(code2.get_instructions())
                    )
                    .unwrap();
                    writeln!(str_block, "GOTO {}", self.block_counter.peek()).unwrap();*/
                }
                OP_BR(label) => {
                    /*
                    writeln!(
                        str_block,
                        "GOTO {}",
                        self.block_counter.peek() + 1 - *label as usize
                    )
                    .unwrap();*/
                }
                OP_BR_IF(label) => {
                   
                    /*
                    writeln!(
                        str_block,
                        "if %{} THEN GOTO {}",
                        self.counter.peek(),
                        self.block_counter.peek() + 1 - *label as usize
                    )
                    .unwrap();*/
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
