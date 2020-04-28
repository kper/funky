use crate::engine::StackContent::*;
use crate::engine::Value::*;
use std::ops::{Add, Mul};
use wasm_parser::core::CtrlInstructions::*;
use wasm_parser::core::Instruction::*;
use wasm_parser::core::NumericInstructions::*;
use wasm_parser::core::ParamInstructions::*;
use wasm_parser::core::VarInstructions::*;
use wasm_parser::core::*;
use wasm_parser::Module;

#[derive(Debug)]
pub struct Engine {
    pub module: ModuleInstance,
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

impl Into<ValueType> for Value {
    fn into(self) -> ValueType {
        match self {
            Value::I32(_) => ValueType::I32,
            Value::I64(_) => ValueType::I64,
            Value::F32(_) => ValueType::F32,
            Value::F64(_) => ValueType::F64,
        }
    }
}

#[derive(Debug, PartialEq, Clone, Copy)]
pub struct Trap;
type Result<T> = std::result::Result<T, Trap>;

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

#[derive(Debug, Clone)]
pub struct Variable {
    mutable: bool, //Actually, there is a `Mut` enum. TODO check if makes sense to use it
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

#[derive(Debug, Clone)]
pub struct ModuleInstance {
    start: u32,
    code: Vec<FunctionBody>,
    fn_types: Vec<FunctionSignature>,
    tableaddrs: Vec<TableIdx>,
    memaddrs: Vec<MemoryIdx>,
    globaladdrs: Vec<GlobalIdx>,
    exports: Vec<ExportInstance>,
    pub store: Store,
}

#[derive(Debug, Clone)]
pub struct Store {
    pub funcs: Vec<FuncInstance>,
    pub tables: Vec<TableInstance>,
    pub memory: Vec<MemoryInstance>,
    pub stack: Vec<StackContent>,
    pub globals: Vec<Variable>, //=GlobalInstance
}

#[derive(Debug, Clone)]
pub struct FuncInstance {
    //FIXME Add HostFunc
    ty: FunctionSignature,
    module: Box<ModuleInstance>,
    code: FunctionBody,
}

#[derive(Debug, Clone)]
pub struct TableInstance {
    elem: Vec<FuncIdx>,
    max: Option<u32>,
}

#[derive(Debug, Clone)]
pub struct MemoryInstance {
    data: Vec<u8>,
    max: Option<u32>,
}

#[derive(Debug, Clone)]
pub struct ExportInstance {
    name: String,
    value: ExternalVal,
}

#[derive(Debug, Clone)]
pub enum ExternalVal {
    Func(FuncIdx),
    Table(TableIdx),
    Mem(MemoryIdx),
    Global(GlobalIdx),
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
            tableaddrs: Vec::new(),
            memaddrs: Vec::new(),
            globaladdrs: Vec::new(),
            exports: Vec::new(),
            store: Store {
                funcs: Vec::new(),
                tables: Vec::new(),
                stack: Vec::new(),
                globals: Vec::new(),
                memory: Vec::new(),
            },
        };
        for section in m.sections.iter() {
            match section {
                Section::Code(CodeSection { entries: x }) => mi.code = x.clone(),
                Section::Type(TypeSection { entries: x }) => mi.fn_types = x.clone(),

                _ => {}
            }
        }

        mi.allocate(&m).expect("Allocation failed");

        mi
    }

    pub fn allocate(&mut self, m: &Module) -> std::result::Result<(), ()> {
        self.allocate_functions(m)?;
        //TODO host functions
        self.allocate_tables(m)?;
        self.allocate_memories(m)?;
        self.allocate_globals(m)?;

        Ok(())
    }

    fn allocate_functions(&mut self, m: &Module) -> std::result::Result<(), ()> {
        let ty: Vec<_> = m
            .sections
            .iter()
            .filter_map(|ref w| match w {
                Section::Function(t) => Some(&t.types),
                _ => None,
            })
            .flatten()
            .collect();

        for t in ty.into_iter() {
            // Allocate function

            if let Some(f) = self.fn_types.get(*t as usize) {
                if let Some(c) = self.code.get(*t as usize) {
                    let instance = FuncInstance {
                        ty: f.clone(),
                        module: Box::new(self.clone()),
                        code: c.clone(),
                    };

                    self.store.funcs.push(instance);
                } else {
                    error!("{} code is not defined", t);
                    return Err(());
                }
            } else {
                error!("{} function type is not defined", t);
                return Err(());
            }
        }

        Ok(())
    }

    fn allocate_tables(&mut self, m: &Module) -> std::result::Result<(), ()> {
        let ty: Vec<_> = m
            .sections
            .iter()
            .filter_map(|ref w| match w {
                Section::Table(t) => Some(&t.entries),
                _ => None,
            })
            .flatten()
            .collect();

        for t in ty.into_iter() {
            let instance = match t.limits {
                Limits::Zero(n) => TableInstance {
                    elem: Vec::with_capacity(n as usize),
                    max: None,
                },
                Limits::One(n, m) => TableInstance {
                    elem: Vec::with_capacity(n as usize),
                    max: Some(m),
                },
            };

            self.tableaddrs.push(self.store.tables.len() as u32);
            self.store.tables.push(instance);
        }

        Ok(())
    }

    fn allocate_memories(&mut self, m: &Module) -> std::result::Result<(), ()> {
        let ty: Vec<_> = m
            .sections
            .iter()
            .filter_map(|ref w| match w {
                Section::Memory(t) => Some(&t.entries),
                _ => None,
            })
            .flatten()
            .collect();

        for memtype in ty.into_iter() {
            let instance = match memtype.limits {
                Limits::Zero(n) => MemoryInstance {
                    data: Vec::with_capacity((n * 1024 * 64) as usize),
                    max: None,
                },
                Limits::One(n, m) => MemoryInstance {
                    data: Vec::with_capacity((n * 1024 * 64) as usize),
                    max: Some(m),
                },
            };

            self.memaddrs.push(self.store.memory.len() as u32);
            self.store.memory.push(instance);
        }

        Ok(())
    }

    fn allocate_globals(&mut self, m: &Module) -> std::result::Result<(), ()> {
        let ty: Vec<_> = m
            .sections
            .iter()
            .filter_map(|ref w| match w {
                Section::Global(t) => Some(&t.globals),
                _ => None,
            })
            .flatten()
            .collect();

        for gl in ty.into_iter() {
            let instance = Variable {
                mutable: match gl.ty.mu {
                    Mu::Var => true,
                    _ => false,
                },
                val: get_expr_const_ty_global(&gl.init)?,
            };

            self.globaladdrs.push(self.store.globals.len() as u32);
            self.store.globals.push(instance);
        }

        Ok(())
    }
}

impl Engine {
    pub fn new(mi: ModuleInstance) -> Self {
        Engine {
            module: mi,
            started: false,
        }
    }
    #[warn(dead_code)]
    pub fn invoke_function(&mut self, idx: u32, args: Vec<Value>) {
        self.module.store.stack.push(Frame(Frame {
            arity: args.len() as u32,
            locals: args,
        }));
        self.run_function(idx);
    }
    fn run_function(&mut self, idx: u32) {
        debug!("Running function {:?}", idx);
        let f = self.module.code[idx as usize].clone();
        let mut fr = match self.module.store.stack.last().cloned() {
            Some(Frame(fr)) => fr,
            Some(x) => panic!("Expected frame but found {:?}", x),
            None => panic!("Empty stack on function call"),
        };
        let mut ip = 0;
        while ip < f.code.len() {
            debug!("Evaluating instruction {:?}", &f.code[ip]);
            match &f.code[ip] {
                Var(OP_LOCAL_GET(idx)) => self
                    .module
                    .store
                    .stack
                    .push(Value(fr.locals[*idx as usize])),
                Var(OP_LOCAL_SET(idx)) => match self.module.store.stack.pop() {
                    Some(Value(v)) => fr.locals[*idx as usize] = v,
                    Some(x) => panic!("Expected value but found {:?}", x),
                    None => panic!("Empty stack during local.set"),
                },
                Var(OP_LOCAL_TEE(idx)) => match self.module.store.stack.last() {
                    Some(Value(v)) => fr.locals[*idx as usize] = *v,
                    Some(x) => panic!("Expected value but found {:?}", x),
                    None => panic!("Empty stack during local.set"),
                },
                Var(OP_GLOBAL_GET(idx)) => self
                    .module
                    .store
                    .stack
                    .push(Value(self.module.store.globals[*idx as usize].val)),
                Var(OP_GLOBAL_SET(idx)) => match self.module.store.stack.pop() {
                    Some(Value(v)) => {
                        if !self.module.store.globals[*idx as usize].mutable {
                            panic!("Attempting to modify a immutable global")
                        }
                        self.module.store.globals[*idx as usize].val = v
                    }
                    Some(x) => panic!("Expected value but found {:?}", x),
                    None => panic!("Empty stack during local.set"),
                },
                Num(OP_I32_CONST(v)) => self.module.store.stack.push(Value(I32(*v))),
                Num(OP_I64_CONST(v)) => self.module.store.stack.push(Value(I64(*v))),
                Num(OP_F32_CONST(v)) => self.module.store.stack.push(Value(F32(*v))),
                Num(OP_F64_CONST(v)) => self.module.store.stack.push(Value(F64(*v))),
                Num(OP_I32_ADD) | Num(OP_I64_ADD) | Num(OP_F32_ADD) | Num(OP_F64_ADD) => {
                    let (v1, v2) = fetch_binop!(self.module.store.stack);
                    self.module.store.stack.push(Value(v1 + v2))
                }
                Num(OP_I32_MUL) | Num(OP_I64_MUL) | Num(OP_F32_MUL) | Num(OP_F64_MUL) => {
                    let (v1, v2) = fetch_binop!(self.module.store.stack);
                    self.module.store.stack.push(Value(v1 * v2))
                }
                Param(OP_DROP) => {
                    self.module.store.stack.pop();
                }
                Param(OP_SELECT) => {
                    let c = match self.module.store.stack.pop() {
                        Some(Value(I32(x))) => x,
                        _ => panic!("Expected I32 on top of stack"),
                    };
                    let (v1, v2) = fetch_binop!(self.module.store.stack);
                    if c != 0 {
                        self.module.store.stack.push(Value(v1))
                    } else {
                        self.module.store.stack.push(Value(v2))
                    }
                }
                Ctrl(OP_CALL(idx)) => {
                    let t = &self.module.fn_types[*idx as usize];
                    let args = self
                        .module
                        .store
                        .stack
                        .split_off(self.module.store.stack.len() - t.param_types.len())
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
                    self.module.store.stack.push(Frame(cfr));
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
            match self.module.store.stack.pop() {
                Some(Value(v)) => ret.push(Value(v)),
                Some(x) => panic!("Expected value but found {:?}", x),
                None => panic!("Unexpected empty stack!"),
            }
        }
        while let Some(Frame(_)) = self.module.store.stack.pop() {}
        self.module.store.stack.append(&mut ret);
    }
}

fn get_expr_const_ty_global<'a>(
    init: &Expr,
) -> std::result::Result<Value, ()> {
    use wasm_parser::core::NumericInstructions::*;
    use wasm_parser::core::VarInstructions::*;

    if init.is_empty() {
        error!("No expr to evaluate");
        return Err(());
    }

    match init.get(0).unwrap() {
        Instruction::Num(n) => match *n {
            OP_I32_CONST(v) => Ok(Value::I32(v)),
            OP_I64_CONST(v) => Ok(Value::I64(v)),
            OP_F32_CONST(v) => Ok(Value::F32(v)),
            OP_F64_CONST(v) => Ok(Value::F64(v)),
            _ => {
                error!("Expression is not a const");
                return Err(());
            },
        },
        _ => {
            error!("Wrong expression");
            return Err(());
        },
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

    #[test]
    fn test_drop_select() {
        let mut e = empty_engine();
        e.store.globals = vec![Variable {
            mutable: true,
            val: I32(20),
        }];
        e.module.code = vec![FunctionBody {
            locals: vec![],
            code: vec![
                Num(OP_I32_CONST(1)),
                Num(OP_I32_CONST(2)),
                Num(OP_I32_CONST(0)),
                Num(OP_I32_CONST(4)),
                Param(OP_DROP),
                Param(OP_SELECT),
                Var(OP_GLOBAL_SET(0)),
            ],
        }];
        e.run_function(0);
        assert_eq!(I32(1), e.store.globals[0].val);
    }
}
