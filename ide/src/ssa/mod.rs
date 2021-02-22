#![allow(dead_code)]

use funky::engine::store::Store;
use wasm_parser::core::*;
use funky::engine::func::FuncInstance;
use std::fmt::Write;

#[derive(Debug, Default)]
pub struct IR {
    buffer: String,
    counter: Counter,
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
    pub fn buffer(&self) -> &str {
        &self.buffer
    }

    pub fn visit(&mut self, store: &Store) {
        for func in store.funcs.iter() {
            self.visit_function(func);
        }
    }

    fn visit_function(&mut self, inst: &FuncInstance) {
        writeln!(self.buffer, "define {} {{", self.counter.get());

        self.visit_body(&inst.code);

        writeln!(self.buffer, "}};");
    }

    fn visit_body(&mut self, body: &FunctionBody) {
        let code = &body.code;

        for instr in code.iter() {
            writeln!(self.buffer, "{}", instr.get_instruction());
        }
    }
}