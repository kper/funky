use crate::engine::StackContent::*;
use crate::engine::Value::*;
use wasm_parser::isa::Instruction::*;

use std::collections::HashMap;
use wasm_parser::isa::Instruction;

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

#[derive(Debug)]
enum Trap {}

#[derive(Debug)]
enum Result {
    None,
    Value(Value),
    Trap(Trap),
}

#[derive(Debug, PartialEq)]
enum StackContent {
    Value(Value),
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
    ($stack: expr, $v_ty: ident, $n_ty: ident) => {{
        let v1: $n_ty = match $stack.pop().unwrap() {
            Value($v_ty(v)) => v.into(),
            x => panic!("Top of stack was not of type $v_ty: {:?}", x),
        };
        let v2: $n_ty = match $stack.pop().unwrap() {
            Value($v_ty(v)) => v.into(),
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
                I32Const(v) => self.store.stack.push(Value(I32(*v))),
                I64Const(v) => self.store.stack.push(Value(I64(*v))),
                I32Add => {
                    let (v1, v2) = fetch_binop!(self.store.stack, I32, i32);
                    self.store.stack.push(Value(I32(v1 + v2)))
                }
                I64Add => {
                    let (v1, v2) = fetch_binop!(self.store.stack, I64, i64);
                    self.store.stack.push(Value(I64(v1 + v2)))
                }
                I32Mul => {
                    let (v1, v2) = fetch_binop!(self.store.stack, I32, i32);
                    self.store.stack.push(Value(I32(v1 * v2)))
                }
                I64Mul => {
                    let (v1, v2) = fetch_binop!(self.store.stack, I64, i64);
                    self.store.stack.push(Value(I64(v1 * v2)))
                }
                End => return,
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
                source: vec![I32Const(42), I32Const(42), I32Add, End],
            },
            store: Store {
                stack: Vec::new(),
                globals: HashMap::new(),
                memory: Vec::new(),
            },
        };
        e.run_function(0);
        assert_eq!(Value(I32(84)), e.store.stack.pop().unwrap());
        e.module.source = vec![I64Const(32), I64Const(32), I64Add, I64Const(2), I64Mul, End];
        e.store.stack = Vec::new();
        e.run_function(0);
        assert_eq!(Value(I64(128)), e.store.stack.pop().unwrap());
    }
}
