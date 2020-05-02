use crate::engine::StackContent::*;
use crate::engine::Value::*;
use std::cell::RefCell;
use std::fmt;
use std::ops::{Add, BitAnd, BitOr, BitXor, Div, Mul, Rem, Shl, Shr, Sub};
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

#[derive(Debug, Clone)]
#[allow(dead_code)]
enum InstructionOutcome {
    Trap,
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

impl Sub for Value {
    type Output = Self;
    fn sub(self, other: Self) -> Self {
        match (self, other) {
            (I32(v1), I32(v2)) => I32(v1 - v2),
            (I64(v1), I64(v2)) => I64(v1 - v2),
            (F32(v1), F32(v2)) => F32(v1 - v2),
            (F64(v1), F64(v2)) => F64(v1 - v2),
            _ => panic!("Type missmatch during subtraction"),
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
            _ => panic!("Type missmatch during subtraction"),
        }
    }
}

impl Div for Value {
    type Output = Self;
    fn div(self, other: Self) -> Self {
        match (self, other) {
            (I32(v1), I32(v2)) => I32(v1 / v2),
            (I64(v1), I64(v2)) => I64(v1 / v2),
            (F32(v1), F32(v2)) => F32(v1 / v2),
            (F64(v1), F64(v2)) => F64(v1 / v2),
            _ => panic!("Type missmatch during division"),
        }
    }
}

impl BitAnd for Value {
    type Output = Self;
    fn bitand(self, other: Self) -> Self {
        match (self, other) {
            (I32(v1), I32(v2)) => I32(v1 & v2),
            (I64(v1), I64(v2)) => I64(v1 & v2),
            _ => panic!("Type missmatch during bitand"),
        }
    }
}

impl BitOr for Value {
    type Output = Self;
    fn bitor(self, other: Self) -> Self {
        match (self, other) {
            (I32(v1), I32(v2)) => I32(v1 | v2),
            (I64(v1), I64(v2)) => I64(v1 | v2),
            _ => panic!("Type missmatch during bitor"),
        }
    }
}

impl BitXor for Value {
    type Output = Self;
    fn bitxor(self, other: Self) -> Self {
        match (self, other) {
            (I32(v1), I32(v2)) => I32(v1 ^ v2),
            (I64(v1), I64(v2)) => I64(v1 ^ v2),
            _ => panic!("Type missmatch during bitxor"),
        }
    }
}

impl Shl for Value {
    type Output = Self;
    fn shl(self, other: Self) -> Self {
        match (self, other) {
            (I32(v1), I32(v2)) => I32(v1 << v2),
            (I64(v1), I64(v2)) => I64(v1 << v2),
            _ => panic!("Type missmatch during shift left"),
        }
    }
}

impl Shr for Value {
    type Output = Self;
    fn shr(self, other: Self) -> Self {
        match (self, other) {
            (I32(v1), I32(v2)) => I32(v1 >> v2),
            (I64(v1), I64(v2)) => I64(v1 >> v2),
            _ => panic!("Type missmatch during shift right"),
        }
    }
}

impl Rem for Value {
    type Output = Self;
    fn rem(self, other: Self) -> Self {
        match (self, other) {
            (I32(v1), I32(v2)) => I32(v1 % v2),
            (I64(v1), I64(v2)) => I64(v1 % v2),
            (F32(v1), F32(v2)) => F32(v1 % v2),
            (F64(v1), F64(v2)) => F64(v1 % v2),
            _ => panic!("Type missmatch during remainder"),
        }
    }
}

macro_rules! impl_two_op_integer {
    ($f:ident) => {
        fn $f(left: Value, right: Value) -> Value {
            match (left, right) {
                (I32(v1), I32(v2)) => I32(v1.$f(v2 as u32)),
                (I64(v1), I64(v2)) => I64(v1.$f(v2 as u32)),
                _ => panic!("Type mismatch during {}", stringify!($f)),
            }
        }
    };
}

macro_rules! impl_two_op_all_numbers {
    ($f:ident, $k:expr) => {
        fn $f(left: Value, right: Value) -> Value {
            match (left, right) {
                (I32(v1), I32(v2)) => I32($k(v1, v2) as i32),
                (I64(v1), I64(v2)) => I64($k(v1, v2) as i64),
                (F32(v1), F32(v2)) => F32($k(v1, v2) as u32 as f32),
                (F64(v1), F64(v2)) => F64($k(v1, v2) as u32 as f64),
                _ => panic!("Type mismatch during {}", stringify!($f)),
            }
        }
    };
}

macro_rules! impl_one_op_integer {
    ($f:ident) => {
        fn $f(left: Value) -> Value {
            match left {
                I32(v1) => I32(v1.$f() as i32),
                I64(v1) => I64(v1.$f() as i64),
                _ => panic!("Type mismatch during {}", stringify!($f)),
            }
        }
    };
}

impl_two_op_integer!(rotate_left);
impl_two_op_integer!(rotate_right);

impl_one_op_integer!(leading_zeros);
impl_one_op_integer!(trailing_zeros);
impl_one_op_integer!(count_ones);

impl_two_op_all_numbers!(lt, |left, right| left < right);
impl_two_op_all_numbers!(gt, |left, right| left > right);
impl_two_op_all_numbers!(le, |left, right| left <= right);
impl_two_op_all_numbers!(ge, |left, right| left >= right);

fn eqz(left: Value) -> Value {
    match left {
        I32(v1) => I32((v1 == 0_i32) as i32),
        I64(v1) => I64((v1 == 0_i64) as i64),
        _ => panic!("Type missmatch during eqz"),
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

macro_rules! fetch_unop {
    ($stack: expr) => {{
        let v1 = match $stack.pop().unwrap() {
            Value(v) => v,
            x => panic!("Top of stack was not of type $v_ty: {:?}", x),
        };
        (v1)
    }};
}

macro_rules! fetch_binop {
    ($stack: expr) => {{
        let v1 = fetch_unop!($stack);
        let v2 = fetch_unop!($stack);

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
        let start_function = crate::instantiation::instantiation(m, &self.module, &mut self.store)
            .expect("Instantiation failed");

        if let Some(func_addr) = start_function {
            debug!("Invoking start function with {:?}", func_addr);
            self.invoke_function(func_addr, vec![]);
        }
    }

    #[warn(dead_code)]
    pub fn invoke_function(&mut self, idx: u32, args: Vec<Value>) {
        //TODO check if function[idx] exists

        self.check_parameters_of_function(idx, &args);

        self.store.stack.push(Frame(Frame {
            arity: args.len() as u32,
            locals: args,
            module_instance: Rc::downgrade(&self.module),
        }));

        debug!("Invoking function on {:#?}", self);
        self.run_function(idx).expect("run function failed");
    }

    fn check_parameters_of_function(&self, idx: u32, args: &[Value]) {
        let fn_types = &self.module.borrow().fn_types[idx as usize];

        if fn_types.param_types
            != args
                .iter()
                .cloned()
                .map(|w| w.into())
                .collect::<Vec<ValueType>>()
        {
            panic!("Function expected different parameters");
        }
    }

    fn run_function(&mut self, idx: u32) -> Result<(), InstructionOutcome> {
        debug!("Running function {:?}", idx);

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
                Num(OP_I32_DIV_U) | Num(OP_I32_DIV_S) | Num(OP_I64_DIV_S) | Num(OP_I64_DIV_U)
                | Num(OP_F32_DIV) | Num(OP_F64_DIV) => {
                    let (v1, v2) = fetch_binop!(self.store.stack);
                    self.store.stack.push(Value(v1 * v2))
                }
                Num(OP_I32_REM_U) | Num(OP_I64_REM_U) | Num(OP_I32_REM_S) | Num(OP_I64_REM_S) => {
                    let (v1, v2) = fetch_binop!(self.store.stack);
                    self.store.stack.push(Value(v1 % v2))
                }
                Num(OP_I32_AND) | Num(OP_I64_AND) => {
                    let (v1, v2) = fetch_binop!(self.store.stack);
                    self.store.stack.push(Value(v1 & v2))
                }
                Num(OP_I32_OR) | Num(OP_I64_OR) => {
                    let (v1, v2) = fetch_binop!(self.store.stack);
                    self.store.stack.push(Value(v1 | v2))
                }
                Num(OP_I32_XOR) | Num(OP_I64_XOR) => {
                    let (v1, v2) = fetch_binop!(self.store.stack);
                    self.store.stack.push(Value(v1 ^ v2))
                }
                Num(OP_I32_SHL) | Num(OP_I64_SHL) => {
                    let (v1, v2) = fetch_binop!(self.store.stack);
                    self.store.stack.push(Value(v1 << v2))
                }
                Num(OP_I32_SHR_U) | Num(OP_I64_SHR_U) | Num(OP_I32_SHR_S) | Num(OP_I64_SHR_S) => {
                    let (v1, v2) = fetch_binop!(self.store.stack);
                    self.store.stack.push(Value(v1 >> v2))
                }
                Num(OP_I32_ROTL) | Num(OP_I64_ROTL) => {
                    let (v1, v2) = fetch_binop!(self.store.stack);
                    self.store.stack.push(Value(rotate_left(v1, v2)))
                }
                Num(OP_I32_ROTR) | Num(OP_I64_ROTR) => {
                    let (v1, v2) = fetch_binop!(self.store.stack);
                    self.store.stack.push(Value(rotate_right(v1, v2)))
                }
                Num(OP_I32_CLZ) | Num(OP_I64_CLZ) => {
                    let v1 = fetch_unop!(self.store.stack);
                    self.store.stack.push(Value(leading_zeros(v1)))
                }
                Num(OP_I32_CTZ) | Num(OP_I64_CTZ) => {
                    let v1 = fetch_unop!(self.store.stack);
                    self.store.stack.push(Value(trailing_zeros(v1)))
                }
                Num(OP_I32_POPCNT) | Num(OP_I64_POPCNT) => {
                    let v1 = fetch_unop!(self.store.stack);
                    self.store.stack.push(Value(count_ones(v1)))
                }
                Num(OP_I32_EQZ) | Num(OP_I64_EQZ) => {
                    let v1 = fetch_unop!(self.store.stack);

                    self.store.stack.push(Value(eqz(v1)))
                }
                Num(OP_I32_EQ) | Num(OP_I64_EQ) | Num(OP_F32_EQ) | Num(OP_F64_EQ) => {
                    let (v1, v2) = fetch_binop!(self.store.stack);
                    let res = v1 == v2;

                    if res {
                        self.store.stack.push(StackContent::Value(Value::I32(1)))
                    } else {
                        self.store.stack.push(StackContent::Value(Value::I32(0)))
                    }
                }
                Num(OP_I32_NE) | Num(OP_I64_NE) | Num(OP_F32_NE) | Num(OP_F64_NE) => {
                    let (v1, v2) = fetch_binop!(self.store.stack);
                    let res = v1 != v2;

                    if res {
                        self.store.stack.push(StackContent::Value(Value::I32(1)))
                    } else {
                        self.store.stack.push(StackContent::Value(Value::I32(0)))
                    }
                }
                Num(OP_I32_LT_S) | Num(OP_I64_LT_S) | Num(OP_F32_LT) | Num(OP_F64_LT)
                | Num(OP_I32_LT_U) | Num(OP_I64_LT_U) => {
                    let (v1, v2) = fetch_binop!(self.store.stack);
                    self.store.stack.push(Value(lt(v1, v2)))
                }
                Num(OP_I32_GT_S) | Num(OP_I64_GT_S) | Num(OP_F32_GT) | Num(OP_F64_GT)
                | Num(OP_I32_GT_U) | Num(OP_I64_GT_U) => {
                    let (v1, v2) = fetch_binop!(self.store.stack);
                    self.store.stack.push(Value(gt(v1, v2)))
                }
                Num(OP_I32_LE_S) | Num(OP_I64_LE_S) | Num(OP_F32_LE) | Num(OP_F64_LE)
                | Num(OP_I32_LE_U) | Num(OP_I64_LE_U) => {
                    let (v1, v2) = fetch_binop!(self.store.stack);
                    self.store.stack.push(Value(le(v1, v2)))
                }
                Num(OP_I32_GE_S) | Num(OP_I64_GE_S) | Num(OP_F32_GE) | Num(OP_F64_GE)
                | Num(OP_I32_GE_U) | Num(OP_I64_GE_U) => {
                    let (v1, v2) = fetch_binop!(self.store.stack);
                    self.store.stack.push(Value(ge(v1, v2)))
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
                    self.run_function(*idx)?;
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

        Ok(())
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
    #[should_panic(expected = "Function expected different parameters")]
    fn test_invoke_wrong_parameters() {
        let mut e = empty_engine();

        // We have 2 parameters, but supply 3
        e.module.borrow_mut().fn_types = vec![FunctionSignature {
            param_types: vec![ValueType::I32, ValueType::I32],
            return_types: vec![],
        }];

        e.module.borrow_mut().code = vec![FunctionBody {
            locals: vec![],
            code: vec![Var(OP_LOCAL_GET(0)), Var(OP_LOCAL_GET(1)), Num(OP_I32_ADD)],
        }];

        e.invoke_function(0, vec![Value::I32(1), Value::I32(2), Value::I32(3)]);
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
