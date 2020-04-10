use crate::engine::StackContent::*;
use crate::engine::Value::*;
use std::ops::{Add, Mul};
use wasm_parser::core::CtrlInstructions::*;
use wasm_parser::core::FunctionBody;
use wasm_parser::core::Instruction::*;
use wasm_parser::core::NumericInstructions::*;
use wasm_parser::core::Section;
use wasm_parser::core::VarInstructions::*;
use wasm_parser::Module;

use std::collections::HashMap;
use wasm_parser::core::Instruction;

#[derive(Debug)]
pub struct Engine {
    pub module: ModuleInstance,
    pub store: Store,
    pub started: bool,
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
pub struct Variable {
    mutable: bool,
    val: Value,
}

#[derive(Debug, PartialEq)]
pub enum StackContent {
    Value(Value),
    Frame(Frame),
}

#[derive(Debug, PartialEq)]
struct Frame {
    locals: Vec<Value>,
}

#[derive(Debug)]
pub struct ModuleInstance {
    start: u32,
    code: Vec<FunctionBody>,
}

#[derive(Debug)]
pub struct Store {
    pub globals: Vec<Variable>,
    pub memory: Vec<u8>,
    pub stack: Vec<StackContent>,
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

impl ModuleInstance {
    pub fn new(m: Module) -> Self {
        let mut mi = ModuleInstance {
            start: 0,
            code: Vec::new(),
        };
        for section in m.sections {
            match section {
                Section::Code { entries: x } => mi.code = x,
                _ => {}
            }
        }
        mi
    }
}

impl Engine {
    pub fn new(mi: ModuleInstance) -> Self {
        Engine {
            module: mi,
            started: false,
            store: Store {
                stack: Vec::new(),
                globals: Vec::new(),
                memory: Vec::new(),
            },
        }
    }
    pub fn invoke_function(&mut self, idx: u32, args: Vec<Value>) {
        self.store.stack.push(Frame(Frame { locals: args }));
        self.run_function(idx);
    }
    fn run_function(&mut self, idx: u32) {
        let f = &self.module.code[idx as usize];
        let mut fr = match self.store.stack.pop() {
            Some(Frame(fr)) => fr,
            Some(x) => panic!("Expected frame but found {:?}", x),
            None => panic!("Empty stack on function call"),
        };
        let mut ip = 0;
        loop {
            if ip >= f.code.len() {
                return;
            }
            match &f.code[ip] {
                Var(OP_LOCAL_GET(idx)) => self.store.stack.push(Value(fr.locals[*idx as usize])),
                Var(OP_LOCAL_SET(idx)) => match self.store.stack.pop() {
                    Some(Value(v)) => fr.locals[*idx as usize] = v,
                    Some(x) => panic!("Expected value but found {:?}", x),
                    None => panic!("Empty stack during local.set"),
                },
                Var(OP_LOCAL_TEE(idx)) => match self.store.stack.last() {
                    Some(Value(v)) => fr.locals[*idx as usize] = *v,
                    Some(x) => panic!("Expected value but found {:?}", x),
                    None => panic!("Empty stack during local.set"),
                },
                Var(OP_GLOBAL_GET(idx)) => self
                    .store
                    .stack
                    .push(Value(self.store.globals[*idx as usize].val)),
                Var(OP_GLOBAL_SET(idx)) => match self.store.stack.pop() {
                    Some(Value(v)) => {
                        if !self.store.globals[*idx as usize].mutable {
                            panic!("Attempting to modify a immutable global")
                        }
                        self.store.globals[*idx as usize].val = v
                    }
                    Some(x) => panic!("Expected value but found {:?}", x),
                    None => panic!("Empty stack during local.set"),
                },
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
                Ctrl(OP_NOP) => {}
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

    fn empty_engine() -> Engine {
        Engine {
            started: true,
            module: ModuleInstance {
                start: 0,
                code: Vec::new(),
            },
            store: Store {
                stack: vec![Frame(Frame { locals: Vec::new() })],
                globals: Vec::new(),
                memory: Vec::new(),
            },
        }
    }

    #[test]
    fn test_run_function() {
        let mut e = empty_engine();
        e.module.code = vec![FunctionBody {
            locals: vec![],
            code: vec![
                Num(OP_I32_CONST(42)),
                Num(OP_I32_CONST(42)),
                Num(OP_I32_ADD),
            ],
        }];
        e.run_function(0);
        assert_eq!(Value(I32(84)), e.store.stack.pop().unwrap());
        e.store.stack = vec![Frame(Frame { locals: Vec::new() })];
        e.module.code = vec![FunctionBody {
            locals: vec![],
            code: vec![
                Num(OP_I64_CONST(32)),
                Num(OP_I64_CONST(32)),
                Num(OP_I64_ADD),
                Num(OP_I64_CONST(2)),
                Num(OP_I64_MUL),
            ],
        }];
        e.run_function(0);
        assert_eq!(Value(I64(128)), e.store.stack.pop().unwrap());
    }

    #[test]
    fn test_function_with_params() {
        let mut e = empty_engine();
        e.store.stack = vec![Frame(Frame {
            locals: vec![I32(1), I32(4)],
        })];
        e.module.code = vec![FunctionBody {
            locals: vec![],
            code: vec![Var(OP_LOCAL_GET(0)), Var(OP_LOCAL_GET(1)), Num(OP_I32_ADD)],
        }];
        e.run_function(0);
        assert_eq!(Value(I32(5)), e.store.stack.pop().unwrap());
    }

    #[test]
    fn test_function_local_set() {
        let mut e = empty_engine();
        e.store.stack = vec![Frame(Frame {
            locals: vec![I32(1), I32(4)],
        })];
        e.module.code = vec![FunctionBody {
            locals: vec![],
            code: vec![
                Var(OP_LOCAL_GET(0)),
                Var(OP_LOCAL_GET(1)),
                Num(OP_I32_ADD),
                Var(OP_LOCAL_SET(0)),
                Num(OP_I32_CONST(32)),
                Var(OP_LOCAL_GET(0)),
                Num(OP_I32_ADD),
            ],
        }];
        e.run_function(0);
        assert_eq!(Value(I32(37)), e.store.stack.pop().unwrap());
    }

    #[test]
    fn test_function_globals() {
        let mut e = empty_engine();
        e.store.globals = vec![Variable {
            mutable: true,
            val: I32(69),
        }];
        e.module.code = vec![FunctionBody {
            locals: vec![],
            code: vec![
                Var(OP_GLOBAL_GET(0)),
                Num(OP_I32_CONST(351)),
                Num(OP_I32_ADD),
                Var(OP_GLOBAL_SET(0)),
            ],
        }];
        e.run_function(0);
        assert_eq!(I32(420), e.store.globals[0].val);
    }
}
