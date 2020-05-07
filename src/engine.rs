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

#[derive(Debug, PartialEq, Clone)]
enum InstructionError {
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

macro_rules! impl_one_op_float {
    ($f:ident) => {
        fn $f(left: Value) -> Value {
            match left {
                F32(v1) => F32(v1.$f() as f32),
                F64(v1) => F64(v1.$f() as f64),
                _ => panic!("Type mismatch during {}", stringify!($f)),
            }
        }
    };
}

macro_rules! impl_one_op_float_closure {
    ($k:ident, $f:expr) => {
        fn $k(left: Value) -> Value {
            match left {
                F32(v1) => F32($f(v1.into()) as f32),
                F64(v1) => F64($f(v1) as f64),
                _ => panic!("Type mismatch during {}", stringify!($f)),
            }
        }
    };
}

macro_rules! impl_two_op_float {
    ($f:ident, $k:expr) => {
        fn $f(left: Value, right: Value) -> Value {
            match (left, right) {
                (F32(v1), F32(v2)) => F32($k(v1.into(), v2.into()) as f32),
                (F64(v1), F64(v2)) => F64($k(v1, v2) as f64),
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

impl_one_op_float!(abs);
impl_one_op_float_closure!(neg, |w: f64| -w);
impl_one_op_float!(ceil);
impl_one_op_float!(floor);
impl_one_op_float!(round);
impl_one_op_float!(sqrt);
impl_one_op_float!(trunc);

impl_two_op_float!(min, |left: f64, right: f64| left.min(right));
impl_two_op_float!(max, |left: f64, right: f64| left.max(right));

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

#[derive(Debug, PartialEq)]
pub enum StackContent {
    Value(Value),
    Frame(Frame),
    Label(Label),
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Label {
    arity: Arity,
    ip_before: usize,
    ip_after: usize,
}

#[derive(Debug)]
pub struct Frame {
    pub arity: u32,
    pub locals: Vec<Value>,
    pub module_instance: Weak<RefCell<ModuleInstance>>,
}

impl PartialEq for Frame {
    fn eq(&self, other: &Self) -> bool {
        self.arity == other.arity && self.locals == other.locals
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

#[derive(Debug)]
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

impl StackContent {
    pub fn is_value(&self) -> bool {
        match self {
            StackContent::Value(_) => true,
            _ => false,
        }
    }

    pub fn is_label(&self) -> bool {
        match self {
            StackContent::Label(_) => true,
            _ => false,
        }
    }
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
        debug!("Popping {:?}", $stack.last());
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

    /// Take only exported functions into consideration
    pub fn invoke_exported_function(&mut self, idx: u32, args: Vec<Value>) {
        debug!("invoke_exported_function {:?}", idx);
        let k = {
            let x = self.module.borrow();

            debug!("x's element {:?}", x.exports.get(idx as usize));

            let w = x
                .exports
                .get(idx as usize)
                .expect("Exported function not found or found something else");

            w.value
        };

        debug!("Exports {:#?}", k);

        let _ = match k {
            ExternalKindType::Function { ty } => {
                let func_addr = *self
                    .module
                    .borrow()
                    .funcaddrs
                    .get(ty as usize)
                    .expect("Function not found");

                self.invoke_function(func_addr, args);
            }
            _ => {
                panic!("Exported function not found");
            }
        };
    }

    fn invoke_function(&mut self, idx: u32, args: Vec<Value>) {
        self.check_parameters_of_function(idx, &args);

        let len = args.len() as u32;

        /*
        self.store.stack.push(Label(Label {
            arity: len,
            ip_before: 0,
            ip_after: self.module.borrow().code[idx as usize].code.len(),
        }));
        */

        self.store.stack.push(Frame(Frame {
            arity: len,
            locals: args,
            module_instance: Rc::downgrade(&self.module),
        }));

        debug!("stack before invoking {:#?}", self.store.stack);

        debug!("Invoking function");
        self.run_function(idx).expect("run function failed");
    }

    fn local_set(&mut self, idx: u32, fr: &mut Frame) -> Result<(), InstructionError> {
        debug!("OP_LOCAL_SET {:?}", idx);
        debug!("locals {:#?}", fr.locals);

        match self.store.stack.pop() {
            Some(Value(v)) => {
                match fr.locals.get_mut(idx as usize) {
                    Some(k) => *k = v, //Exists replace
                    None => {
                        //Does not exists; push
                        fr.locals.push(v)
                    }
                }
            }
            Some(x) => panic!("Expected value but found {:?}", x),
            None => panic!("Empty stack during local.set"),
        }

        Ok(())
    }

    fn check_parameters_of_function(&self, idx: u32, args: &[Value]) {
        let fn_types = &self
            .store
            .funcs
            .get(idx as usize)
            .expect("Function not found")
            .ty
            .param_types;

        if fn_types
            != &args
                .iter()
                .cloned()
                .map(|w| w.into())
                .collect::<Vec<ValueType>>()
        {
            panic!("Function expected different parameters");
        }
    }

    #[allow(clippy::cognitive_complexity)]
    fn run_function(&mut self, idx: u32) -> Result<(), InstructionError> {
        debug!("Running function {:?}", idx);

        let f = self.module.borrow().code[idx as usize].clone(); //TODO maybe remove .clone()

        let mut fr = self.get_frame();

        debug!("frame {:#?}", fr);

        let mut instructions = f.code.clone();
        self.run_instructions(&mut fr, &mut instructions)?;

        // implicit return
        debug!("Implicit return (arity {:?})", fr.arity);

        let mut ret = Vec::new();
        for _ in 0..fr.arity {
            debug!("Popping {:?}", self.store.stack.last());
            match self.store.stack.pop() {
                Some(Value(v)) => ret.push(Value(v)),
                Some(x) => panic!("Expected value but found {:?}", x),
                None => {} //None => panic!("Unexpected empty stack!"),
            }
        }
        debug!("Popping frames");
        while let Some(Frame(_)) = self.store.stack.last() {
            self.store.stack.pop();
        }

        /*
        debug!("Popping labels");
        while let Some(Label(_)) = self.store.stack.last() {
            debug!("Pop label");
            self.store.stack.pop();
        }
        */

        self.store.stack.append(&mut ret);

        debug!("Stack after function return {:#?}", self.store.stack);

        Ok(())
    }

    fn run_instructions(
        &mut self,
        fr: &mut Frame,
        instructions: &mut Vec<Instruction>,
    ) -> Result<(), InstructionError> {
        let mut ip = 0;
        while ip < instructions.len() {
            debug!("Evaluating instruction {:?}", &instructions[ip]);
            match instructions[ip].clone() {
                Var(OP_LOCAL_GET(idx)) => {
                    self.store.stack.push(Value(fr.locals[idx as usize]));
                    debug!("LOCAL_GET at {} is {:?}", idx, fr.locals[idx as usize]);
                }
                Var(OP_LOCAL_SET(idx)) => {
                    self.local_set(idx, fr)?;
                }
                Var(OP_LOCAL_TEE(idx)) => {
                    debug!("OP_LOCAL_TEE {:?}", idx);

                    let value = match self.store.stack.pop() {
                        Some(StackContent::Value(v)) => v,
                        Some(x) => panic!("Expected value but found {:?}", x),
                        None => panic!("Empty stack during local.tee"),
                    };

                    self.store.stack.push(StackContent::Value(value));
                    self.store.stack.push(StackContent::Value(value));

                    self.local_set(idx, fr)?;

                    debug!("stack {:?}", self.store.stack);
                    debug!("locals {:#?}", fr.locals);
                }
                Var(OP_GLOBAL_GET(idx)) => self
                    .store
                    .stack
                    .push(Value(self.store.globals[idx as usize].val)),
                Var(OP_GLOBAL_SET(idx)) => match self.store.stack.pop() {
                    Some(Value(v)) => {
                        if !self.store.globals[idx as usize].mutable {
                            panic!("Attempting to modify a immutable global")
                        }
                        self.store.globals[idx as usize].val = v
                    }
                    Some(x) => panic!("Expected value but found {:?}", x),
                    None => panic!("Empty stack during local.set"),
                },
                Num(OP_I32_CONST(v)) => {
                    debug!("OP_I32_CONST: pushing {} to stack", v);
                    self.store.stack.push(Value(I32(v)));
                    debug!("stack {:#?}", self.store.stack);
                }
                Num(OP_I64_CONST(v)) => {
                    debug!("OP_I64_CONST: pushing {} to stack", v);
                    self.store.stack.push(Value(I64(v)))
                }
                Num(OP_F32_CONST(v)) => {
                    debug!("OP_F32_CONST: pushing {} to stack", v);
                    self.store.stack.push(Value(F32(v)))
                }
                Num(OP_F64_CONST(v)) => {
                    debug!("OP_F64_CONST: pushing {} to stack", v);
                    self.store.stack.push(Value(F64(v)))
                }
                Num(OP_I32_ADD) | Num(OP_I64_ADD) | Num(OP_F32_ADD) | Num(OP_F64_ADD) => {
                    let (v2, v1) = fetch_binop!(self.store.stack);
                    self.store.stack.push(Value(v1 + v2))
                }
                Num(OP_I32_SUB) | Num(OP_I64_SUB) | Num(OP_F32_SUB) | Num(OP_F64_SUB) => {
                    let (v2, v1) = fetch_binop!(self.store.stack);
                    self.store.stack.push(Value(v1 - v2))
                }
                Num(OP_I32_MUL) | Num(OP_I64_MUL) | Num(OP_F32_MUL) | Num(OP_F64_MUL) => {
                    let (v2, v1) = fetch_binop!(self.store.stack);
                    self.store.stack.push(Value(v1 * v2))
                }
                Num(OP_I32_DIV_U) | Num(OP_I32_DIV_S) | Num(OP_I64_DIV_S) | Num(OP_I64_DIV_U)
                | Num(OP_F32_DIV) | Num(OP_F64_DIV) => {
                    let (v2, v1) = fetch_binop!(self.store.stack);
                    self.store.stack.push(Value(v1 * v2))
                }
                Num(OP_I32_REM_U) | Num(OP_I64_REM_U) | Num(OP_I32_REM_S) | Num(OP_I64_REM_S) => {
                    let (v2, v1) = fetch_binop!(self.store.stack);
                    self.store.stack.push(Value(v1 % v2))
                }
                Num(OP_I32_AND) | Num(OP_I64_AND) => {
                    let (v2, v1) = fetch_binop!(self.store.stack);
                    self.store.stack.push(Value(v1 & v2))
                }
                Num(OP_I32_OR) | Num(OP_I64_OR) => {
                    let (v2, v1) = fetch_binop!(self.store.stack);
                    self.store.stack.push(Value(v1 | v2))
                }
                Num(OP_I32_XOR) | Num(OP_I64_XOR) => {
                    let (v2, v1) = fetch_binop!(self.store.stack);
                    self.store.stack.push(Value(v1 ^ v2))
                }
                Num(OP_I32_SHL) | Num(OP_I64_SHL) => {
                    let (v2, v1) = fetch_binop!(self.store.stack);
                    self.store.stack.push(Value(v1 << v2))
                }
                Num(OP_I32_SHR_U) | Num(OP_I64_SHR_U) | Num(OP_I32_SHR_S) | Num(OP_I64_SHR_S) => {
                    let (v2, v1) = fetch_binop!(self.store.stack);
                    self.store.stack.push(Value(v1 >> v2))
                }
                Num(OP_I32_ROTL) | Num(OP_I64_ROTL) => {
                    let (v2, v1) = fetch_binop!(self.store.stack);
                    self.store.stack.push(Value(rotate_left(v1, v2)))
                }
                Num(OP_I32_ROTR) | Num(OP_I64_ROTR) => {
                    let (v2, v1) = fetch_binop!(self.store.stack);
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
                    let (v2, v1) = fetch_binop!(self.store.stack);
                    self.store.stack.push(Value(gt(v1, v2)))
                }
                Num(OP_I32_LE_S) | Num(OP_I64_LE_S) | Num(OP_F32_LE) | Num(OP_F64_LE)
                | Num(OP_I32_LE_U) | Num(OP_I64_LE_U) => {
                    let (v2, v1) = fetch_binop!(self.store.stack);
                    self.store.stack.push(Value(le(v1, v2)))
                }
                Num(OP_I32_GE_S) | Num(OP_I64_GE_S) | Num(OP_F32_GE) | Num(OP_F64_GE)
                | Num(OP_I32_GE_U) | Num(OP_I64_GE_U) => {
                    let (v2, v1) = fetch_binop!(self.store.stack);
                    self.store.stack.push(Value(ge(v1, v2)))
                }
                Num(OP_F32_ABS) | Num(OP_F64_ABS) => {
                    let v1 = fetch_unop!(self.store.stack);
                    self.store.stack.push(Value(abs(v1)))
                }
                Num(OP_F32_NEG) | Num(OP_F64_NEG) => {
                    let v1 = fetch_unop!(self.store.stack);
                    self.store.stack.push(Value(neg(v1)))
                }
                Num(OP_F32_CEIL) | Num(OP_F64_CEIL) => {
                    let v1 = fetch_unop!(self.store.stack);
                    self.store.stack.push(Value(ceil(v1)))
                }
                Num(OP_F32_FLOOR) | Num(OP_F64_FLOOR) => {
                    let v1 = fetch_unop!(self.store.stack);
                    self.store.stack.push(Value(floor(v1)))
                }
                Num(OP_F32_TRUNC) | Num(OP_F64_TRUNC) => {
                    let v1 = fetch_unop!(self.store.stack);
                    self.store.stack.push(Value(trunc(v1)))
                }
                Num(OP_F32_NEAREST) | Num(OP_F64_NEAREST) => {
                    let v1 = fetch_unop!(self.store.stack);
                    self.store.stack.push(Value(round(v1)))
                }
                Num(OP_F32_SQRT) | Num(OP_F64_SQRT) => {
                    let v1 = fetch_unop!(self.store.stack);
                    self.store.stack.push(Value(sqrt(v1)))
                }
                Num(OP_F32_MIN) | Num(OP_F64_MIN) => {
                    let (v1, v2) = fetch_binop!(self.store.stack);
                    self.store.stack.push(Value(min(v1, v2)))
                }
                Num(OP_F32_MAX) | Num(OP_F64_MAX) => {
                    let (v1, v2) = fetch_binop!(self.store.stack);
                    self.store.stack.push(Value(max(v1, v2)))
                }
                Param(OP_DROP) => {
                    debug!("OP_DROP");
                    let k = self.store.stack.pop();
                    debug!("Dropping {:?}", k);
                }
                Param(OP_SELECT) => {
                    debug!("OP_SELECT");
                    debug!("Popping {:?}", self.store.stack.last());
                    let c = match self.store.stack.pop() {
                        Some(Value(I32(x))) => x,
                        _ => panic!("Expected I32 on top of stack"),
                    };
                    let (v1, v2) = fetch_binop!(self.store.stack);
                    if c != 0 {
                        debug!("C is not 0 therefore, pushing {:?}", v2);
                        self.store.stack.push(Value(v2))
                    } else {
                        debug!("C is not 0 therefore, pushing {:?}", v1);
                        self.store.stack.push(Value(v1))
                    }
                }
                Ctrl(OP_BLOCK(ty, block_instructions)) => {
                    debug!("OP_BLOCK {:?}", ty);

                    //let label_idx = self.get_label_count()?;

                    let arity = self.get_block_ty_arity(&ty)?;

                    let label = Label {
                        arity: arity as u32,
                        ip_before: ip,
                        ip_after: ip,
                    };

                    //self.store.stack.push(StackContent::Label(label));

                    self.enter_block(
                        label,
                        fr,
                        instructions,
                        &block_instructions,
                        ip,
                        Instruction::EXIT_BLOCK,
                    )?;

                    //self.exit_block(&label, &block_instructions)?;
                }
                Ctrl(OP_LOOP(ty, block_instructions)) => {
                    debug!("OP_LOOP {:?}, {:?}", ty, block_instructions);

                    let arity = self.get_block_ty_arity(&ty)?;

                    let label = Label {
                        arity: arity as u32,
                        ip_before: ip,
                        ip_after: ip,
                    };

                    self.enter_block(
                        label,
                        fr,
                        instructions,
                        &block_instructions,
                        ip,
                        Instruction::REPEAT_LOOP(ip + 1), // skip copying instructions
                    )?;
                }
                Ctrl(OP_IF(ty, block_instructions_branch)) => {
                    debug!("OP_IF {:?}", ty);
                    let element = self.store.stack.pop();
                    debug!("Popping value {:?}", element);
                    if let Some(StackContent::Value(Value::I32(v))) = element {
                        //let (arity, args) = self.get_block_params(&ty)?;
                        let arity = self.get_block_ty_arity(&ty)?;

                        //TODO do something with the args

                        if v != 0 {
                            debug!("C is not zero, therefore branching");

                            let label = Label {
                                arity: arity as u32,
                                ip_before: ip,
                                ip_after: ip,
                            };

                            self.enter_block(
                                label,
                                fr,
                                instructions,
                                &block_instructions_branch,
                                ip,
                                Instruction::EXIT_BLOCK,
                            )?;
                        } else {
                            debug!("C is zero, therefore not branching");
                        }
                    } else {
                        panic!("Value must be i32.const. Instead {:#?}", element);
                    }
                }
                Ctrl(OP_IF_AND_ELSE(
                    ty,
                    block_instructions_branch_1,
                    block_instructions_branch_2,
                )) => {
                    debug!("OP_IF_AND_ELSE {:?}", ty);
                    if let Some(StackContent::Value(Value::I32(v))) = self.store.stack.pop() {
                        //let label_idx = self.get_label_count()?;
                        //let (arity, args) = self.get_block_params(&ty)?;
                        let arity = self.get_block_ty_arity(&ty)?;

                        //TODO do something with the args

                        let label = Label {
                            arity: arity as u32,
                            ip_before: ip,
                            ip_after: ip,
                        };

                        if v != 0 {
                            debug!("C is not zero, therefore branching (1)");

                            self.enter_block(
                                label,
                                fr,
                                instructions,
                                &block_instructions_branch_1,
                                ip,
                                Instruction::EXIT_BLOCK,
                            )?;
                        } else {
                            debug!("C is zero, therefore branching (2)");

                            self.enter_block(
                                label,
                                fr,
                                instructions,
                                &block_instructions_branch_2,
                                ip,
                                Instruction::EXIT_BLOCK,
                            )?;
                        }
                    } else {
                        panic!("Value must be i32.const");
                    }
                }
                Ctrl(OP_BR(label_idx)) => {
                    debug!("OP_BR {}", label_idx);
                    self.do_branch(label_idx, &mut ip)?;
                }
                Ctrl(OP_BR_IF(label_idx)) => {
                    debug!("OP_BR_IF {}", label_idx);
                    if let Some(StackContent::Value(Value::I32(c))) = self.store.stack.pop() {
                        debug!("c is {}", c);
                        if c != 0 {
                            debug!("Branching to {}", label_idx);
                            self.do_branch(label_idx, &mut ip)?;
                        } else {
                            debug!("Not Branching to {}", label_idx);
                        }
                    }
                }
                Ctrl(OP_CALL(idx)) => {
                    debug!("OP_CALL {:?}", idx);

                    let t = self.module.borrow().fn_types[idx as usize].clone();
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

                    self.invoke_function(idx, args);
                }
                Ctrl(OP_RETURN) | Ctrl(OP_END) => {
                    debug!("Return");
                    return Ok(());
                }
                Ctrl(OP_NOP) => {}
                EXIT_BLOCK => {
                    debug!("EXIT_BLOCK");
                    self.exit_block()?;
                }
                REPEAT_LOOP(ip_before) => {
                    debug!("REPEAT_LOOP");
                    ip = ip_before;
                    debug!("Iterating to ip {}", ip);

                    continue;
                }
                x => panic!("Instruction {:?} not implemented", x),
            }
            ip += 1;

            debug!("ip is now {}", ip);

            debug!("stack {:#?}", self.store.stack);
        }

        Ok(())
    }

    /// Get the frame at the top of the stack
    fn get_frame(&mut self) -> Frame {
        debug!("get_frame");
        match self.store.stack.pop() {
            Some(Frame(fr)) => fr,
            Some(x) => panic!("Expected frame but found {:?}", x),
            None => panic!("Empty stack on function call"),
        }
    }

    fn enter_block(
        &mut self,
        l: Label,
        _frame: &mut Frame,
        instructions: &mut Vec<Instruction>,
        block: &[Instruction],
        pc: usize,
        terminator_instruction: Instruction,
    ) -> Result<(), InstructionError> {
        debug!("enter block");

        debug!("instructions before {:?}", instructions);

        let mut v = instructions.split_off(pc + 1);
        instructions.extend_from_slice(block);
        instructions.push(terminator_instruction);
        instructions.append(&mut v);

        debug!("instructions after {:?}", instructions);

        debug!("Adjusting stack");
        debug!("Stack before {:#?}", self.store.stack);
        self.store.stack.push(StackContent::Label(l));
        self.store
            .stack
            .iter_mut()
            .filter_map(|w| match w {
                StackContent::Label(v) => Some(v),
                _ => None,
            })
            .for_each(|w| w.ip_after += block.len() + 1);

        //l.ip_after += 1;

        debug!("Stack after {:#?}", self.store.stack);

        Ok(())

        //self.run_instructions(frame, block)
    }

    fn exit_block(&mut self) -> Result<(), InstructionError> {
        debug!("exit_block");

        let mut val_m = Vec::new();

        while let Some(Value(_v)) = self.store.stack.last() {
            val_m.push(self.store.stack.pop().unwrap());
        }

        debug!("values {:?}", val_m);

        assert!(self
            .store
            .stack
            .pop()
            .expect("Expected Label, but found nothing")
            .is_label());

        self.store.stack.append(&mut val_m);

        Ok(())
    }

    fn do_branch(&mut self, label_idx: u32, ip: &mut usize) -> Result<(), InstructionError> {
        debug!("do_branch {}", label_idx);
        let labels = self.get_labels()?.iter().copied().collect::<Vec<_>>();
        let labels_len = labels.len();

        assert!(label_idx + 1 <= labels_len as u32);

        // Get the last label + label_idx
        let label = labels
            .get(labels.len() - 1 - label_idx as usize)
            .expect("No label found");

        debug!("label {:?}", label);

        *ip = label.ip_after;
        debug!("Iterating to {}", ip);

        let content = self.get_content_from_stack(label.arity)?;
        debug!("content {:?}", content);
        for i in content.iter() {
            if let StackContent::Value(_) = i {
                // ok
            } else {
                panic!("Expected value");
            }
        }

        debug!("Range {:?}", 0..label_idx);
        for _ in 0..(label_idx + 1) {
            while let Some(StackContent::Value(_)) = self.store.stack.last() {
                let k = self.store.stack.pop();
                debug!("Popping value {:?}", k);
            }

            if let Some(StackContent::Label(_)) = self.store.stack.last() {
                let k = self.store.stack.pop();
                debug!("Popping label {:?}", k);
            } else {
                panic!("Expected label");
            }
        }

        self.store.stack.extend(content);

        Ok(())
    }

    /// Gets the labels of the stack
    fn get_labels<'a>(&'a self) -> Result<Vec<&'a Label>, InstructionError> {
        Ok(self
            .store
            .stack
            .iter()
            .filter_map(|w| {
                if let StackContent::Label(x) = w {
                    Some(x)
                } else {
                    None
                }
            })
            .collect::<Vec<_>>())
    }

    /// Pops `arity` times off the stack and returns it
    fn get_content_from_stack(
        &mut self,
        arity: u32,
    ) -> Result<Vec<StackContent>, InstructionError> {
        let mut v = Vec::with_capacity(arity as usize);
        for _ in 0..arity {
            v.push(self.store.stack.pop().expect("Not expecting None"));
        }

        Ok(v)
    }

    fn get_block_ty_arity(&mut self, block_ty: &BlockType) -> Result<usize, InstructionError> {
        Ok(match block_ty {
            BlockType::Empty => 0,
            BlockType::ValueType(_) => 1,
            BlockType::ValueTypeTy(ty) => self
                .module
                .borrow()
                .fn_types
                .get(*ty as usize)
                .ok_or(InstructionError::Trap)?
                .return_types
                .len(),
        })
    }

    /*
    fn get_block_params(
        &mut self,
        block_ty: &BlockType,
    ) -> Result<(usize, Vec<Value>), InstructionError> {
        let (arity, args) = match block_ty {
            BlockType::Empty => (0, vec![]),
            BlockType::ValueType(v) => (1, vec![self.store.stack.pop()]),
            BlockType::ValueTypeTy(ty) => {
                let m = self
                    .module
                    .borrow()
                    .fn_types
                    .get(*ty as usize)
                    .ok_or(InstructionError::Trap)?
                    .param_types
                    .len();

                let n = self
                    .module
                    .borrow()
                    .fn_types
                    .get(*ty as usize)
                    .ok_or(InstructionError::Trap)?
                    .return_types
                    .len();

                let mut v = Vec::with_capacity(m);
                for _ in 0..m {
                    v.push(self.store.stack.pop());
                }

                (n, v)
            }
        };

        debug!("args {:#?}", args);

        Ok((
            arity,
            args.iter()
                .map(|w| match w.as_ref().expect("Cannot be None") {
                    StackContent::Value(v) => v,
                    _ => panic!("Something was messed up"),
                })
                .collect(),
        ))
    }
    */
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

        let body = FunctionBody {
            locals: vec![],
            code: vec![Var(OP_LOCAL_GET(0)), Var(OP_LOCAL_GET(1)), Num(OP_I32_ADD)],
        };

        // We have 2 parameters, but supply 3
        e.store.funcs = vec![FuncInstance {
            ty: FunctionSignature {
                param_types: vec![ValueType::I32, ValueType::I32],
                return_types: vec![],
            },
            module: Rc::downgrade(&e.module),
            code: body.clone(),
        }];

        e.module.borrow_mut().code = vec![body.clone()];

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
    fn test_function_block() {
        let mut e = empty_engine();
        e.store.stack = vec![Frame(Frame {
            arity: 1,
            locals: vec![I32(1), I32(1)],
            module_instance: e.downgrade_mod_instance(),
        })];
        e.module.borrow_mut().code = vec![FunctionBody {
            locals: vec![],
            code: vec![Ctrl(OP_BLOCK(
                BlockType::ValueType(ValueType::I32),
                vec![Var(OP_LOCAL_GET(0)), Var(OP_LOCAL_GET(1)), Num(OP_I32_ADD)],
            ))],
        }];
        e.run_function(0);
        assert_eq!(Value(I32(2)), e.store.stack.pop().unwrap());
    }

    #[test]
    fn test_function_block_br() {
        let mut e = empty_engine();

        let code = vec![Ctrl(OP_BLOCK(
            BlockType::Empty,
            vec![Ctrl(OP_BLOCK(BlockType::Empty, vec![Ctrl(OP_BR(1))]))],
        ))];

        e.store.stack = vec![Frame(Frame {
            arity: 0,
            locals: vec![],
            module_instance: e.downgrade_mod_instance(),
        })];

        e.module.borrow_mut().code = vec![FunctionBody {
            locals: vec![],
            code: code,
        }];
        e.run_function(0);
        assert_eq!(None, e.store.stack.pop());
    }

    #[test]
    fn test_function_block_br_deep() {
        let mut e = empty_engine();

        //env_logger::init();
        let code = vec![Ctrl(OP_BLOCK(
            BlockType::Empty,
            vec![Ctrl(OP_BLOCK(
                BlockType::Empty,
                vec![Ctrl(OP_BLOCK(BlockType::Empty, vec![Ctrl(OP_BR(2))]))],
            ))],
        ))];

        e.store.stack = vec![Frame(Frame {
            arity: 0,
            locals: vec![],
            module_instance: e.downgrade_mod_instance(),
        })];

        e.module.borrow_mut().code = vec![FunctionBody {
            locals: vec![],
            code,
        }];
        e.run_function(0);
        assert_eq!(None, e.store.stack.pop());
    }

    //#[test]
    fn test_function_if() {
        let mut e = empty_engine();
        e.store.stack = vec![
            Value(Value::I32(1)),
            Frame(Frame {
                arity: 1,
                locals: vec![I32(1), I32(1)], //arguments for LOCAL_GET
                module_instance: e.downgrade_mod_instance(),
            }),
        ];
        e.module.borrow_mut().code = vec![FunctionBody {
            locals: vec![],
            code: vec![Ctrl(OP_IF(
                BlockType::ValueType(ValueType::I32),
                vec![Var(OP_LOCAL_GET(0)), Var(OP_LOCAL_GET(1)), Num(OP_I32_ADD)],
            ))],
        }];
        e.run_function(0);
        assert_eq!(Value(I32(2)), e.store.stack.pop().unwrap());
    }

    //#[test]
    fn test_function_if_false() {
        let mut e = empty_engine();
        e.store.stack = vec![
            Value(Value::I32(0)), //THIS CHANGED
            Frame(Frame {
                arity: 1,
                locals: vec![I32(1), I32(1)], //arguments for LOCAL_GET
                module_instance: e.downgrade_mod_instance(),
            }),
        ];
        e.module.borrow_mut().code = vec![FunctionBody {
            locals: vec![],
            code: vec![Ctrl(OP_IF(
                BlockType::ValueType(ValueType::I32),
                vec![Var(OP_LOCAL_GET(0)), Var(OP_LOCAL_GET(1)), Num(OP_I32_ADD)],
            ))],
        }];
        e.run_function(0);
        assert_eq!(None, e.store.stack.pop());
    }

    //#[test]
    fn test_function_if_else_1() {
        let mut e = empty_engine();
        e.store.stack = vec![
            Value(Value::I32(1)),
            Frame(Frame {
                arity: 1,
                locals: vec![I32(1), I32(1)], //arguments for LOCAL_GET
                module_instance: e.downgrade_mod_instance(),
            }),
        ];
        e.module.borrow_mut().code = vec![FunctionBody {
            locals: vec![],
            code: vec![Ctrl(OP_IF_AND_ELSE(
                BlockType::ValueType(ValueType::I32),
                vec![Var(OP_LOCAL_GET(0)), Var(OP_LOCAL_GET(1)), Num(OP_I32_ADD)],
                vec![Num(OP_I32_CONST(-1000))],
            ))],
        }];
        e.run_function(0);
        assert_eq!(
            Some(StackContent::Value(Value::I32(2))),
            e.store.stack.pop()
        );
    }

    //#[test]
    fn test_function_if_else_2() {
        let mut e = empty_engine();
        e.store.stack = vec![
            Value(Value::I32(0)), //changed
            Frame(Frame {
                arity: 1,
                locals: vec![I32(1), I32(1)], //arguments for LOCAL_GET
                module_instance: e.downgrade_mod_instance(),
            }),
        ];
        e.module.borrow_mut().code = vec![FunctionBody {
            locals: vec![],
            code: vec![Ctrl(OP_IF_AND_ELSE(
                BlockType::ValueType(ValueType::I32),
                vec![Var(OP_LOCAL_GET(0)), Var(OP_LOCAL_GET(1)), Num(OP_I32_ADD)],
                vec![Num(OP_I32_CONST(-1000))],
            ))],
        }];
        e.run_function(0);
        assert_eq!(
            Some(StackContent::Value(Value::I32(-1000))),
            e.store.stack.pop()
        );
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
        assert_eq!(I32(2), e.store.globals[0].val);
    }
}
