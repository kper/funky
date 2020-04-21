use crate::engine::StackContent::*;
use crate::engine::Value::*;
use std::ops::{Add, Mul};
use wasm_parser::core::CtrlInstructions::*;
use wasm_parser::core::Instruction::*;
use wasm_parser::core::NumericInstructions::*;
use wasm_parser::core::VarInstructions::*;
use wasm_parser::core::*;
use wasm_parser::Module;

#[derive(Debug)]
pub struct Engine {
    pub module: ModuleInstance,
    pub store: Store,
    pub started: bool,
}

#[derive(Debug, PartialEq, Clone, Copy)]
#[allow(dead_code)]
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

#[derive(Debug, PartialEq, Clone)]
#[allow(dead_code)]
pub enum StackContent {
    Value(Value),
    Frame(Frame),
}

#[derive(Debug, PartialEq, Clone)]
pub struct Frame {
    arity: u32,
    locals: Vec<Value>,
}

#[derive(Debug)]
pub struct ModuleInstance {
    start: u32,
    code: Vec<FunctionBody>,
    fn_types: Vec<FunctionSignature>,
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
            fn_types: Vec::new(),
        };
        for section in m.sections {
            match section {
                Section::Code(CodeSection { entries: x }) => mi.code = x,
                Section::Type(TypeSection { entries: x }) => mi.fn_types = x,
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
    #[warn(dead_code)]
    pub fn invoke_function(&mut self, idx: u32, args: Vec<Value>) {
        self.store.stack.push(Frame(Frame {
            arity: args.len() as u32,
            locals: args,
        }));
        self.run_function(idx);
    }
    fn run_function(&mut self, idx: u32) {
        debug!("Running function {:?}", idx);
        let f = self.module.code[idx as usize].clone();
        let mut fr = match self.store.stack.last().cloned() {
            Some(Frame(fr)) => fr,
            Some(x) => panic!("Expected frame but found {:?}", x),
            None => panic!("Empty stack on function call"),
        };
        let mut ip = 0;
        while ip < f.code.len() {
            debug!("Evaluating instruction {:?}", &f.code[ip]);
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
                Num(OP_F32_CONST(v)) => self.store.stack.push(Value(F32(*v))),
                Num(OP_F64_CONST(v)) => self.store.stack.push(Value(F64(*v))),
                Num(OP_I32_ADD) | Num(OP_I64_ADD) | Num(OP_F32_ADD) | Num(OP_F64_ADD) => {
                    let (v1, v2) = fetch_binop!(self.store.stack);
                    self.store.stack.push(Value(v1 + v2))
                }
                Num(OP_I32_MUL) | Num(OP_I64_MUL) | Num(OP_F32_MUL) | Num(OP_F64_MUL) => {
                    let (v1, v2) = fetch_binop!(self.store.stack);
                    self.store.stack.push(Value(v1 * v2))
                }
                Ctrl(OP_CALL(idx)) => {
                    let t = &self.module.fn_types[*idx as usize];
                    let args = self
                        .store
                        .stack
                        .split_off(self.store.stack.len() - t.param_types.len())
                        .into_iter()
                        .map(|x| match x {
                            Value(v) => v,
                            other => panic!("Expected value but found {:?}", other),
                        })
                        .collect();
                    let cfr = Frame {
                        arity: t.return_types.len() as u32,
                        locals: args,
                    };
                    debug!("Calling {:?} with {:#?}", *idx, cfr);
                    self.store.stack.push(Frame(cfr));
                    self.run_function(*idx);
                }
                Ctrl(OP_RETURN) | Ctrl(OP_END) => {
                    break;
                }
                Ctrl(OP_NOP) => {}
                x => panic!("Instruction {:?} not implemented", x),
            }
            ip += 1;
        }
        // implicit return
        let mut ret = Vec::new();
        for _ in 0..fr.arity {
            match self.store.stack.pop() {
                Some(Value(v)) => ret.push(Value(v)),
                Some(x) => panic!("Expected value but found {:?}", x),
                None => panic!("Unexpected empty stack!"),
            }
        }
        while let Some(Frame(_)) = self.store.stack.pop() {}
        self.store.stack.append(&mut ret);
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
                fn_types: Vec::new(),
            },
            store: Store {
                stack: vec![Frame(Frame {
                    arity: 0,
                    locals: Vec::new(),
                })],
                globals: Vec::new(),
                memory: Vec::new(),
            },
        }
    }

    #[test]
    fn test_run_function() {
        let mut e = empty_engine();
        e.store.stack = vec![Frame(Frame {
            arity: 1,
            locals: Vec::new(),
        })];
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
        e.store.stack = vec![Frame(Frame {
            arity: 1,
            locals: Vec::new(),
        })];
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
            arity: 1,
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
            arity: 1,
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
