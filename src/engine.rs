use crate::engine::StackContent::*;
use crate::engine::Value::*;
use std::ops::{Add, Mul};
use wasm_parser::core::CtrlInstructions::*;
use wasm_parser::core::FunctionBody;
use wasm_parser::core::Instruction::*;
use wasm_parser::core::NumericInstructions::*;
use wasm_parser::core::ValueType;
use wasm_parser::core::VarInstructions::*;

use std::collections::HashMap;
use wasm_parser::core::Instruction;

#[derive(Debug)]
struct Engine {
    module: ModuleInstance,
    store: Store,
    started: bool,
}

#[derive(Debug, PartialEq, Clone, Copy)]
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
    Frame(Frame),
}

#[derive(Debug, PartialEq)]
struct Frame {
    locals: Vec<Value>,
}

#[derive(Debug)]
struct ModuleInstance {}

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
    pub fn run_function(&mut self, f: FunctionBody) {
        let fr = match self.store.stack.pop() {
            Some(Frame(fr)) => fr,
            Some(x) => panic!("Expected frame but found {:?}", x),
            None => panic!("Empty stack on function call"),
        };
        let mut ip = 0;
        loop {
            if ip >= f.code.0.len() {
                return;
            }
            match &f.code.0[ip] {
                Var(OP_LOCAL_GET(idx)) => self.store.stack.push(Value(fr.locals[*idx as usize])),
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
    use wasm_parser::core::Expr;

    fn empty_engine() -> Engine {
        Engine {
            started: true,
            module: ModuleInstance {},
            store: Store {
                stack: vec![Frame(Frame { locals: Vec::new() })],
                globals: HashMap::new(),
                memory: Vec::new(),
            },
        }
    }

    #[test]
    fn test_run_function() {
        let mut e = empty_engine();
        e.run_function(FunctionBody {
            locals: vec![],
            code: Expr(vec![
                Num(OP_I32_CONST(42)),
                Num(OP_I32_CONST(42)),
                Num(OP_I32_ADD),
            ]),
        });
        assert_eq!(Value(I32(84)), e.store.stack.pop().unwrap());
        e.store.stack = vec![Frame(Frame { locals: Vec::new() })];
        e.run_function(FunctionBody {
            locals: vec![],
            code: Expr(vec![
                Num(OP_I64_CONST(32)),
                Num(OP_I64_CONST(32)),
                Num(OP_I64_ADD),
                Num(OP_I64_CONST(2)),
                Num(OP_I64_MUL),
            ]),
        });
        assert_eq!(Value(I64(128)), e.store.stack.pop().unwrap());
    }

    #[test]
    fn test_function_with_params() {
        let mut e = empty_engine();
        e.store.stack = vec![Frame(Frame {
            locals: vec![I32(1), I32(4)],
        })];
        e.run_function(FunctionBody {
            locals: vec![],
            code: Expr(vec![
                Var(OP_LOCAL_GET(0)),
                Var(OP_LOCAL_GET(1)),
                Num(OP_I32_ADD),
            ]),
        });
        assert_eq!(Value(I32(5)), e.store.stack.pop().unwrap());
    }
}
