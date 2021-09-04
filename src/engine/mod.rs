#![allow(dead_code)]
#![allow(clippy::upper_case_acronyms)]

pub(crate) mod export;
pub mod func;
pub mod import_resolver;
pub(crate) mod memory;
pub mod module;
mod op;
pub(crate) mod prelude;
pub mod stack;
pub mod store;
pub(crate) mod table;

use self::stack::StackContent;
use self::stack::StackContent::*;
use self::stack::{Frame, Label};
use self::store::Store;
use crate::convert;
pub use crate::debugger::BorrowedProgramState;
pub use crate::debugger::{ProgramCounter, RelativeProgramCounter};
use crate::engine::func::FuncInstance;
use crate::engine::import_resolver::Import;
use crate::engine::module::ModuleInstance;
pub use crate::engine::store::GlobalInstance;
pub use crate::engine::table::TableInstance;
use crate::operations::*;
pub use crate::page::Page;
use crate::value::{Value, Value::*};
pub use crate::PAGE_SIZE;
pub use anyhow::{anyhow, bail, Context, Result};
pub use wasm_parser::core::Instruction::*;
pub use wasm_parser::core::*;
pub use wasm_parser::Module;

#[derive(Debug)]
pub struct Engine {
    pub module: ModuleInstance,
    //TODO rename to `module_instance`
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
    let mi = ModuleInstance::default();

    Engine {
        started: true,
        store: Store::default_with_frame(),
        module: mi,
        debugger: Box::new(RelativeProgramCounter::default()),
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct Variable {
    pub mutable: bool,
    //Actually, there is a `Mut` enum. TODO check if makes sense to use it
    pub val: Value,
}

impl Variable {
    pub fn immutable(val: Value) -> Self {
        Self {
            mutable: false,
            val,
        }
    }
}

#[macro_export]
macro_rules! fetch_unop {
    ($stack: expr) => {{
        use crate::engine::stack::StackContent;
        debug!("Popping {:?}", $stack.last());
        let v1 = match $stack.pop().unwrap() {
            StackContent::Value(v) => v,
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

            let addr = module.lookup_memory_addr(&0)
                .context("No memory address found")?;

            let instance = &$self.store.memory[addr.get()];

            debug!("instance {:?}", instance);
            debug!("Range {:?}", ea..ea + $size);
            debug!("part {:?}", &instance.data[ea..ea + $size]);

            let mut b = vec![0; $size];
            b.copy_from_slice(&instance.data[ea..ea + $size]);
            assert!(b.len() == $size);

            debug!("b {:?}", b);

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

macro_rules! load_memory_sx {
    ($self:expr, $arg:expr, $size:expr, $ty:ty, $variant:expr, $cast_ty:ty) => {
        let v1 = fetch_unop!($self.store.stack);

        if let I32(v) = v1 {
            let ea = (v + $arg.offset as i32) as usize;

            let module = &$self.module;

            let addr = module.lookup_memory_addr(&0).context("No memory address found")?;

            let instance = &$self
                .store
                .memory
                .get(addr.get())
                .with_context(|| format!("Cannot access memory addr {:?}", addr))?;

            debug!("instance {:?}", instance);
            debug!("Range {:?}", ea..ea + $size);
            debug!(
                "part {:?}",
                &instance
                    .data
                    .get(ea..ea + $size)
                    .context("Cannot access range")
            );

            let mut b = vec![0; $size];
            b.copy_from_slice(
                &instance
                    .data
                    .get(ea..ea + $size)
                    .context("Cannot access range")?,
            );
            assert!(b.len() == $size);

            debug!("b {:?}", b);

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

                let addr = module.lookup_memory_addr(&0).context("No memory address found")?;

                let instance = &mut $self.store.memory[addr.get()];

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

macro_rules! store_memory_n {
    ($self:expr, $arg:expr, $size:expr, $ty:ty, $variant:ident, $new_ty:ty, $N:expr) => {
        let k = fetch_unop!($self.store.stack);
        let v1 = fetch_unop!($self.store.stack);

        if let $variant(t) = k {
            if let I32(v) = v1 {
                let ea = (v + $arg.offset as i32) as usize;

                let module = &$self.module;

                let addr = module.lookup_memory_addr(&0).context("No memory address found")?;

                let instance = &mut $self.store.memory[addr.get()];

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
    pub fn new(
        mi: ModuleInstance,
        module: &Module,
        debugger: Box<dyn ProgramCounter>,
        imports: &[Import],
    ) -> Result<Engine> {
        let mut e = Engine {
            module: mi,
            started: false,
            store: Store::default(),
            debugger,
        };

        e.allocate(module, &imports)
            .context("Allocation instance failed")?;
        e.instantiation(module).context("Instantiation failed")?;

        Ok(e)
    }

    /// Initializes `n` pages in memory
    pub(crate) fn init_memory(&mut self, n: usize) -> Result<()> {
        let res = self
            .store
            .init_memory(n)
            .context("Trying to initialize an empty memory instance");

        // only one memory module is allowed
        assert!(self.module.get_mem_addrs().is_empty());
        let addr = MemoryAddr::new(self.module.get_mem_addrs().len());
        self.module.store_memory_addr(addr)?;

        res
    }

    fn allocate(&mut self, m: &Module, imports: &[Import]) -> Result<()> {
        info!("Allocation");
        crate::allocation::allocate(m, &mut self.module, &mut self.store, imports)
            .context("Allocation failed")?;

        Ok(())
    }

    pub fn instantiation(&mut self, m: &Module) -> Result<()> {
        info!("Instantiation");
        let start_function = crate::instantiation::instantiation(m, &self.module, &mut self.store)
            .context("Instantiation failed")?;

        if let Some(func_addr) = start_function {
            debug!("Invoking start function with {:?}", func_addr);
            self.invoke_function(func_addr, vec![])
                .context("Invoking function failed")?;
        }

        Ok(())
    }

    /// Get a global value
    pub fn get(&mut self, name: &str) -> Result<Value> {
        debug!("get global for {:?}", name);

        let export_instance = self.module.get_export_instance_by_name(name)
            .ok_or_else(|| anyhow!("Export instance was not found by name: {}", name))?;
        debug!("Export {:#?}", export_instance);

        match export_instance.value {
            ExternalKindType::Global { ty } => {
                // The external type of the export is a global.

                let global_addr = self.module.lookup_global_addr(&ty)
                    .context("Global not found")?;

                Ok(self
                    .store
                    .get_global_instance(&global_addr)
                    .context("Global not found in the store")?
                    .val)
            }
            _ => Err(anyhow!("Exported global not found")),
        }
    }

    /// Adding new function to the engine
    /// It will allocate the function in store and add it to the module's code.
    pub(crate) fn add_function(&mut self, signature: FunctionSignature, body: FunctionBody) -> Result<()> {
        self.module.add_code(body.clone())?;
        self.store.allocate_func_instance(signature, body);

        Ok(())
    }

    /*
    /// Get function's address by index.
    pub fn get_function_addr_by_index(&self, ty: u32) -> Result<FuncAddr> {
        Ok(self
            .module
            .func_addrs
            .get(ty as usize)
            .ok_or_else(|| anyhow!("Function not found"))?
            .clone())
    }*/

    /// Get function's instance by addr
    pub fn get_function_instance(&self, addr: &FuncAddr) -> Result<&FuncInstance> {
        self.store.get_func_instance(addr)
    }

    /// Take only exported functions into consideration
    /// `idx` is the id of the export instance, not the function
    pub fn invoke_exported_function(&mut self, idx: u32, args: Vec<Value>) -> Result<()> {
        debug!("invoke_exported_function {} with args {:?}", idx, args);
        let k = {
            let x = &self.module;

            debug!("the export instance is {:?}", x.get_export_instance(idx as usize));

            let w = x
                .get_export_instance(idx as usize)
                .ok_or_else(|| anyhow!("Exported function not found or found something else"))?;

            w.value
        };

        debug!("Exports {:#?}", k);

        match k {
            ExternalKindType::Function { ty } => {
                let func_addr = self.module.lookup_function_addr(&ty)
                    .ok_or_else(|| anyhow!("Cannot find function's addr"))?
                    .clone();

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
            .position_export_instance_by_name(name)
            .ok_or_else(|| anyhow!("Cannot find export instance by name: {}", name))?;

        debug!("=> Export instance found");
        debug!("The function addr is {:?}", idx);

        self.invoke_exported_function(idx as u32, args)
            .context("Invoking the exported function failed")?;

        Ok(())
    }

    pub(crate) fn invoke_function(&mut self, func_addr: FuncAddr, args: Vec<Value>) -> Result<()> {
        self.check_parameters_of_function(&func_addr, &args)
            .with_context(|| format!("Checking parameter for function {:?} failed", func_addr))?;

        let count_return_types =
            self.get_function_instance(&func_addr)?.ty.return_types.len() as u32;

        let typed_locals = &self.store.get_func_instance(&func_addr)?.code.locals;

        debug!("defined locals are {:#?}", typed_locals);

        let mut locals = args;

        // All parameters are `locals`, but
        // we can additionaly define more of them.
        // This is done in the function definition of locals
        // It is very important to use the correct type
        debug!("Adding additional locals");
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

        debug!("Locals for frame are {:?}", locals);

        let mut frame = Frame {
            arity: count_return_types,
            locals,
        };

        self.store.stack.push(StackContent::Frame(frame.clone()));

        self.store
            .stack
            .push(StackContent::Label(Label::new(count_return_types, 0)));

        trace!("stack before invoking {:#?}", self.store.stack);

        debug!("Invoking function");
        self.run_function(&mut frame, &func_addr)
            .with_context(|| format!("Function with addr {:?} failed", func_addr))?;

        Ok(())
    }

    fn check_parameters_of_function(&self, func_addr: &FuncAddr, args: &[Value]) -> Result<()> {
        debug!(
            "Required type of function {:?} is {:?}",
            func_addr,
            self.store.get_func_instance(func_addr)?.ty
        );

        let fn_types = self
            .store
            .get_func_instance(func_addr)?
            .ty
            .param_types
            .iter();

        // Extract the ValueType
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
            // Report error

            let argtypes2 = args
                .iter()
                .map(|w| match *w {
                    Value::I32(_) => ValueType::I32,
                    Value::I64(_) => ValueType::I64,
                    Value::F32(_) => ValueType::F32,
                    Value::F64(_) => ValueType::F64,
                })
                .collect::<Vec<_>>();

            let fn_types2 = self
                .store
                .get_func_instance(func_addr)?
                .ty
                .param_types
                .iter()
                .collect::<Vec<_>>();

            return Err(anyhow!(
                "Function expected different parameters! {:?} != {:?}",
                argtypes2,
                fn_types2
            ));
        }

        debug!("=> Parameters of functions and args are equal");

        Ok(())
    }

    #[allow(clippy::cognitive_complexity)]
    pub(crate) fn run_function(&mut self, fr: &mut Frame, func_addr: &FuncAddr) -> Result<()> {
        debug!("Running function with addr {:?}", func_addr);

        //FIXME this `.clone` is extremly expensive!!!
        // But there is a problem
        // Because, we iterate over the borrowed iterator,
        // we cannot easily run the block
        //let f = &self.module.code[idx as usize].clone();
        let f = &self.store.get_func_instance(func_addr)?.code.clone();

        //let f = &self.module.code[func_addr.get() as usize].clone();
        //debug!("mod {:#?}", f);
        //debug!("instance {:#?}", self.store.get_func_instance(func_addr)?.code.clone());

        //let mut fr = self.get_frame()?;

        debug!("frame {:#?}", fr);

        match self
            .run_instructions(fr, &mut f.code.iter())
            .with_context(|| format!("Running instructions for function addr {:?}", func_addr))?
        {
            InstructionOutcome::EXIT | InstructionOutcome::RETURN => {
                self.exit_block()
                    .context("Exiting block for end of function failed")?;
            }
            _ => {}
        }

        // implicit return
        debug!("Implicit return (arity {:?})", fr.arity);

        debug!("Stack before function return {:#?}", self.store.stack);

        let mut ret = Vec::new();
        for _ in 0..fr.arity {
            debug!("Popping {:?}", self.store.stack.last());
            if let Some(val) = self.store.stack.pop() {
                ret.push(val);
            }
        }

        let mut ret = ret.into_iter().rev().collect();

        if let Some(Frame(_)) = self.store.stack.pop() {
            debug!("Popping frame");
        } else {
            bail!("Cannot remove frame");
        }

        self.store.stack.append(&mut ret);

        debug!("Stack after function return {:#?}", self.store.stack);

        Ok(())
    }

    #[allow(clippy::cognitive_complexity)]
    fn run_instructions<'a>(
        &mut self,
        fr: &mut Frame,
        instruction_wrapper: &'a mut impl std::iter::Iterator<Item=&'a InstructionWrapper>,
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
            debug!("Evaluating instruction {}", instruction);

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
                    self.store.stack.push(StackContent::Value(I32(*v)));
                    debug!("stack {:#?}", self.store.stack);
                }
                OP_I64_CONST(v) => {
                    debug!("OP_I64_CONST: pushing {} to stack", v);
                    self.store.stack.push(StackContent::Value(I64(*v)))
                }
                OP_F32_CONST(v) => {
                    debug!("OP_F32_CONST: pushing {} to stack", v);
                    self.store.stack.push(StackContent::Value(F32(*v)))
                }
                OP_F64_CONST(v) => {
                    debug!("OP_F64_CONST: pushing {} to stack", v);
                    self.store.stack.push(StackContent::Value(F64(*v)))
                }
                OP_F32_COPYSIGN => {
                    let (z1, z2) = fetch_binop!(self.store.stack);
                    self.store.stack.push(StackContent::Value(copysign(z1, z2)))
                }
                OP_F64_COPYSIGN => {
                    let (z1, z2) = fetch_binop!(self.store.stack);
                    self.store.stack.push(StackContent::Value(copysign(z1, z2)))
                }
                OP_I32_ADD | OP_I64_ADD | OP_F32_ADD | OP_F64_ADD => {
                    let (v2, v1) = fetch_binop!(self.store.stack);
                    self.store.stack.push(StackContent::Value(v1 + v2))
                }
                OP_I32_SUB | OP_I64_SUB | OP_F32_SUB | OP_F64_SUB => {
                    let (v2, v1) = fetch_binop!(self.store.stack);
                    self.store.stack.push(StackContent::Value(v1 - v2))
                }
                OP_I32_MUL | OP_I64_MUL | OP_F32_MUL | OP_F64_MUL => {
                    let (v2, v1) = fetch_binop!(self.store.stack);
                    self.store.stack.push(StackContent::Value(v1 * v2))
                }
                OP_I32_DIV_S | OP_I64_DIV_S | OP_F32_DIV | OP_F64_DIV => {
                    let (v2, v1) = fetch_binop!(self.store.stack);
                    self.store.stack.push(StackContent::Value(v1 / v2))
                }
                OP_I32_DIV_U | OP_I64_DIV_U => {
                    let (v2, v1) = fetch_binop!(self.store.stack);
                    match (v1, v2) {
                        (I32(x1), I32(x2)) => self
                            .store
                            .stack
                            .push(StackContent::Value(I32(((x1 as u32) / (x2 as u32)) as i32))),
                        (I64(x1), I64(x2)) => self
                            .store
                            .stack
                            .push(StackContent::Value(I64(((x1 as u64) / (x2 as u64)) as i64))),
                        _ => return Err(anyhow!("Invalid types for DIV_U")),
                    }
                }
                OP_I32_REM_S | OP_I64_REM_S => {
                    let (v2, v1) = fetch_binop!(self.store.stack);
                    self.store.stack.push(StackContent::Value(v1 % v2))
                }
                OP_I32_REM_U | OP_I64_REM_U => {
                    let (v2, v1) = fetch_binop!(self.store.stack);
                    match (v1, v2) {
                        (I32(x1), I32(x2)) => self
                            .store
                            .stack
                            .push(StackContent::Value(I32(((x1 as u32) % (x2 as u32)) as i32))),
                        (I64(x1), I64(x2)) => self
                            .store
                            .stack
                            .push(StackContent::Value(I64(((x1 as u64) % (x2 as u64)) as i64))),
                        _ => return Err(anyhow!("Invalid types for REM_U")),
                    }
                }
                OP_I32_AND | OP_I64_AND => {
                    let (v2, v1) = fetch_binop!(self.store.stack);
                    self.store.stack.push(StackContent::Value(v1 & v2))
                }
                OP_I32_OR | OP_I64_OR => {
                    let (v2, v1) = fetch_binop!(self.store.stack);
                    self.store.stack.push(StackContent::Value(v1 | v2))
                }
                OP_I32_XOR | OP_I64_XOR => {
                    let (v2, v1) = fetch_binop!(self.store.stack);
                    self.store.stack.push(StackContent::Value(v1 ^ v2))
                }
                OP_I32_SHL | OP_I64_SHL => {
                    let (v2, v1) = fetch_binop!(self.store.stack);
                    self.store.stack.push(StackContent::Value(v1 << v2))
                }
                OP_I32_SHR_S | OP_I64_SHR_S => {
                    let (v2, v1) = fetch_binop!(self.store.stack);
                    self.store.stack.push(StackContent::Value(v1 >> v2))
                }
                OP_I32_SHR_U | OP_I64_SHR_U => {
                    let (v2, v1) = fetch_binop!(self.store.stack);
                    match (v1, v2) {
                        (I32(x1), I32(x2)) => {
                            let k = x2 as u32 % 32;
                            self.store
                                .stack
                                .push(StackContent::Value(I32(((x1 as u32).checked_shr(k))
                                    .unwrap_or(0)
                                    as i32)));
                        }
                        (I64(x1), I64(x2)) => {
                            let k = x2 as u64 % 64;
                            self.store
                                .stack
                                .push(StackContent::Value(I64(
                                    ((x1 as u64).checked_shr(k as u32)).unwrap_or(0) as i64,
                                )));
                        }
                        _ => return Err(anyhow!("Invalid types for SHR_U")),
                    }
                }
                OP_I32_ROTL | OP_I64_ROTL => {
                    let (v2, v1) = fetch_binop!(self.store.stack);
                    self.store
                        .stack
                        .push(StackContent::Value(rotate_left(v1, v2)))
                }
                OP_I32_ROTR | OP_I64_ROTR => {
                    let (v2, v1) = fetch_binop!(self.store.stack);
                    self.store
                        .stack
                        .push(StackContent::Value(rotate_right(v1, v2)))
                }
                OP_I32_CLZ | OP_I64_CLZ => {
                    let v1 = fetch_unop!(self.store.stack);
                    self.store
                        .stack
                        .push(StackContent::Value(leading_zeros(v1)))
                }
                OP_I32_CTZ | OP_I64_CTZ => {
                    let v1 = fetch_unop!(self.store.stack);
                    self.store
                        .stack
                        .push(StackContent::Value(trailing_zeros(v1)))
                }
                OP_I32_POPCNT | OP_I64_POPCNT => {
                    let v1 = fetch_unop!(self.store.stack);
                    self.store.stack.push(StackContent::Value(count_ones(v1)))
                }
                OP_I32_EQZ | OP_I64_EQZ => {
                    let v1 = fetch_unop!(self.store.stack);

                    self.store.stack.push(StackContent::Value(eqz(v1)))
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
                        .push(StackContent::Value(lt(v1, v2).convert(ValueType::I32)))
                }
                OP_I32_LT_U | OP_I64_LT_U => {
                    let (v2, v1) = fetch_binop!(self.store.stack);
                    match (v1, v2) {
                        (I32(x1), I32(x2)) => self
                            .store
                            .stack
                            .push(StackContent::Value(I32(((x1 as u32) < (x2 as u32)) as i32))),
                        (I64(x1), I64(x2)) => self
                            .store
                            .stack
                            .push(StackContent::Value(I32(((x1 as u64) < (x2 as u64)) as i32))),
                        _ => return Err(anyhow!("Invalid types for LT_U comparison")),
                    }
                }
                OP_I32_GT_S | OP_I64_GT_S | OP_F32_GT | OP_F64_GT => {
                    // switch ordering because of stack layout
                    let (v2, v1) = fetch_binop!(self.store.stack);
                    self.store
                        .stack
                        .push(StackContent::Value(gt(v1, v2).convert(ValueType::I32)))
                }
                OP_I32_GT_U | OP_I64_GT_U => {
                    let (v2, v1) = fetch_binop!(self.store.stack);
                    match (v1, v2) {
                        (I32(x1), I32(x2)) => self
                            .store
                            .stack
                            .push(StackContent::Value(I32(((x1 as u32) > (x2 as u32)) as i32))),
                        (I64(x1), I64(x2)) => self
                            .store
                            .stack
                            .push(StackContent::Value(I32(((x1 as u64) > (x2 as u64)) as i32))),
                        _ => return Err(anyhow!("Invalid types for GT_U comparison")),
                    }
                }
                OP_I32_LE_S | OP_I64_LE_S | OP_F32_LE | OP_F64_LE => {
                    // switch ordering because of stack layout
                    let (v2, v1) = fetch_binop!(self.store.stack);
                    self.store
                        .stack
                        .push(StackContent::Value(le(v1, v2).convert(ValueType::I32)))
                }
                OP_I32_LE_U | OP_I64_LE_U => {
                    let (v2, v1) = fetch_binop!(self.store.stack);
                    match (v1, v2) {
                        (I32(x1), I32(x2)) => {
                            self.store.stack.push(StackContent::Value(I32(((x1 as u32)
                                <= (x2 as u32))
                                as i32)))
                        }
                        (I64(x1), I64(x2)) => {
                            self.store.stack.push(StackContent::Value(I32(((x1 as u64)
                                <= (x2 as u64))
                                as i32)))
                        }
                        _ => return Err(anyhow!("Invalid types for LE_U comparison")),
                    }
                }
                OP_I32_GE_S | OP_I64_GE_S | OP_F32_GE | OP_F64_GE => {
                    // switch ordering because of stack layout
                    let (v2, v1) = fetch_binop!(self.store.stack);
                    self.store
                        .stack
                        .push(StackContent::Value(ge(v1, v2).convert(ValueType::I32)))
                }
                OP_I32_GE_U | OP_I64_GE_U => {
                    let (v2, v1) = fetch_binop!(self.store.stack);
                    match (v1, v2) {
                        (I32(x1), I32(x2)) => {
                            self.store.stack.push(StackContent::Value(I32(((x1 as u32)
                                >= (x2 as u32))
                                as i32)))
                        }
                        (I64(x1), I64(x2)) => {
                            self.store.stack.push(StackContent::Value(I32(((x1 as u64)
                                >= (x2 as u64))
                                as i32)))
                        }
                        _ => return Err(anyhow!("Invalid types for GE_U comparison")),
                    }
                }
                OP_F32_ABS | OP_F64_ABS => {
                    let v1 = fetch_unop!(self.store.stack);
                    self.store.stack.push(StackContent::Value(abs(v1)))
                }
                OP_F32_NEG | OP_F64_NEG => {
                    let v1 = fetch_unop!(self.store.stack);
                    self.store.stack.push(StackContent::Value(neg(v1)))
                }
                OP_F32_CEIL | OP_F64_CEIL => {
                    let v1 = fetch_unop!(self.store.stack);
                    self.store.stack.push(StackContent::Value(ceil(v1)))
                }
                OP_F32_FLOOR | OP_F64_FLOOR => {
                    let v1 = fetch_unop!(self.store.stack);
                    self.store.stack.push(StackContent::Value(floor(v1)))
                }
                OP_F32_TRUNC | OP_F64_TRUNC => {
                    let v1 = fetch_unop!(self.store.stack);
                    self.store.stack.push(StackContent::Value(trunc(v1)))
                }
                OP_I32_TRUNC_SAT_F32_S | OP_I32_TRUNC_SAT_F64_S => {
                    let v1 = fetch_unop!(self.store.stack);
                    self.store
                        .stack
                        .push(StackContent::Value(trunc_sat_i32_s(v1)))
                }
                OP_I64_TRUNC_SAT_F32_S | OP_I64_TRUNC_SAT_F64_S => {
                    let v1 = fetch_unop!(self.store.stack);
                    self.store
                        .stack
                        .push(StackContent::Value(trunc_sat_i64_s(v1)))
                }
                OP_I32_TRUNC_SAT_F32_U => {
                    let v1 = fetch_unop!(self.store.stack);
                    self.store
                        .stack
                        .push(StackContent::Value(trunc_sat_from_f32_to_i32_u(v1)))
                }
                OP_I32_TRUNC_SAT_F64_U => {
                    let v1 = fetch_unop!(self.store.stack);
                    self.store
                        .stack
                        .push(StackContent::Value(trunc_sat_from_f64_to_i32_u(v1)))
                }
                OP_I64_TRUNC_SAT_F32_U => {
                    let v1 = fetch_unop!(self.store.stack);
                    self.store
                        .stack
                        .push(StackContent::Value(trunc_sat_from_f32_to_i64_u(v1)))
                }
                OP_I64_TRUNC_SAT_F64_U => {
                    let v1 = fetch_unop!(self.store.stack);
                    self.store
                        .stack
                        .push(StackContent::Value(trunc_sat_from_f64_to_i64_u(v1)))
                }
                OP_F32_NEAREST | OP_F64_NEAREST => {
                    let v1 = fetch_unop!(self.store.stack);
                    self.store.stack.push(StackContent::Value(nearest(v1)))
                }
                OP_F32_SQRT | OP_F64_SQRT => {
                    let v1 = fetch_unop!(self.store.stack);
                    self.store.stack.push(StackContent::Value(sqrt(v1)))
                }
                OP_F32_MIN | OP_F64_MIN => {
                    let (v1, v2) = fetch_binop!(self.store.stack);
                    self.store.stack.push(StackContent::Value(min(v1, v2)))
                }
                OP_F32_MAX | OP_F64_MAX => {
                    let (v1, v2) = fetch_binop!(self.store.stack);
                    self.store.stack.push(StackContent::Value(max(v1, v2)))
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
                    self.store.stack.push(StackContent::Value(reinterpret(v)));
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
                    load_memory_sx!(self, arg, 4, i32, I32, u8);
                }
                OP_I32_LOAD_16_u(arg) => {
                    load_memory_sx!(self, arg, 4, i32, I32, u16);
                }
                OP_I32_LOAD_8_s(arg) => {
                    load_memory_sx!(self, arg, 4, i32, I32, i8);
                }
                OP_I32_LOAD_16_s(arg) => {
                    load_memory_sx!(self, arg, 4, i32, I32, i16);
                }
                OP_I32_LOAD(arg) => {
                    load_memory!(self, arg, 4, i32, I32);
                }
                OP_I64_LOAD(arg) => {
                    load_memory!(self, arg, 8, i64, I64);
                }
                OP_I64_LOAD_8_u(arg) => {
                    load_memory_sx!(self, arg, 8, i64, I64, u8);
                }
                OP_I64_LOAD_16_u(arg) => {
                    load_memory_sx!(self, arg, 8, i64, I64, u16);
                }
                OP_I64_LOAD_32_u(arg) => {
                    load_memory_sx!(self, arg, 8, i64, I64, u32);
                }
                OP_I64_LOAD_8_s(arg) => {
                    load_memory_sx!(self, arg, 8, i64, I64, i8);
                }
                OP_I64_LOAD_16_s(arg) => {
                    load_memory_sx!(self, arg, 8, i64, I64, i16);
                }
                OP_I64_LOAD_32_s(arg) => {
                    load_memory_sx!(self, arg, 8, i64, I64, i32);
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
                    store_memory_n!(self, arg, 1, i32, I32, i8, 8);
                }
                OP_I32_STORE_16(arg) => {
                    store_memory_n!(self, arg, 2, i32, I32, i16, 16);
                }
                OP_I64_STORE_8(arg) => {
                    store_memory_n!(self, arg, 1, i64, I64, i8, 8);
                }
                OP_I64_STORE_16(arg) => {
                    store_memory_n!(self, arg, 2, i64, I64, i16, 16);
                }
                OP_I64_STORE_32(arg) => {
                    store_memory_n!(self, arg, 4, i64, I64, i32, 32);
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
                        InstructionOutcome::BRANCH(0) => {
                            debug!("Instruction outcome was 0.");
                        }
                        InstructionOutcome::BRANCH(x) => {
                            debug!("Instruction outcome was {}.", x - 1);
                            return Ok(InstructionOutcome::BRANCH(x - 1));
                        }
                        InstructionOutcome::RETURN => {
                            debug!("Instruction outcome was return.");
                            self.exit_block()?;
                            return Ok(InstructionOutcome::RETURN);
                        }
                        InstructionOutcome::EXIT => {
                            debug!("Instruction outcome was exit.");
                            self.exit_block()?;
                        }
                    }
                }
                OP_LOOP(ty, block_instructions) => {
                    debug!("OP_LOOP {:?}, {:?}", ty, block_instructions);

                    let param_count = self.get_mut_param_count_block(&ty)?;
                    loop {
                        self.setup_stack_for_entering_block(ty, block_instructions, param_count)?;

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
                                return Ok(InstructionOutcome::BRANCH(x - 1));
                            }
                            InstructionOutcome::RETURN => {
                                self.exit_block()?;
                                return Ok(InstructionOutcome::RETURN);
                            }
                            InstructionOutcome::EXIT => {
                                self.exit_block()?;
                                break;
                            }
                        }
                    }
                }
                OP_IF(ty, block_instructions_branch) => {
                    debug!("OP_IF {:?}", ty);
                    let element = self.store.stack.pop();
                    debug!("Popping value {:?}", element);

                    if let Some(StackContent::Value(Value::I32(v))) = element {
                        if v != 0 {
                            debug!("C is not zero, therefore branching");

                            let arity_return_count = self.get_return_count_block(&ty)?;
                            self.setup_stack_for_entering_block(
                                ty,
                                block_instructions_branch,
                                arity_return_count,
                            )?;

                            let outcome = self
                                .run_instructions(fr, &mut block_instructions_branch.iter())
                                .with_context(|| {
                                    format!("OP_IF({:?}) `run_instructions` failed", ty)
                                })?;

                            match outcome {
                                InstructionOutcome::BRANCH(0) => {}
                                InstructionOutcome::BRANCH(x) => {
                                    return Ok(InstructionOutcome::BRANCH(x - 1));
                                }
                                InstructionOutcome::RETURN => {
                                    self.exit_block()?;
                                    return Ok(InstructionOutcome::RETURN);
                                }
                                InstructionOutcome::EXIT => {
                                    self.exit_block()?;
                                }
                            }
                        } else {
                            debug!("C is zero, therefore not branching");
                        }
                    } else {
                        return Err(anyhow!("Value must be i32.const. Instead {:#?}", element));
                    }
                }
                OP_IF_AND_ELSE(ty, block_instructions_branch_1, block_instructions_branch_2) => {
                    debug!("OP_IF_AND_ELSE {:?}", ty);
                    let element = self.store.stack.pop();
                    debug!("Popping value {:?}", element);

                    if let Some(StackContent::Value(Value::I32(v))) = element {
                        if v != 0 {
                            debug!("C is not zero, therefore branching (1)");

                            let arity_return_count = self.get_return_count_block(&ty)?;
                            self.setup_stack_for_entering_block(
                                ty,
                                block_instructions_branch_1,
                                arity_return_count,
                            )?;

                            let outcome = self
                                .run_instructions(fr, &mut block_instructions_branch_1.iter())
                                .with_context(|| {
                                    format!("OP_IF_AND_ELSE({:?}) `run_instructions` failed", ty)
                                })?;

                            match outcome {
                                InstructionOutcome::BRANCH(0) => {}
                                InstructionOutcome::BRANCH(x) => {
                                    return Ok(InstructionOutcome::BRANCH(x - 1));
                                }
                                InstructionOutcome::RETURN => {
                                    self.exit_block()?;
                                    return Ok(InstructionOutcome::RETURN);
                                }
                                InstructionOutcome::EXIT => {
                                    self.exit_block()?;
                                }
                            }
                        } else {
                            debug!("C is zero, therefore branching (2)");

                            let arity_return_count = self.get_return_count_block(&ty)?;
                            self.setup_stack_for_entering_block(
                                ty,
                                block_instructions_branch_2,
                                arity_return_count,
                            )?;

                            let outcome = self
                                .run_instructions(fr, &mut block_instructions_branch_2.iter())
                                .with_context(|| {
                                    format!("OP_IF_AND_ELSE({:?}) `run_instructions` failed", ty)
                                })?;

                            match outcome {
                                InstructionOutcome::BRANCH(0) => {}
                                InstructionOutcome::BRANCH(x) => {
                                    return Ok(InstructionOutcome::BRANCH(x - 1));
                                }
                                InstructionOutcome::RETURN => {
                                    self.exit_block()?;
                                    return Ok(InstructionOutcome::RETURN);
                                }
                                InstructionOutcome::EXIT => {
                                    self.exit_block()?;
                                }
                            }
                        }
                    } else {
                        return Err(anyhow!("Value must be i32.const"));
                    }
                }
                OP_BR(label_idx) => {
                    return self.br(fr, *label_idx);
                }
                OP_BR_IF(label_idx) => {
                    debug!("OP_BR_IF {}", label_idx);
                    let element = self.store.stack.pop();
                    debug!("Popping value {:?}", element);

                    if let Some(StackContent::Value(Value::I32(c))) = element {
                        debug!("c is {}", c);
                        if c != 0 {
                            debug!("Branching to {}", label_idx);
                            return self.br(fr, *label_idx);
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

                        return self.br(fr, label_idx);
                    } else {
                        return Err(anyhow!("invalid index type: {:?}", ival));
                    }
                }
                OP_CALL(function_module_addr) => {
                    let func_addr = self.module.lookup_function_addr(function_module_addr)
                        .ok_or_else(|| anyhow!("Cannot find function's addr"))?.clone();
                    let func_addr_inner = func_addr.get();
                    self.call_function(func_addr).with_context(|| {
                        format!(
                            "OP_CALL for function {:?} and module addr ({}) failed",
                            func_addr_inner, function_module_addr
                        )
                    })?;
                }
                OP_CALL_INDIRECT(function_module_addr) => {
                    let func_addr = self.module.lookup_function_addr(function_module_addr)
                        .ok_or_else(|| anyhow!("Cannot find function's addr"))?.clone();
                    let func_addr_inner = func_addr.get();
                    self.call_indirect_function(func_addr).with_context(|| {
                        format!(
                            "OP_CALL_INDIRECT for function {:?} failed and module addr ({}) failed",
                            func_addr_inner, function_module_addr
                        )
                    })?;
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
    fn get_mut_frame(&mut self) -> Result<&Frame> {
        debug!("get_frame");
        // -2 because label
        match self.store.stack.get(self.store.stack.len() - 2) {
            Some(Frame(fr)) => Ok(fr),
            Some(x) => bail!("Expected frame but found {:?}", x),
            None => bail!("Empty stack on function call"),
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
}
