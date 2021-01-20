#![allow(dead_code)]

pub(crate) mod export;
pub(crate) mod func;
pub(crate) mod memory;
pub mod module;
mod op;
pub(crate) mod prelude;
pub mod stack;
pub(crate) mod store;
pub(crate) mod table;

use self::stack::StackContent;
use self::stack::StackContent::*;
use self::stack::{Frame, Label};
use self::store::Store;
use crate::convert;
pub use crate::debugger::BorrowedProgramState;
pub use crate::debugger::{ProgramCounter, RelativeProgramCounter};
use crate::engine::func::FuncInstance;
use crate::engine::module::ModuleInstance;
use crate::engine::table::TableInstance;
use crate::operations::*;
pub use crate::page::Page;
use crate::value::{Arity, Value, Value::*};
pub use crate::PAGE_SIZE;
pub use anyhow::{anyhow, Context, Result};
pub use wasm_parser::core::Instruction::*;
pub use wasm_parser::core::*;
pub use wasm_parser::Module;

#[derive(Debug)]
pub struct Engine {
    pub module: ModuleInstance, //TODO rename to `module_instance`
    pub started: bool,
    pub store: Store,
    debugger: Box<dyn ProgramCounter>,
}

#[derive(Debug)]
pub enum InstructionOutcome {
    EXIT,
    BRANCH(u32),
    RETURN,
}

#[allow(dead_code)]
pub(crate) fn empty_engine() -> Engine {
    let mi = ModuleInstance {
        start: 0,
        code: Vec::new(),
        fn_types: Vec::new(),
        funcaddrs: Vec::new(),
        tableaddrs: Vec::new(),
        memaddrs: Vec::new(),
        globaladdrs: Vec::new(),
        exports: Vec::new(),
    };

    Engine {
        started: true,
        store: Store {
            funcs: Vec::new(),
            tables: Vec::new(),
            globals: Vec::new(),
            memory: Vec::new(),
            stack: vec![StackContent::Frame(Frame {
                arity: 0,
                locals: Vec::new(),
            })],
        },
        module: mi,
        debugger: Box::new(RelativeProgramCounter::default()),
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct Variable {
    pub mutable: bool, //Actually, there is a `Mut` enum. TODO check if makes sense to use it
    pub val: Value,
}

#[macro_export]
macro_rules! fetch_unop {
    ($stack: expr) => {{
        debug!("Popping {:?}", $stack.last());
        let v1 = match $stack.pop().unwrap() {
            Value(v) => v,
            x => panic!("Top of stack was not a value, but instead {:?}", x),
        };
        (v1)
    }};
}

#[macro_export]
macro_rules! fetch_binop {
    ($stack: expr) => {{
        use crate::fetch_unop;

        let v1 = fetch_unop!($stack);
        let v2 = fetch_unop!($stack);

        (v1, v2)
    }};
}

macro_rules! load_memory {
    ($self:expr, $arg:expr, $size:expr, $ty:ty, $variant:expr) => {
        let v1 = fetch_unop!($self.store.stack);

        if let I32(v) = v1 {
            let ea = (v + $arg.offset as i32) as usize;

            let module = &$self.module;

            let addr = module.memaddrs.get(0).expect("No memory address found");

            let instance = &$self.store.memory[*addr as usize];

            debug!("instance {:?}", instance);
            debug!("Range {:?}", ea..ea + $size);
            //debug!("part {:?}", &instance.data[ea..ea + $size]);
            let mut b = vec![0; $size];
            b.copy_from_slice(&instance.data[ea..ea + $size]);
            assert!(
                b.len() == $size,
                "Size of bytes sequence is expected to match size"
            );

            unsafe {
                //Convert [u8] to [number]
                let c = &*(b.as_slice() as *const [u8] as *const [$ty]);
                debug!("c is {:?}", c);

                $self.store.stack.push(StackContent::Value($variant(c[0])));
            }
        } else {
            panic!("Expected I32, found something else");
        }
    };
}

macro_rules! load_memorySX {
    ($self:expr, $arg:expr, $size:expr, $ty:ty, $variant:expr, $cast_ty:ty) => {
        let v1 = fetch_unop!($self.store.stack);

        if let I32(v) = v1 {
            let ea = (v + $arg.offset as i32) as usize;

            let module = &$self.module;

            let addr = module.memaddrs.get(0).expect("No memory address found");

            let instance = &$self.store.memory[*addr as usize];

            debug!("instance {:?}", instance);
            debug!("Range {:?}", ea..ea + $size);
            debug!("part {:?}", &instance.data[ea..ea + $size]);

            let mut b = vec![0; $size];
            b.copy_from_slice(&instance.data[ea..ea + $size]);
            assert!(b.len() == $size);

            debug!("b {:?}", b);

            unsafe {
                // Convert [u8] to [number]
                let c = &*(b.as_slice() as *const [u8] as *const [$ty]);
                debug!("c is {:?}", c);

                // THIS IS DIFFERENT THAN `load_memory!`

                let c2 = c[0] as $cast_ty;

                $self
                    .store
                    .stack
                    .push(StackContent::Value($variant(c2.into())));

                // END
            }
        } else {
            panic!("Expected I32, found something else");
        }
    };
}

macro_rules! store_memory {
    ($self:expr, $arg:expr, $size:expr, $ty:ty, $variant:ident) => {
        let k = fetch_unop!($self.store.stack);
        let v1 = fetch_unop!($self.store.stack);

        if let $variant(t) = k {
            if let I32(v) = v1 {
                let ea = (v + $arg.offset as i32) as usize;

                let module = &$self.module;

                let addr = module.memaddrs.get(0).expect("No memory address found");

                let instance = &mut $self.store.memory[*addr as usize];

                let mut bytes = t.to_le_bytes();

                instance.data[ea..ea + $size].swap_with_slice(&mut bytes);
            } else {
                panic!("Expected I32, found something else");
            }
        } else {
            panic!("Expected a different value on the stack");
        }
    };
}

macro_rules! store_memoryN {
    ($self:expr, $arg:expr, $size:expr, $ty:ty, $variant:ident, $new_ty:ty, $N:expr) => {
        let k = fetch_unop!($self.store.stack);
        let v1 = fetch_unop!($self.store.stack);

        if let $variant(t) = k {
            if let I32(v) = v1 {
                let ea = (v + $arg.offset as i32) as usize;

                let module = &$self.module;

                let addr = module.memaddrs.get(0).expect("No memory address found");

                let instance = &mut $self.store.memory[*addr as usize];

                if instance.data.len() < ea + ($N / 8) {
                    panic!("Offset is corrupt");
                }

                let mut bytes = t.to_le_bytes();

                instance.data[ea..ea + $size].swap_with_slice(&mut bytes[0..($N / 8)]);
            } else {
                panic!("Expected I32, found something else");
            }
        } else {
            panic!("Expected a different value on the stack");
        }
    };
}

impl Engine {
    pub fn new(mi: ModuleInstance, module: &Module, debugger: Box<dyn ProgramCounter>) -> Engine {
        let mut e = Engine {
            module: mi,
            started: false,
            store: Store {
                funcs: Vec::new(),
                tables: Vec::new(),
                stack: Vec::new(),
                globals: Vec::new(),
                memory: Vec::new(),
            },
            debugger,
        };

        debug!("before allocate {:#?}", e);
        e.allocate(module);
        debug!("after allocate {:#?}", e);

        e
    }

    /// Initializes `n` pages in memory
    pub(crate) fn init_memory(&mut self, n: usize) -> Result<()> {
        let res = self
            .store
            .init_memory(n)
            .context("Trying to initializean empty memory instance");

        // only one memory module is allowed
        assert!(self.module.memaddrs.is_empty());
        self.module.memaddrs.push(self.module.memaddrs.len() as u32);

        res
    }

    fn allocate(&mut self, m: &Module) {
        info!("Allocation");
        crate::allocation::allocate(m, &mut self.module, &mut self.store)
            .expect("Allocation failed");
    }

    pub fn instantiation(&mut self, m: &Module) -> Result<()> {
        info!("Instantiation");
        let start_function = crate::instantiation::instantiation(m, &self.module, &mut self.store)?;

        if let Some(func_addr) = start_function {
            debug!("Invoking start function with {:?}", func_addr);
            self.invoke_function(func_addr, vec![])?;
        }

        Ok(())
    }

    /// Get a global value
    pub fn get(&mut self, name: &str) -> Result<Value> {
        debug!("get global for {:?}", name);

        let module = &self.module.exports;

        let filtered: Vec<_> = module.iter().filter(|x| x.name == name).take(1).collect();

        let export_instance = filtered
            .get(0)
            .context("Exported function not found or found something else")?;

        debug!("Exports {:#?}", export_instance);

        match export_instance.value {
            ExternalKindType::Global { ty } => {
                let global_addr = *self
                    .module
                    .globaladdrs
                    .get(ty as usize)
                    .ok_or_else(|| anyhow!("Global not found"))?;

                return Ok(self
                    .store
                    .globals
                    .get(global_addr as usize)
                    .ok_or_else(|| anyhow!("Global not found in the store"))?
                    .val);
            }
            _ => Err(anyhow!("Exported global not found")),
        }
    }

    /// Take only exported functions into consideration
    /// `idx` is the id of the export instance, not the function
    pub fn invoke_exported_function(&mut self, idx: u32, args: Vec<Value>) -> Result<()> {
        debug!("invoke_exported_function {} with args {:?}", idx, args);
        let k = {
            let x = &self.module;

            debug!("the export instance is {:?}", x.exports.get(idx as usize));

            let w = x
                .exports
                .get(idx as usize)
                .context("Exported function not found or found something else")?;

            w.value
        };

        debug!("Exports {:#?}", k);

        match k {
            ExternalKindType::Function { ty } => {
                let func_addr = *self
                    .module
                    .funcaddrs
                    .get(ty as usize)
                    .ok_or_else(|| anyhow!("Function not found"))?;

                self.invoke_function(func_addr, args)
                    .context("Invoking the function failed")?;
            }
            _ => {
                return Err(anyhow!("Exported function not found"));
            }
        }

        Ok(())
    }

    pub fn invoke_exported_function_by_name(&mut self, name: &str, args: Vec<Value>) -> Result<()> {
        debug!(
            "Invoking exporting function by name {} with args {:?}",
            name, args
        );
        let idx = self
            .module
            .exports
            .iter()
            .position(|e| e.name == name)
            .context("Cannot find function in exports")?;

        debug!("=> Export instance found");
        debug!("The function addr is {}", idx);

        self.invoke_exported_function(idx as u32, args)
            .context("Invoking the exported function failed")?;

        Ok(())
    }

    pub(crate) fn invoke_function(&mut self, idx: u32, args: Vec<Value>) -> Result<()> {
        self.check_parameters_of_function(idx, &args);

        let t = &self.store.funcs[idx as usize].ty;

        let typed_locals = &self.store.funcs[idx as usize].code.locals;

        debug!("defined locals are {:#?}", typed_locals);

        let mut locals = args;

        // All parameters are `locals`, but
        // we can additionaly define more of them.
        // This is done in the function definition of locals
        // It is very important to use the correct type
        for i in 0..typed_locals.len() {
            let entry = typed_locals.get(i).unwrap();

            // There is a count property to define multiple at once
            for _ in 0..entry.count {
                match entry.ty {
                    ValueType::I32 => locals.push(I32(0)),
                    ValueType::I64 => locals.push(I64(0)),
                    ValueType::F32 => locals.push(F32(0.0)),
                    ValueType::F64 => locals.push(F64(0.0)),
                }
            }
        }

        self.store.stack.push(Frame(Frame {
            arity: t.return_types.len() as u32,
            locals,
            //module_instance: Rc::downgrade(&self.module),
        }));

        trace!("stack before invoking {:#?}", self.store.stack);

        debug!("Invoking function");
        self.run_function(idx)
            .with_context(|| format!("Function with id {} failed", idx))?;

        Ok(())
    }

    fn check_parameters_of_function(&self, idx: u32, args: &[Value]) {
        let fn_types = self
            .store
            .funcs
            .get(idx as usize)
            .expect("Function not found")
            .ty
            .param_types
            .iter();

        let argtypes = args.iter().map(|w| match *w {
            Value::I32(_) => ValueType::I32,
            Value::I64(_) => ValueType::I64,
            Value::F32(_) => ValueType::F32,
            Value::F64(_) => ValueType::F64,
        });

        let len_1 = fn_types.len();
        let len_2 = argtypes.len();

        // Check if `fn_types` and `argtypes` are elementwise equal
        let is_same = fn_types.zip(argtypes).map(|(x, y)| *x == y).all(|w| w);

        if !is_same || len_1 != len_2 {
            panic!("Function expected different parameters!");
        }
    }

    #[allow(clippy::cognitive_complexity)]
    pub(crate) fn run_function(&mut self, idx: u32) -> Result<()> {
        debug!("Running function {:?}", idx);

        //FIXME this `.clone` is extremly expensive!!!
        // But there is a problem
        // Because, we iterate over the borrowed iterator,
        // we cannot easily run the block
        let f = &self.module.code[idx as usize].clone();

        let mut fr = self.get_frame()?;

        debug!("frame {:#?}", fr);

        self.run_instructions(&mut fr, &mut f.code.iter())
            .with_context(|| format!("Running instructions for function {}", idx))?;

        // implicit return
        debug!("Implicit return (arity {:?})", fr.arity);

        debug!("Stack before function return {:#?}", self.store.stack);

        let mut ret = Vec::new();
        for _ in 0..fr.arity {
            debug!("Popping {:?}", self.store.stack.last());
            match self.store.stack.pop() {
                Some(Value(v)) => ret.push(Value(v)),
                Some(x) => {
                    return Err(anyhow!("Expected value but found {:?}", x));
                }
                None => {} //None => panic!("Unexpected empty stack!"),
            }
        }

        debug!("Popping frames");
        while let Some(Frame(_)) = self.store.stack.last() {
            debug!("Removing {:?}", self.store.stack.last());
            self.store.stack.pop();
        }

        while let Some(val) = ret.pop() {
            self.store.stack.push(val);
        }

        debug!("Stack after function return {:#?}", self.store.stack);

        Ok(())
    }

    #[allow(clippy::cognitive_complexity)]
    fn run_instructions<'a>(
        &mut self,
        fr: &mut Frame,
        instruction_wrapper: &'a mut impl std::iter::Iterator<Item = &'a InstructionWrapper>,
    ) -> Result<InstructionOutcome> {
        //let mut ip = 0;
        for wrapped_instruction in instruction_wrapper {
            self.debugger
                .set_pc(BorrowedProgramState::new(
                    wrapped_instruction.get_id(),
                    &self.store.stack,
                    &fr.locals,
                ))
                .context("Setting program state failed")?;

            let instruction = wrapped_instruction.get_instruction();
            debug!("Evaluating instruction {:?}", instruction);

            match &instruction {
                OP_LOCAL_GET(idx) => {
                    self.local_get(idx, fr)
                        .with_context(|| format!("OP_LOCAL_GET({})", idx))?;
                }
                OP_LOCAL_SET(idx) => {
                    self.local_set(idx, fr)
                        .with_context(|| format!("OP_LOCAL_SET({})", idx))?;
                }
                OP_LOCAL_TEE(idx) => {
                    self.local_tee(idx, fr)
                        .with_context(|| format!("OP_LOCAL_TEE({})", idx))?;
                }
                OP_GLOBAL_GET(idx) => {
                    self.global_get(idx, fr)
                        .with_context(|| format!("OP_GLOBAL_GET({})", idx))?;
                }
                OP_GLOBAL_SET(idx) => {
                    self.global_set(idx, fr)
                        .with_context(|| format!("OP_GLOBAL_SET({})", idx))?;
                }
                OP_I32_CONST(v) => {
                    debug!("OP_I32_CONST: pushing {} to stack", v);
                    self.store.stack.push(Value(I32(*v)));
                    debug!("stack {:#?}", self.store.stack);
                }
                OP_I64_CONST(v) => {
                    debug!("OP_I64_CONST: pushing {} to stack", v);
                    self.store.stack.push(Value(I64(*v)))
                }
                OP_F32_CONST(v) => {
                    debug!("OP_F32_CONST: pushing {} to stack", v);
                    self.store.stack.push(Value(F32(*v)))
                }
                OP_F64_CONST(v) => {
                    debug!("OP_F64_CONST: pushing {} to stack", v);
                    self.store.stack.push(Value(F64(*v)))
                }
                OP_F32_COPYSIGN => {
                    let (z1, z2) = fetch_binop!(self.store.stack);
                    self.store.stack.push(Value(copysign(z1, z2)))
                }
                OP_F64_COPYSIGN => {
                    let (z1, z2) = fetch_binop!(self.store.stack);
                    self.store.stack.push(Value(copysign(z1, z2)))
                }
                OP_I32_ADD | OP_I64_ADD | OP_F32_ADD | OP_F64_ADD => {
                    let (v2, v1) = fetch_binop!(self.store.stack);
                    self.store.stack.push(Value(v1 + v2))
                }
                OP_I32_SUB | OP_I64_SUB | OP_F32_SUB | OP_F64_SUB => {
                    let (v2, v1) = fetch_binop!(self.store.stack);
                    self.store.stack.push(Value(v1 - v2))
                }
                OP_I32_MUL | OP_I64_MUL | OP_F32_MUL | OP_F64_MUL => {
                    let (v2, v1) = fetch_binop!(self.store.stack);
                    self.store.stack.push(Value(v1 * v2))
                }
                OP_I32_DIV_S | OP_I64_DIV_S | OP_F32_DIV | OP_F64_DIV => {
                    let (v2, v1) = fetch_binop!(self.store.stack);
                    self.store.stack.push(Value(v1 / v2))
                }
                OP_I32_DIV_U | OP_I64_DIV_U => {
                    let (v2, v1) = fetch_binop!(self.store.stack);
                    match (v1, v2) {
                        (I32(x1), I32(x2)) => self
                            .store
                            .stack
                            .push(Value(I32(((x1 as u32) / (x2 as u32)) as i32))),
                        (I64(x1), I64(x2)) => self
                            .store
                            .stack
                            .push(Value(I64(((x1 as u64) / (x2 as u64)) as i64))),
                        _ => return Err(anyhow!("Invalid types for DIV_U")),
                    }
                }
                OP_I32_REM_S | OP_I64_REM_S => {
                    let (v2, v1) = fetch_binop!(self.store.stack);
                    self.store.stack.push(Value(v1 % v2))
                }
                OP_I32_REM_U | OP_I64_REM_U => {
                    let (v2, v1) = fetch_binop!(self.store.stack);
                    match (v1, v2) {
                        (I32(x1), I32(x2)) => self
                            .store
                            .stack
                            .push(Value(I32(((x1 as u32) % (x2 as u32)) as i32))),
                        (I64(x1), I64(x2)) => self
                            .store
                            .stack
                            .push(Value(I64(((x1 as u64) % (x2 as u64)) as i64))),
                        _ => panic!("Invalid types for REM_U"),
                    }
                }
                OP_I32_AND | OP_I64_AND => {
                    let (v2, v1) = fetch_binop!(self.store.stack);
                    self.store.stack.push(Value(v1 & v2))
                }
                OP_I32_OR | OP_I64_OR => {
                    let (v2, v1) = fetch_binop!(self.store.stack);
                    self.store.stack.push(Value(v1 | v2))
                }
                OP_I32_XOR | OP_I64_XOR => {
                    let (v2, v1) = fetch_binop!(self.store.stack);
                    self.store.stack.push(Value(v1 ^ v2))
                }
                OP_I32_SHL | OP_I64_SHL => {
                    let (v2, v1) = fetch_binop!(self.store.stack);
                    self.store.stack.push(Value(v1 << v2))
                }
                OP_I32_SHR_S | OP_I64_SHR_S => {
                    let (v2, v1) = fetch_binop!(self.store.stack);
                    self.store.stack.push(Value(v1 >> v2))
                }
                OP_I32_SHR_U | OP_I64_SHR_U => {
                    let (v2, v1) = fetch_binop!(self.store.stack);
                    match (v1, v2) {
                        (I32(x1), I32(x2)) => {
                            let k = x2 as u32 % 32;
                            self.store
                                .stack
                                .push(Value(I32(((x1 as u32).checked_shr(k)).unwrap_or(0) as i32)));
                        }
                        (I64(x1), I64(x2)) => {
                            let k = x2 as u64 % 64;
                            self.store
                                .stack
                                .push(Value(I64(
                                    ((x1 as u64).checked_shr(k as u32)).unwrap_or(0) as i64
                                )));
                        }
                        _ => return Err(anyhow!("Invalid types for SHR_U")),
                    }
                }
                OP_I32_ROTL | OP_I64_ROTL => {
                    let (v2, v1) = fetch_binop!(self.store.stack);
                    self.store.stack.push(Value(rotate_left(v1, v2)))
                }
                OP_I32_ROTR | OP_I64_ROTR => {
                    let (v2, v1) = fetch_binop!(self.store.stack);
                    self.store.stack.push(Value(rotate_right(v1, v2)))
                }
                OP_I32_CLZ | OP_I64_CLZ => {
                    let v1 = fetch_unop!(self.store.stack);
                    self.store.stack.push(Value(leading_zeros(v1)))
                }
                OP_I32_CTZ | OP_I64_CTZ => {
                    let v1 = fetch_unop!(self.store.stack);
                    self.store.stack.push(Value(trailing_zeros(v1)))
                }
                OP_I32_POPCNT | OP_I64_POPCNT => {
                    let v1 = fetch_unop!(self.store.stack);
                    self.store.stack.push(Value(count_ones(v1)))
                }
                OP_I32_EQZ | OP_I64_EQZ => {
                    let v1 = fetch_unop!(self.store.stack);

                    self.store.stack.push(Value(eqz(v1)))
                }
                OP_I32_EQ | OP_I64_EQ | OP_F32_EQ | OP_F64_EQ => {
                    let (v1, v2) = fetch_binop!(self.store.stack);
                    let res = v1 == v2;

                    if res {
                        self.store.stack.push(StackContent::Value(Value::I32(1)))
                    } else {
                        self.store.stack.push(StackContent::Value(Value::I32(0)))
                    }
                }
                OP_I32_NE | OP_I64_NE | OP_F32_NE | OP_F64_NE => {
                    let (v1, v2) = fetch_binop!(self.store.stack);
                    let res = v1 != v2;

                    if res {
                        self.store.stack.push(StackContent::Value(Value::I32(1)))
                    } else {
                        self.store.stack.push(StackContent::Value(Value::I32(0)))
                    }
                }
                OP_I32_LT_S | OP_I64_LT_S | OP_F32_LT | OP_F64_LT => {
                    // switch ordering because of stack layout
                    let (v2, v1) = fetch_binop!(self.store.stack);
                    self.store
                        .stack
                        .push(Value(lt(v1, v2).convert(ValueType::I32)))
                }
                OP_I32_LT_U | OP_I64_LT_U => {
                    let (v2, v1) = fetch_binop!(self.store.stack);
                    match (v1, v2) {
                        (I32(x1), I32(x2)) => self
                            .store
                            .stack
                            .push(Value(I32(((x1 as u32) < (x2 as u32)) as i32))),
                        (I64(x1), I64(x2)) => self
                            .store
                            .stack
                            .push(Value(I32(((x1 as u64) < (x2 as u64)) as i32))),
                        _ => return Err(anyhow!("Invalid types for LT_U comparison")),
                    }
                }
                OP_I32_GT_S | OP_I64_GT_S | OP_F32_GT | OP_F64_GT => {
                    // switch ordering because of stack layout
                    let (v2, v1) = fetch_binop!(self.store.stack);
                    self.store
                        .stack
                        .push(Value(gt(v1, v2).convert(ValueType::I32)))
                }
                OP_I32_GT_U | OP_I64_GT_U => {
                    let (v2, v1) = fetch_binop!(self.store.stack);
                    match (v1, v2) {
                        (I32(x1), I32(x2)) => self
                            .store
                            .stack
                            .push(Value(I32(((x1 as u32) > (x2 as u32)) as i32))),
                        (I64(x1), I64(x2)) => self
                            .store
                            .stack
                            .push(Value(I32(((x1 as u64) > (x2 as u64)) as i32))),
                        _ => return Err(anyhow!("Invalid types for GT_U comparison")),
                    }
                }
                OP_I32_LE_S | OP_I64_LE_S | OP_F32_LE | OP_F64_LE => {
                    // switch ordering because of stack layout
                    let (v2, v1) = fetch_binop!(self.store.stack);
                    self.store
                        .stack
                        .push(Value(le(v1, v2).convert(ValueType::I32)))
                }
                OP_I32_LE_U | OP_I64_LE_U => {
                    let (v2, v1) = fetch_binop!(self.store.stack);
                    match (v1, v2) {
                        (I32(x1), I32(x2)) => self
                            .store
                            .stack
                            .push(Value(I32(((x1 as u32) <= (x2 as u32)) as i32))),
                        (I64(x1), I64(x2)) => self
                            .store
                            .stack
                            .push(Value(I32(((x1 as u64) <= (x2 as u64)) as i32))),
                        _ => return Err(anyhow!("Invalid types for LE_U comparison")),
                    }
                }
                OP_I32_GE_S | OP_I64_GE_S | OP_F32_GE | OP_F64_GE => {
                    // switch ordering because of stack layout
                    let (v2, v1) = fetch_binop!(self.store.stack);
                    self.store
                        .stack
                        .push(Value(ge(v1, v2).convert(ValueType::I32)))
                }
                OP_I32_GE_U | OP_I64_GE_U => {
                    let (v2, v1) = fetch_binop!(self.store.stack);
                    match (v1, v2) {
                        (I32(x1), I32(x2)) => self
                            .store
                            .stack
                            .push(Value(I32(((x1 as u32) >= (x2 as u32)) as i32))),
                        (I64(x1), I64(x2)) => self
                            .store
                            .stack
                            .push(Value(I32(((x1 as u64) >= (x2 as u64)) as i32))),
                        _ => return Err(anyhow!("Invalid types for GE_U comparison")),
                    }
                }
                OP_F32_ABS | OP_F64_ABS => {
                    let v1 = fetch_unop!(self.store.stack);
                    self.store.stack.push(Value(abs(v1)))
                }
                OP_F32_NEG | OP_F64_NEG => {
                    let v1 = fetch_unop!(self.store.stack);
                    self.store.stack.push(Value(neg(v1)))
                }
                OP_F32_CEIL | OP_F64_CEIL => {
                    let v1 = fetch_unop!(self.store.stack);
                    self.store.stack.push(Value(ceil(v1)))
                }
                OP_F32_FLOOR | OP_F64_FLOOR => {
                    let v1 = fetch_unop!(self.store.stack);
                    self.store.stack.push(Value(floor(v1)))
                }
                OP_F32_TRUNC | OP_F64_TRUNC => {
                    let v1 = fetch_unop!(self.store.stack);
                    self.store.stack.push(Value(trunc(v1)))
                }
                OP_I32_TRUNC_SAT_F32_S | OP_I32_TRUNC_SAT_F64_S => {
                    let v1 = fetch_unop!(self.store.stack);
                    self.store.stack.push(Value(trunc_sat_i32_s(v1)))
                }
                OP_I64_TRUNC_SAT_F32_S | OP_I64_TRUNC_SAT_F64_S => {
                    let v1 = fetch_unop!(self.store.stack);
                    self.store.stack.push(Value(trunc_sat_i64_s(v1)))
                }
                OP_I32_TRUNC_SAT_F32_U => {
                    let v1 = fetch_unop!(self.store.stack);
                    self.store
                        .stack
                        .push(Value(trunc_sat_from_f32_to_i32_u(v1)))
                }
                OP_I32_TRUNC_SAT_F64_U => {
                    let v1 = fetch_unop!(self.store.stack);
                    self.store
                        .stack
                        .push(Value(trunc_sat_from_f64_to_i32_u(v1)))
                }
                OP_I64_TRUNC_SAT_F32_U => {
                    let v1 = fetch_unop!(self.store.stack);
                    self.store
                        .stack
                        .push(Value(trunc_sat_from_f32_to_i64_u(v1)))
                }
                OP_I64_TRUNC_SAT_F64_U => {
                    let v1 = fetch_unop!(self.store.stack);
                    self.store
                        .stack
                        .push(Value(trunc_sat_from_f64_to_i64_u(v1)))
                }
                OP_F32_NEAREST | OP_F64_NEAREST => {
                    let v1 = fetch_unop!(self.store.stack);
                    self.store.stack.push(Value(nearest(v1)))
                }
                OP_F32_SQRT | OP_F64_SQRT => {
                    let v1 = fetch_unop!(self.store.stack);
                    self.store.stack.push(Value(sqrt(v1)))
                }
                OP_F32_MIN | OP_F64_MIN => {
                    let (v1, v2) = fetch_binop!(self.store.stack);
                    self.store.stack.push(Value(min(v1, v2)))
                }
                OP_F32_MAX | OP_F64_MAX => {
                    let (v1, v2) = fetch_binop!(self.store.stack);
                    self.store.stack.push(Value(max(v1, v2)))
                }
                OP_I32_WRAP_I64 => {
                    let v = fetch_unop!(self.store.stack);
                    convert!(self, v, I64, I32, i32);
                }
                OP_I64_EXTEND_I32_S => {
                    let v = fetch_unop!(self.store.stack);
                    convert!(self, v, I32, I64, i64);
                }
                OP_I64_EXTEND_I32_U => {
                    let v = fetch_unop!(self.store.stack);
                    convert!(self, v, I32, I64, i64, u32);
                }
                OP_I64_TRUNC_F32_S => {
                    let v = fetch_unop!(self.store.stack);
                    convert!(self, v, F32, I64, i64);
                }
                OP_I64_TRUNC_F64_S => {
                    let v = fetch_unop!(self.store.stack);
                    convert!(self, v, F64, I64, i64);
                }
                OP_I64_TRUNC_F32_U => {
                    let v = fetch_unop!(self.store.stack);
                    convert!(self, v, F32, I64, i64, u64);
                }
                OP_I64_TRUNC_F64_U => {
                    let v = fetch_unop!(self.store.stack);
                    convert!(self, v, F64, I64, i64, u64);
                }
                OP_I32_TRUNC_F32_S => {
                    let v = fetch_unop!(self.store.stack);
                    convert!(self, v, F32, I32, i32);
                }
                OP_I32_TRUNC_F64_S => {
                    let v = fetch_unop!(self.store.stack);
                    convert!(self, v, F64, I32, i32);
                }
                OP_I32_TRUNC_F32_U => {
                    let v = fetch_unop!(self.store.stack);
                    convert!(self, v, F32, I32, i32, u32);
                }
                OP_I32_TRUNC_F64_U => {
                    let v = fetch_unop!(self.store.stack);
                    convert!(self, v, F64, I32, i32, u32);
                }
                OP_F32_DEMOTE_F64 => {
                    let v = fetch_unop!(self.store.stack);
                    convert!(self, v, F64, F32, f32);
                }
                OP_F64_PROMOTE_F32 => {
                    let v = fetch_unop!(self.store.stack);
                    convert!(self, v, F32, F64, f64);
                }
                OP_F32_CONVERT_I32_S => {
                    let v = fetch_unop!(self.store.stack);
                    convert!(self, v, I32, F32, f32);
                }
                OP_F64_CONVERT_I32_S => {
                    let v = fetch_unop!(self.store.stack);
                    convert!(self, v, I32, F64, f64);
                }
                OP_F32_CONVERT_I64_S => {
                    // Convert I64_S to F32
                    let v = fetch_unop!(self.store.stack);
                    convert!(self, v, I64, F32, f32);
                }
                OP_F64_CONVERT_I64_S => {
                    // Convert I64_S to F64
                    let v = fetch_unop!(self.store.stack);
                    convert!(self, v, I64, F64, f64);
                }
                OP_F32_CONVERT_I32_U => {
                    // Convert I32_S to F32
                    let v = fetch_unop!(self.store.stack);
                    convert!(self, v, I32, F32, f32, u32);
                }
                OP_F64_CONVERT_I32_U => {
                    // Convert I32_S to F64
                    let v = fetch_unop!(self.store.stack);
                    convert!(self, v, I32, F64, f64, u32);
                }
                OP_F32_CONVERT_I64_U => {
                    let v = fetch_unop!(self.store.stack);
                    convert!(self, v, I64, F32, f32, u64);
                }
                OP_F64_CONVERT_I64_U => {
                    let v = fetch_unop!(self.store.stack);
                    convert!(self, v, I64, F64, f64, u64);
                }
                OP_I32_EXTEND8_S => {
                    let v = fetch_unop!(self.store.stack);
                    convert!(self, v, I32, I32, i32, i8);
                }
                OP_I32_EXTEND16_S => {
                    let v = fetch_unop!(self.store.stack);
                    convert!(self, v, I32, I32, i32, i16);
                }
                OP_I64_EXTEND8_S => {
                    let v = fetch_unop!(self.store.stack);
                    convert!(self, v, I64, I64, i64, i8);
                }
                OP_I64_EXTEND16_S => {
                    let v = fetch_unop!(self.store.stack);
                    convert!(self, v, I64, I64, i64, i16);
                }
                OP_I64_EXTEND32_S => {
                    let v = fetch_unop!(self.store.stack);
                    convert!(self, v, I64, I64, i64, i32);
                }
                OP_I32_REINTERPRET_F32
                | OP_I64_REINTERPRET_F64
                | OP_F32_REINTERPRET_I32
                | OP_F64_REINTERPRET_I64 => {
                    let v = fetch_unop!(self.store.stack);
                    self.store.stack.push(Value(reinterpret(v)));
                }
                OP_DROP => {
                    debug!("OP_DROP");
                    let k = self.store.stack.pop();
                    debug!("Dropping {:?}", k);
                }
                OP_SELECT => {
                    self.select()?;
                }
                OP_I32_LOAD_8_u(arg) => {
                    load_memorySX!(self, arg, 4, i32, I32, u8);
                }
                OP_I32_LOAD_16_u(arg) => {
                    load_memorySX!(self, arg, 4, i32, I32, u16);
                }
                OP_I32_LOAD_8_s(arg) => {
                    load_memorySX!(self, arg, 4, i32, I32, i8);
                }
                OP_I32_LOAD_16_s(arg) => {
                    load_memorySX!(self, arg, 4, i32, I32, i16);
                }
                OP_I32_LOAD(arg) => {
                    load_memory!(self, arg, 4, i32, I32);
                }
                OP_I64_LOAD(arg) => {
                    load_memory!(self, arg, 8, i64, I64);
                }
                OP_I64_LOAD_8_u(arg) => {
                    load_memorySX!(self, arg, 8, i64, I64, u8);
                }
                OP_I64_LOAD_16_u(arg) => {
                    load_memorySX!(self, arg, 8, i64, I64, u16);
                }
                OP_I64_LOAD_32_u(arg) => {
                    load_memorySX!(self, arg, 8, i64, I64, u32);
                }
                OP_I64_LOAD_8_s(arg) => {
                    load_memorySX!(self, arg, 8, i64, I64, i8);
                }
                OP_I64_LOAD_16_s(arg) => {
                    load_memorySX!(self, arg, 8, i64, I64, i16);
                }
                OP_I64_LOAD_32_s(arg) => {
                    load_memorySX!(self, arg, 8, i64, I64, i32);
                }
                OP_F32_LOAD(arg) => {
                    load_memory!(self, arg, 4, f32, F32);
                }
                OP_F64_LOAD(arg) => {
                    load_memory!(self, arg, 8, f64, F64);
                }
                OP_I32_STORE(arg) => {
                    store_memory!(self, arg, 4, i32, I32);
                }
                OP_I64_STORE(arg) => {
                    store_memory!(self, arg, 8, i64, I64);
                }
                OP_F32_STORE(arg) => {
                    store_memory!(self, arg, 4, f32, F32);
                }
                OP_F64_STORE(arg) => {
                    store_memory!(self, arg, 8, f64, F64);
                }
                OP_I32_STORE_8(arg) => {
                    store_memoryN!(self, arg, 1, i32, I32, i8, 8);
                }
                OP_I32_STORE_16(arg) => {
                    store_memoryN!(self, arg, 2, i32, I32, i16, 16);
                }
                OP_I64_STORE_8(arg) => {
                    store_memoryN!(self, arg, 1, i64, I64, i8, 8);
                }
                OP_I64_STORE_16(arg) => {
                    store_memoryN!(self, arg, 2, i64, I64, i16, 16);
                }
                OP_I64_STORE_32(arg) => {
                    store_memoryN!(self, arg, 4, i64, I64, i32, 32);
                }
                OP_MEMORY_SIZE => {
                    self.memory_size().context("Memory size failed")?;
                }
                OP_MEMORY_GROW => {
                    self.memory_grow().context("Memory grow failed")?;
                }
                OP_BLOCK(ty, block_instructions) => {
                    let outcome = self
                        .block(fr, ty, block_instructions)
                        .with_context(|| format!("OP_BLOCK with ty {:?} failed", ty))?;

                    match outcome {
                        InstructionOutcome::BRANCH(0) => {}
                        InstructionOutcome::BRANCH(x) => {
                            self.exit_block()?;
                            return Ok(InstructionOutcome::BRANCH(x - 1));
                        }
                        InstructionOutcome::RETURN => {
                            return Ok(InstructionOutcome::RETURN);
                        }
                        InstructionOutcome::EXIT => {}
                    }

                    self.exit_block()?;
                }
                OP_LOOP(ty, block_instructions) => {
                    debug!("OP_LOOP {:?}, {:?}", ty, block_instructions);

                    let arity = self.get_block_ty_arity(&ty)?;

                    debug!("Arity for loop ({:?}) is {}", ty, arity);

                    let label = Label::new(arity);

                    self.store.stack.push(StackContent::Label(label));

                    loop {
                        let outcome = self
                            .run_instructions(fr, &mut block_instructions.iter())
                            .with_context(|| {
                                format!("OP_LOOP({:?}) `run_instructions` failed", ty)
                            })?;

                        match outcome {
                            InstructionOutcome::BRANCH(0) => {
                                continue;
                            }
                            InstructionOutcome::BRANCH(x) => {
                                self.exit_block()?;
                                return Ok(InstructionOutcome::BRANCH(x - 1));
                            }
                            InstructionOutcome::RETURN => {
                                return Ok(InstructionOutcome::RETURN);
                            }
                            InstructionOutcome::EXIT => {
                                break;
                            }
                        }
                    }

                    self.exit_block()?;
                }
                OP_IF(ty, block_instructions_branch) => {
                    debug!("OP_IF {:?}", ty);
                    let element = self.store.stack.pop();
                    debug!("Popping value {:?}", element);

                    if let Some(StackContent::Value(Value::I32(v))) = element {
                        let arity = self.get_block_ty_arity(&ty)?;

                        debug!("Arity for if ({:?}) is {}", ty, arity);

                        if v != 0 {
                            debug!("C is not zero, therefore branching");

                            let label = Label::new(arity);

                            self.store.stack.push(StackContent::Label(label));

                            let outcome = self
                                .run_instructions(fr, &mut block_instructions_branch.iter())
                                .with_context(|| {
                                    format!("OP_IF({:?}) `run_instructions` failed", ty)
                                })?;

                            match outcome {
                                InstructionOutcome::BRANCH(0) => {}
                                InstructionOutcome::BRANCH(x) => {
                                    self.exit_block()?;
                                    return Ok(InstructionOutcome::BRANCH(x - 1));
                                }
                                InstructionOutcome::RETURN => {
                                    return Ok(InstructionOutcome::RETURN);
                                }
                                InstructionOutcome::EXIT => {}
                            }

                            self.exit_block()?;
                        } else {
                            debug!("C is zero, therefore not branching");
                        }
                    } else {
                        panic!("Value must be i32.const. Instead {:#?}", element);
                    }
                }
                OP_IF_AND_ELSE(ty, block_instructions_branch_1, block_instructions_branch_2) => {
                    debug!("OP_IF_AND_ELSE {:?}", ty);
                    if let Some(StackContent::Value(Value::I32(v))) = self.store.stack.pop() {
                        let arity = self.get_block_ty_arity(&ty)?;

                        let label = Label::new(arity);

                        self.store.stack.push(StackContent::Label(label));
                        if v != 0 {
                            debug!("C is not zero, therefore branching (1)");

                            let outcome = self
                                .run_instructions(fr, &mut block_instructions_branch_1.iter())
                                .with_context(|| {
                                    format!("OP_IF_AND_ELSE({:?}) `run_instructions` failed", ty)
                                })?;

                            match outcome {
                                InstructionOutcome::BRANCH(0) => {}
                                InstructionOutcome::BRANCH(x) => {
                                    self.exit_block()?;
                                    return Ok(InstructionOutcome::BRANCH(x - 1));
                                }
                                InstructionOutcome::RETURN => {
                                    return Ok(InstructionOutcome::RETURN);
                                }
                                InstructionOutcome::EXIT => {}
                            }
                        } else {
                            debug!("C is zero, therefore branching (2)");

                            let outcome = self
                                .run_instructions(fr, &mut block_instructions_branch_2.iter())
                                .with_context(|| {
                                    format!("OP_IF_AND_ELSE({:?}) `run_instructions` failed", ty)
                                })?;

                            match outcome {
                                InstructionOutcome::BRANCH(0) => {}
                                InstructionOutcome::BRANCH(x) => {
                                    self.exit_block()?;
                                    return Ok(InstructionOutcome::BRANCH(x - 1));
                                }
                                InstructionOutcome::RETURN => {
                                    return Ok(InstructionOutcome::RETURN);
                                }
                                InstructionOutcome::EXIT => {}
                            }
                        }

                        self.exit_block()?;
                    } else {
                        panic!("Value must be i32.const");
                    }
                }
                OP_BR(label_idx) => {
                    debug!("OP_BR {}", label_idx);

                    return Ok(InstructionOutcome::BRANCH(*label_idx));
                }
                OP_BR_IF(label_idx) => {
                    debug!("OP_BR_IF {}", label_idx);
                    if let Some(StackContent::Value(Value::I32(c))) = self.store.stack.pop() {
                        debug!("c is {}", c);
                        if c != 0 {
                            debug!("Branching to {}", label_idx);
                            return Ok(InstructionOutcome::BRANCH(*label_idx));
                        } else {
                            debug!("Not Branching to {}", label_idx);
                        }
                    }
                }
                OP_BR_TABLE(table, default) => {
                    debug!("OP_BR_TABLE {:?}, {:?}", table, default);
                    let ival = fetch_unop!(self.store.stack);
                    if let I32(index) = ival {
                        let label_idx = if (index as usize) < table.len() {
                            table[index as usize]
                        } else {
                            debug!("Using default case");
                            *default
                        };
                        return Ok(InstructionOutcome::BRANCH(label_idx));
                    } else {
                        panic!("invalid index type: {:?}", ival);
                    }
                }
                OP_CALL(idx) => {
                    self.call_function(idx)
                        .with_context(|| format!("OP_CALL for function addr {} failed", idx))?;
                }
                OP_CALL_INDIRECT(idx) => {
                    self.call_indirect_function(idx)
                        .with_context(|| format!("OP_CALL_INDIRECT for function addr {} failed", idx))?;
                }
                OP_RETURN => {
                    debug!("Return");
                    return Ok(InstructionOutcome::RETURN);
                }
                OP_NOP => {}
                OP_UNREACHABLE => return Err(anyhow!("Reached unreachable => trap!")),
            }

            trace!("stack {:#?}", self.store.stack);
        }

        Ok(InstructionOutcome::EXIT)
    }

    /// Get the frame at the top of the stack
    fn get_frame(&mut self) -> Result<Frame> {
        debug!("get_frame");
        match self.store.stack.pop() {
            Some(Frame(fr)) => Ok(fr),
            Some(x) => Err(anyhow!("Expected frame but found {:?}", x)),
            None => Err(anyhow!("Empty stack on function call")),
        }
    }

    fn exit_block(&mut self) -> Result<()> {
        debug!("exit_block");

        let mut val_m = Vec::new();

        while let Some(Value(_v)) = self.store.stack.last() {
            val_m.push(self.store.stack.pop().unwrap());
        }

        val_m.reverse();

        debug!("values before applying block's arity {:?}", val_m);

        if let Some(Label(lb)) = self.store.stack.pop() {
            debug!("Label on the top of the stack {:?}", lb);

            // If and If-Else labels have arity 0
            // therefore, we keep all results
            if lb.get_arity() != 0 {
                val_m = val_m
                    .into_iter()
                    .rev()
                    .take(lb.get_arity() as usize)
                    .rev()
                    .collect();
            }
        } else {
            return Err(anyhow!("Expected label, but it's not a label"));
        }

        debug!("values after applying block's arity {:?}", val_m);
        self.store.stack.append(&mut val_m);

        Ok(())
    }

    fn get_block_ty_arity(&mut self, block_ty: &BlockType) -> Result<Arity> {
        let arity = match block_ty {
            BlockType::Empty => 0,
            BlockType::ValueType(_) => 1,
            BlockType::ValueTypeTy(ty) => self
                .module
                .fn_types
                .get(*ty as usize)
                .ok_or_else(|| anyhow!("Trap"))?
                .return_types
                .len(),
        };

        debug!("Arity is {}", arity);

        Ok(arity as u32)
    }
}

/// Maps `StackContent` to `Value`, if not then throw error
fn map_stackcontent_to_value(x: StackContent) -> Result<Value> {
    debug!("=> Mapping StackContent to Value {:?}", x);

    match x {
        Value(v) => Ok(v),
        other => Err(anyhow!("Expected value but found {:?}", other)),
    }
}
