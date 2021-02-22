#![allow(dead_code)]

use funky::engine::func::{self, FuncInstance};
use funky::engine::store::Store;
use log::debug;
use std::fmt::Write;
use wasm_parser::core::Instruction::*;
use wasm_parser::core::*;

#[derive(Debug, Default)]
pub struct IR {
    //functions: Vec<Function>,
    counter: Counter,
    block_counter: Counter,
    function_counter: Counter,
    functions: Vec<Function>,
}

#[derive(Debug)]
struct Function {
    name: String,
    blocks: Vec<Block>,
}

#[derive(Debug)]
struct Block {
    name: String,
    instructions: Vec<String>,
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
        }

        buffer
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
        //writeln!(self.buffer, "define {} {{", name).unwrap();

        let function = Function {
            name,
            blocks: Vec::default(),
        };

        self.functions.push(function);
        let func_index = self.functions.len() - 1;

        debug!("code {:#?}", inst.code);
        self.visit_body(&inst.code, func_index);

        //writeln!(self.buffer, "}};").unwrap();
    }

    fn visit_body(&mut self, body: &FunctionBody, function_index: usize) {
        let code = &body.code;

        let block_index = 0;

        let block = Block {
            name: format!("{}", self.block_counter.get()),
            instructions: Vec::new(),
            is_loop: false,
        };

        let function = self.functions.get_mut(function_index).unwrap();
        function.blocks.push(block);

        let blk = self.visit_instruction_wrapper(code, function_index, block_index);

        //writeln!(self.buffer, "{}", blk).unwrap();
    }

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
            .push(format!("GOTO {} // BLOCK ENDED", block_index + 1));
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
    }

    fn visit_instruction_wrapper(
        &mut self,
        code: &[InstructionWrapper],
        function_index: usize,
        block_index: usize,
    ) {
        debug!("Visiting instruction wrapper");

        //let mut str_block = String::new();
        //writeln!(str_block, "BLOCK {}", self.block_counter.get()).unwrap();

        for instr in code.iter() {
            debug!("Instruction {}", instr.get_instruction());

            match instr.get_instruction() {
                OP_BLOCK(_ty, code) => {
                    let block = Block {
                        name: format!("{}", self.block_counter.get()),
                        instructions: Vec::new(),
                        is_loop: false,
                    };

                    let block_index = self.push_block(function_index, block);

                    self.visit_instruction_wrapper(
                        code.get_instructions(),
                        function_index,
                        block_index,
                    );

                    self.goto_next(function_index, block_index);

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
                    let block = Block {
                        name: format!("Loop{}", self.block_counter.get()),
                        instructions: Vec::new(),
                        is_loop: true,
                    };

                    let block_index = self.push_block(function_index, block);

                    self.visit_instruction_wrapper(
                        code.get_instructions(),
                        function_index,
                        block_index,
                    );

                    self.goto_next(function_index, block_index);

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
                    self.br(function_index, block_index, *label as usize);
                    /*
                    writeln!(
                        str_block,
                        "GOTO {}",
                        self.block_counter.peek() + 1 - *label as usize
                    )
                    .unwrap();*/
                }
                OP_BR_IF(label) => {
                    let function = self.functions.get_mut(function_index).unwrap();

                    let jmp_index = function.blocks.len() - 1 - *label as usize;
                    let block = &function.blocks[jmp_index];

                    if block.is_loop {
                        self.push_instr(
                            function_index,
                            block_index,
                            format!(
                                "IF %{} THEN GOTO {} // REPEAT",
                                self.counter.peek(),
                                jmp_index
                            ),
                        );
                    } else {
                        self.push_instr(
                            function_index,
                            block_index,
                            format!(
                                "IF %{} THEN GOTO {} // BREAK",
                                self.counter.peek(),
                                jmp_index + 1
                            ),
                        );
                    }
                    /*
                    writeln!(
                        str_block,
                        "if %{} THEN GOTO {}",
                        self.counter.peek(),
                        self.block_counter.peek() + 1 - *label as usize
                    )
                    .unwrap();*/
                }
                OP_BR_TABLE(labels, else_lb) => {
                    let function = self.functions.get_mut(function_index).unwrap();

                    let jmp_index: Vec<_> = labels
                        .iter()
                        .map(|label| {
                            let jmp = function.blocks.len() - 1 - *label as usize;

                            if function.blocks[jmp].is_loop {
                                jmp
                            } else {
                                jmp + 1
                            }
                        })
                        .collect();

                    let else_index = function.blocks.len() - 1 - *else_lb as usize;
                    let else_block = &function.blocks[else_index];

                    let else_index = match else_block.is_loop {
                        true => else_index,
                        false => else_index + 1,
                    };

                    self.push_instr(
                        function_index,
                        block_index,
                        format!(
                            "BR TABLE GOTO %{} ELSE GOTO {}",
                            jmp_index
                                .into_iter()
                                .map(|x| format!("{}", x))
                                .collect::<Vec<_>>()
                                .join(" "),
                            else_index
                        ),
                    );

                    /*
                        debug!("table labels {:?} else {:?}", labels, else_lb);
                        write!(
                            str_block,
                            "GOTO table "
                        )
                        .unwrap();

                        for lb in labels {
                            write!(str_block, "{} ", self.block_counter.peek() - *lb as usize).unwrap();
                        }

                        write!(
                            str_block,
                            "ELSE GOTO {}", self.block_counter.peek() - *else_lb as usize
                        ).unwrap();

                        writeln!(str_block, "");

                    */
                }
                _ => {
                    self.push_instr(
                        function_index,
                        block_index,
                        format!("{}", instr.get_instruction()),
                    );
                    //writeln!(str_block, "{}", instr.get_instruction()).unwrap();
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
