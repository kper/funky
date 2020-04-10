use crate::engine::StackContent::*;
use crate::engine::Value::*;
use std::ops::{Add, Mul};
use wasm_parser::core::CtrlInstructions::*;
use wasm_parser::core::Instruction::*;
use wasm_parser::core::NumericInstructions::*;

use std::collections::HashMap;
use wasm_parser::core::Instruction;

#[derive(Debug)]
struct Engine {
    module: ModuleInstance,
    store: Store,
    started: bool,
}

#[derive(Debug, PartialEq)]
pub enum Value {
    I32(i32),
    I64(i64),
    F32(f32),
    F64(f64),
}

impl Add for Value {
    type Output = Self;
    fn add(self, other: Self) -> Self {
        match (self, other) {
            (I32(v1), I32(v2)) => I32(v1 + v2),
            (I64(v1), I64(v2)) => I64(v1 + v2),
            (F32(v1), F32(v2)) => F32(v1 + v2),
            (F64(v1), F64(v2)) => F64(v1 + v2),
            _ => panic!("Type missmatch during addition"),
        }
    }
}
impl Mul for Value {
    type Output = Self;
    fn mul(self, other: Self) -> Self {
        match (self, other) {
            (I32(v1), I32(v2)) => I32(v1 * v2),
            (I64(v1), I64(v2)) => I64(v1 * v2),
            (F32(v1), F32(v2)) => F32(v1 * v2),
            (F64(v1), F64(v2)) => F64(v1 * v2),
            _ => panic!("Type missmatch during addition"),
        }
    }
}

#[derive(Debug)]
pub enum Variable {
    I32(i32),
    I32M(i32),

    I64(i64),
    I64M(i64),

    F32(f32),
    F32M(f32),

    F64(f64),
    F64M(f64),
}

#[derive(Debug, PartialEq)]
enum StackContent {
    Value(Value),
    Frame,
}

#[derive(Debug)]
struct ModuleInstance {
    source: Vec<Instruction>,
}

#[derive(Debug)]
struct Store {
    globals: HashMap<u32, Variable>,
    memory: Vec<u8>,
    stack: Vec<StackContent>,
}

macro_rules! fetch_binop {
    ($stack: expr) => {{
        let v1 = match $stack.pop().unwrap() {
            Value(v) => v,
            x => panic!("Top of stack was not of type $v_ty: {:?}", x),
        };
        let v2 = match $stack.pop().unwrap() {
            Value(v) => v,
            x => panic!("Top of stack was not of type $v_ty: {:?}", x),
        };
        (v1, v2)
    }};
}

impl Engine {
    pub fn run_function(&mut self, addr: usize) {
        let mut ip = addr;
        loop {
            match &self.module.source[ip] {
                Num(OP_I32_CONST(v)) => self.store.stack.push(Value(I32(*v))),
                Num(OP_I64_CONST(v)) => self.store.stack.push(Value(I64(*v))),
                Num(OP_I32_ADD) | Num(OP_I64_ADD) => {
                    let (v1, v2) = fetch_binop!(self.store.stack);
                    self.store.stack.push(Value(v1 + v2))
                }
                Num(OP_I32_MUL) | Num(OP_I64_MUL) => {
                    let (v1, v2) = fetch_binop!(self.store.stack);
                    self.store.stack.push(Value(v1 * v2))
                }
                Ctrl(OP_END) => return,
                x => panic!("Instruction {:?} not implemented", x),
            }
            ip += 1;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_run_function() {
        let mut e = Engine {
            started: true,
            module: ModuleInstance {
                source: vec![
                    Num(OP_I32_CONST(42)),
                    Num(OP_I32_CONST(42)),
                    Num(OP_I32_ADD),
                    Ctrl(OP_END),
                ],
            },
            store: Store {
                stack: Vec::new(),
                globals: HashMap::new(),
                memory: Vec::new(),
            },
        };
        e.run_function(0);
        assert_eq!(Value(I32(84)), e.store.stack.pop().unwrap());
        e.module.source = vec![
            Num(OP_I64_CONST(32)),
            Num(OP_I64_CONST(32)),
            Num(OP_I64_ADD),
            Num(OP_I64_CONST(2)),
            Num(OP_I64_MUL),
            Ctrl(OP_END),
        ];
        e.store.stack = Vec::new();
        e.run_function(0);
        assert_eq!(Value(I64(128)), e.store.stack.pop().unwrap());
    }
}
