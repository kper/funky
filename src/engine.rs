use crate::engine::StackContent::*;
use crate::engine::Value::*;
use std::cell::RefCell;
use std::fmt;
use std::ops::{Add, Mul};
use std::rc::{Rc, Weak};
use wasm_parser::core::CtrlInstructions::*;
use wasm_parser::core::Instruction::*;
use wasm_parser::core::NumericInstructions::*;
use wasm_parser::core::ParamInstructions::*;
use wasm_parser::core::VarInstructions::*;
use wasm_parser::core::*;
use wasm_parser::Module;

#[derive(Debug)]
pub struct Engine {
    pub module: Rc<RefCell<ModuleInstance>>, //TODO rename to `module_instance`
    pub started: bool,
    pub store: Store,
}

#[derive(Debug, PartialEq, Clone, Copy)]
#[allow(dead_code)]
pub enum Value {
    I32(i32),
    I64(i64),
    F32(f32),
    F64(f64),
}

type Arity = u32;

#[derive(Debug, Clone)]
#[allow(dead_code)]
pub enum ExtendedInstruction {
    Trap,
    Invoke(FuncIdx),
    InitElement(TableIdx, u32, Vec<FuncIdx>),
    InitData(MemoryIdx, u32, Vec<u8>),
    Label(Arity, Vec<ExtendedInstruction>, Vec<Instruction>),
    Frame(Arity, Option<Frame>, Vec<Instruction>),
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

//#[derive(Debug, PartialEq, Clone, Copy)]
//pub struct Trap;
//type Result<T> = std::result::Result<T, Trap>;

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

#[derive(Debug, Clone, PartialEq)]
pub struct Variable {
    pub mutable: bool, //Actually, there is a `Mut` enum. TODO check if makes sense to use it
    pub val: Value,
}

#[derive(Debug, Clone, PartialEq)]
#[allow(dead_code)]
pub enum StackContent {
    Value(Value),
    Frame(Frame), //TODO add labels
}

#[derive(Debug, Clone)]
pub struct Frame {
    pub arity: u32,
    pub locals: Vec<Value>,
    pub module_instance: Weak<RefCell<ModuleInstance>>,
}

impl PartialEq for Frame {
    fn eq(&self, other: &Self) -> bool {
        self.arity == other.arity && self.locals == other.locals
        //&& self.module_instance.ptr_eq(&other.module_instance)
    }
}

#[derive(Debug, Clone)]
pub struct ModuleInstance {
    pub start: u32,
    pub code: Vec<FunctionBody>,
    pub fn_types: Vec<FunctionSignature>,
    pub funcaddrs: Vec<FuncIdx>,
    pub tableaddrs: Vec<TableIdx>,
    pub memaddrs: Vec<MemoryIdx>,
    pub globaladdrs: Vec<GlobalIdx>,
    pub exports: Vec<ExportInstance>,
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
    pub ty: FunctionSignature,
    pub module: Weak<RefCell<ModuleInstance>>,
    pub code: FunctionBody,
}

#[derive(Debug, Clone)]
pub struct TableInstance {
    pub elem: Vec<Option<FuncIdx>>,
    pub max: Option<u32>,
}

#[derive(Clone)]
pub struct MemoryInstance {
    pub data: Vec<u8>,
    pub max: Option<u32>,
}

/// Overwritten debug implementation
/// Because `data` can have a lot of entries, which
/// can be a problem when printing
impl fmt::Debug for MemoryInstance {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("MemoryInstance")
            .field("data (only length)", &self.data.len())
            .field("max", &self.max)
            .finish()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct ExportInstance {
    pub name: String,
    pub value: ExternalKindType, //TODO maybe drop the Type in name?
}

impl Into<ExportInstance> for &ExportEntry {
    fn into(self) -> ExportInstance {
        ExportInstance {
            name: self.name.clone(),
            value: self.kind,
        }
    }
}

/*
#[derive(Debug, Clone)]
#[allow(dead_code)]
pub enum ExternalVal {
    Func(FuncIdx),
    Table(TableIdx),
    Mem(MemoryIdx),
    Global(GlobalIdx),
}
*/

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
    pub fn new(m: &Module) -> Self {
        let mut mi = ModuleInstance {
            start: 0,
            code: Vec::new(),
            fn_types: Vec::new(),
            funcaddrs: Vec::new(),
            tableaddrs: Vec::new(),
            memaddrs: Vec::new(),
            globaladdrs: Vec::new(),
            exports: Vec::new(),
        };
        for section in m.sections.iter() {
            match section {
                Section::Code(CodeSection { entries: x }) => mi.code = x.clone(),
                Section::Type(TypeSection { entries: x }) => mi.fn_types = x.clone(),
                _ => {}
            }
        }

        mi
    }
}

impl Engine {
    pub fn new(mi: ModuleInstance, module: &Module) -> Self {
        let mut e = Engine {
            module: Rc::new(RefCell::new(mi)),
            started: false,
            store: Store {
                funcs: Vec::new(),
                tables: Vec::new(),
                stack: Vec::new(),
                globals: Vec::new(),
                memory: Vec::new(),
            },
        };

        e.allocate(module);

        e
    }

    pub fn downgrade_mod_instance(&self) -> Weak<RefCell<ModuleInstance>> {
        Rc::downgrade(&self.module)
    }

    fn allocate(&mut self, m: &Module) {
        info!("Allocation");
        crate::allocation::allocate(m, &self.module, &mut self.store).expect("Allocation failed");
    }

    pub fn instantiation(&mut self, m: &Module) {
        info!("Instantiation");
        crate::instantiation::instantiation(m, &self.module, &mut self.store)
            .expect("Instantiation failed");
    }

    #[warn(dead_code)]
    pub fn invoke_function(&mut self, idx: u32, args: Vec<Value>) {
        self.store.stack.push(Frame(Frame {
            arity: args.len() as u32,
            locals: args,
            module_instance: Rc::downgrade(&self.module),
        }));
        debug!("Invoking function on {:#?}", self);
        self.run_function(idx);
    }
    fn run_function(&mut self, idx: u32) {
        debug!("Running function {:?}", idx);
        //let instance = self.module.borrow_mut();
        let f = self.module.borrow().code[idx as usize].clone();
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
                Param(OP_DROP) => {
                    self.store.stack.pop();
                }
                Param(OP_SELECT) => {
                    let c = match self.store.stack.pop() {
                        Some(Value(I32(x))) => x,
                        _ => panic!("Expected I32 on top of stack"),
                    };
                    let (v1, v2) = fetch_binop!(self.store.stack);
                    if c != 0 {
                        self.store.stack.push(Value(v1))
                    } else {
                        self.store.stack.push(Value(v2))
                    }
                }
                Ctrl(OP_CALL(idx)) => {
                    let t = self.module.borrow().fn_types[*idx as usize].clone();
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
                        module_instance: Rc::downgrade(&self.module),
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
        let mi = Rc::new(RefCell::new(ModuleInstance {
            start: 0,
            code: Vec::new(),
            fn_types: Vec::new(),
            funcaddrs: Vec::new(),
            tableaddrs: Vec::new(),
            memaddrs: Vec::new(),
            globaladdrs: Vec::new(),
            exports: Vec::new(),
        }));
        Engine {
            started: true,
            store: Store {
                funcs: Vec::new(),
                tables: Vec::new(),
                globals: Vec::new(),
                memory: Vec::new(),
                stack: vec![Frame(Frame {
                    arity: 0,
                    locals: Vec::new(),
                    module_instance: Rc::downgrade(&mi),
                })],
            },
            module: mi,
        }
    }

    #[test]
    fn test_run_function() {
        let mut e = empty_engine();
        e.store.stack = vec![Frame(Frame {
            arity: 1,
            locals: Vec::new(),
            module_instance: e.downgrade_mod_instance(),
        })];
        e.module.borrow_mut().code = vec![FunctionBody {
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
            module_instance: e.downgrade_mod_instance(),
        })];
        e.module.borrow_mut().code = vec![FunctionBody {
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
            module_instance: e.downgrade_mod_instance(),
        })];
        e.module.borrow_mut().code = vec![FunctionBody {
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
            module_instance: e.downgrade_mod_instance(),
        })];
        e.module.borrow_mut().code = vec![FunctionBody {
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
        e.module.borrow_mut().code = vec![FunctionBody {
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
        e.module.borrow_mut().code = vec![FunctionBody {
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
