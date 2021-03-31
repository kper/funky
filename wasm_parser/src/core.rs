#![allow(clippy::clippy::upper_case_acronyms)]

use custom_display::CustomDisplay;
use serde::{Deserialize, Serialize};

pub type FuncIdx = u32;
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct FuncAddr(usize);
pub type TableIdx = u32;
pub type MemoryIdx = u32;
pub type GlobalIdx = u32;
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct GlobalAddr(usize);
pub type LabelIdx = u32;
pub type LocalIdx = u32;

macro_rules! implAddr {
    ($name:ident) => {
        impl $name {
            pub fn new(value: u32) -> Self {
                Self(value as usize)
            }

            pub fn get(&self) -> usize {
                self.0
            }
        }
    };
}

implAddr!(FuncAddr);
implAddr!(GlobalAddr);

pub type Expr = Vec<InstructionWrapper>;

/// This struct is basically the same as FuncType.
/// But `FuncType` defines a concrete type of a function.
/// Whereas `FunctionSignature` is the unique function signature in the module.
#[derive(Debug, PartialEq, Eq, Clone, Serialize, Deserialize)]
pub struct FunctionSignature {
    pub param_types: Vec<ValueType>,
    pub return_types: Vec<ValueType>,
}

impl FunctionSignature {
    pub fn empty() -> Self {
        FunctionSignature {
            param_types: vec![],
            return_types: vec![],
        }
    }
}

#[derive(Debug, PartialEq, Eq, Copy, Clone, Serialize, Deserialize)]
pub enum ValueType {
    I32,
    I64,
    F32,
    F64,
}

//type TypeIdx = u32;
#[derive(Debug, PartialEq, Eq, Clone, Serialize, Deserialize)]
pub enum BlockType {
    Empty,
    ValueType(ValueType),
    /// The signature of the block
    /// is defined as function definition
    FuncTy(i64), // this is actually a FuncIdx, but it is required to have s33
}

#[derive(Debug, Serialize, Deserialize, PartialEq)]
pub struct ImportEntry {
    pub module_name: String, //utf8 string
    pub name: String,        //utf8 string
    pub desc: ImportDesc,
}

#[derive(Debug, Serialize, Deserialize, PartialEq)]
pub enum ImportDesc {
    Function { ty: FuncIdx },
    Table { ty: TableType },
    Memory { ty: MemoryType },
    Global { ty: GlobalType },
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, Copy)]
pub enum ExternalKindType {
    Function { ty: u32 },
    Table { ty: u32 },
    Memory { ty: u32 },
    Global { ty: u32 },
}

#[derive(Debug, Serialize, Deserialize, PartialEq)]
pub enum Section {
    Custom(CustomSection),
    Type(TypeSection),
    Import(ImportSection),
    Function(FunctionSection),
    Table(TableSection),
    Memory(MemorySection),
    Global(GlobalSection),
    Export(ExportSection),
    Start(StartSection),
    Element(ElementSection),
    Code(CodeSection),
    Data(DataSection),
    Name(NameSection),
}

#[derive(Debug, PartialEq, Serialize, Deserialize)]
pub struct CustomSection {
    pub name: String,
}

#[derive(Debug, PartialEq, Serialize, Deserialize)]
pub struct TypeSection {
    pub entries: Vec<FunctionSignature>,
}

#[derive(Debug, PartialEq, Serialize, Deserialize)]
pub struct ImportSection {
    pub entries: Vec<ImportEntry>,
}

#[derive(Debug, PartialEq, Serialize, Deserialize)]
pub struct FunctionSection {
    pub types: Vec<FuncIdx>,
}

#[derive(Debug, PartialEq, Serialize, Deserialize)]
pub struct TableSection {
    pub entries: Vec<TableType>,
}

#[derive(Debug, PartialEq, Serialize, Deserialize)]
pub struct MemorySection {
    pub entries: Vec<MemoryType>,
}

#[derive(Debug, PartialEq, Serialize, Deserialize)]
pub struct GlobalSection {
    pub globals: Vec<GlobalVariable>,
}

#[derive(Debug, PartialEq, Serialize, Deserialize)]
pub struct ExportSection {
    pub entries: Vec<ExportEntry>,
}

#[derive(Debug, PartialEq, Serialize, Deserialize)]
pub struct StartSection {
    pub index: u32,
}

#[derive(Debug, PartialEq, Serialize, Deserialize)]
pub struct ElementSection {
    pub entries: Vec<ElementSegment>,
}

#[derive(Debug, PartialEq, Serialize, Deserialize)]
pub struct CodeSection {
    pub entries: Vec<FunctionBody>,
}

#[derive(Debug, PartialEq, Serialize, Deserialize)]
pub struct DataSection {
    pub entries: Vec<DataSegment>,
}

#[derive(Debug, PartialEq, Serialize, Deserialize)]
pub struct NameSection {
    pub name_type: u8,
    pub name_payload_len: u32,
    pub name_payload_data: Vec<u8>,
}

#[derive(Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct TableType {
    pub element_type: u8, //0x70 for future ref
    pub limits: Limits,
}

#[derive(Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct MemoryType {
    pub limits: Limits,
}

#[derive(Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct GlobalType {
    pub value_type: ValueType,
    pub mu: Mu,
}

#[derive(Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum Mu {
    Const,
    Var,
}

#[derive(Debug, PartialEq, Serialize, Deserialize)]
pub struct GlobalVariable {
    pub ty: GlobalType,
    pub init: Expr,
}

#[derive(Debug, PartialEq, Serialize, Deserialize)]
pub struct ExportEntry {
    pub name: String, //utf8 string
    pub kind: ExternalKindType,
}

#[derive(Debug, PartialEq, Serialize, Deserialize)]
pub struct ElementSegment {
    pub table: TableIdx,
    pub offset: Expr,
    pub init: Vec<FuncIdx>,
}

#[derive(Debug, Serialize, Deserialize, PartialEq)]
pub struct DataSegment {
    pub data: MemoryIdx,
    pub offset: Expr,
    pub init: Vec<u8>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct FunctionBody {
    pub locals: Vec<LocalEntry>,
    pub code: Expr,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
/// All parameters are locals. However, we
/// can define more of them with `LocalEntry`
/// A `LocalEntry` does **not** correspond to one local.
/// We have a `count` property here, therefore we need
/// to add as many as `count`.
pub struct LocalEntry {
    pub count: u32,
    pub ty: ValueType,
}

#[derive(Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum Limits {
    Zero(u32),
    One(u32, u32),
}

/// A helper struct to count codeblocks (`OP_CODE`)
/// and instruction ids.
#[derive(Debug, PartialEq, Serialize, Deserialize)]
pub struct Counter {
    value: usize,
    instruction_id: usize,
}

impl Counter {
    /// Increases the `value` for codeblocks.
    fn inc(&mut self) {
        self.value += 1;
    }

    /// Increases the last value and get's the current `value` for codeblocks.
    pub fn get_value(&mut self) -> usize {
        self.inc();
        self.value
    }

    /// Increases the `instruction_id` for instructions.
    fn inc_next_instruction(&mut self) {
        self.instruction_id += 1;
    }

    /// Increases the last `instruction_id` and get's the value for instructions.
    pub fn get_next_instruction(&mut self) -> usize {
        self.inc_next_instruction();
        self.instruction_id
    }
}

impl Default for Counter {
    fn default() -> Self {
        Counter {
            value: 0,
            instruction_id: 0,
        }
    }
}

#[derive(Debug, PartialEq, Clone, Serialize, Deserialize)]
pub struct CodeBlock {
    pub id: usize,
    instructions: Vec<InstructionWrapper>,
}

impl CodeBlock {
    /// Creates a new `CodeBlock`.
    pub fn new(counter: &mut Counter, instructions: Vec<Instruction>) -> Self {
        Self {
            id: counter.get_value(),
            instructions: InstructionWrapper::wrap_instructions(counter, instructions),
        }
    }

    /// Checks whether the given `cmp` instruction
    /// is in this CodeBlock
    pub fn has_instruction(&self, cmp: usize) -> bool {
        let min = self
            .instructions
            .iter()
            .min_by_key(|x| x.instruction_id)
            .unwrap()
            .instruction_id;
        let max = self
            .instructions
            .iter()
            .max_by_key(|x| x.instruction_id)
            .unwrap()
            .instruction_id;

        min <= cmp && cmp <= max
    }

    pub fn iter(&self) -> std::slice::Iter<'_, InstructionWrapper> {
        self.instructions.iter()
    }

    /// Get the code block's instructions.
    pub fn get_instructions(&self) -> &[InstructionWrapper] {
        &self.instructions
    }
}

/// Wrapper for the opcodes
#[derive(Debug, PartialEq, Clone, Serialize, Deserialize)]
pub struct InstructionWrapper {
    /// Identifies the instruction
    instruction_id: usize,
    /// Instruction code
    instruction: Instruction,
}

impl InstructionWrapper {
    /// Wraps an instruction with `instruction_id` and returns an `InstructionWrapper`.
    pub fn wrap(counter: &mut Counter, instruction: Instruction) -> Self {
        Self {
            instruction_id: counter.get_next_instruction(),
            instruction,
        }
    }

    /// Wrap multiple instructions with `instruction_id` and returns a `Vev<InstructionWrapper>`.
    pub fn wrap_instructions(counter: &mut Counter, instructions: Vec<Instruction>) -> Vec<Self> {
        instructions
            .into_iter()
            .map(|i| InstructionWrapper::wrap(counter, i))
            .collect()
    }

    pub fn get_instruction(&self) -> &Instruction {
        &self.instruction
    }

    pub fn get_id(&self) -> usize {
        self.instruction_id
    }
}

/// Internal representation of web assembly's opcodes
#[derive(Debug, CustomDisplay, PartialEq, Clone, Serialize, Deserialize)]
#[allow(non_snake_case)]
#[allow(non_camel_case_types)]
#[allow(unused_variables)]
pub enum Instruction {
    // CTRL
    OP_UNREACHABLE,
    OP_NOP,
    OP_BLOCK(BlockType, CodeBlock),
    OP_LOOP(BlockType, CodeBlock),
    OP_IF(BlockType, CodeBlock),
    OP_IF_AND_ELSE(BlockType, CodeBlock, CodeBlock),
    OP_BR(LabelIdx),    //label_id
    OP_BR_IF(LabelIdx), //label_id
    OP_BR_TABLE(Vec<LabelIdx>, LabelIdx),
    OP_RETURN,
    OP_CALL(FuncIdx),
    OP_CALL_INDIRECT(FuncIdx),

    // Param
    OP_DROP,
    OP_SELECT,

    // Var
    OP_LOCAL_GET(LocalIdx),
    OP_LOCAL_SET(LocalIdx),
    OP_LOCAL_TEE(LocalIdx),
    OP_GLOBAL_GET(LocalIdx),
    OP_GLOBAL_SET(LocalIdx),

    // Mem
    OP_I32_LOAD(MemArg),
    OP_I64_LOAD(MemArg),
    OP_F32_LOAD(MemArg),
    OP_F64_LOAD(MemArg),
    OP_I32_LOAD_8_s(MemArg),
    OP_I32_LOAD_8_u(MemArg),
    OP_I32_LOAD_16_s(MemArg),
    OP_I32_LOAD_16_u(MemArg),
    OP_I64_LOAD_8_s(MemArg),
    OP_I64_LOAD_8_u(MemArg),
    OP_I64_LOAD_16_s(MemArg),
    OP_I64_LOAD_16_u(MemArg),
    OP_I64_LOAD_32_u(MemArg),
    OP_I64_LOAD_32_s(MemArg),
    OP_I32_STORE(MemArg),
    OP_I64_STORE(MemArg),
    OP_F32_STORE(MemArg),
    OP_F64_STORE(MemArg),
    OP_I32_STORE_8(MemArg),
    OP_I32_STORE_16(MemArg),
    OP_I64_STORE_8(MemArg),
    OP_I64_STORE_16(MemArg),
    OP_I64_STORE_32(MemArg),
    OP_MEMORY_SIZE,
    OP_MEMORY_GROW,

    // Num
    OP_I32_CONST(i32),
    OP_I64_CONST(i64),
    OP_F32_CONST(f32),
    OP_F64_CONST(f64),

    OP_I32_EQZ,
    OP_I32_EQ,
    OP_I32_NE,
    OP_I32_LT_S,
    OP_I32_LT_U,
    OP_I32_GT_S,
    OP_I32_GT_U,
    OP_I32_LE_S,
    OP_I32_LE_U,
    OP_I32_GE_S,
    OP_I32_GE_U,

    OP_I64_EQZ,
    OP_I64_EQ,
    OP_I64_NE,
    OP_I64_LT_S,
    OP_I64_LT_U,
    OP_I64_GT_S,
    OP_I64_GT_U,
    OP_I64_LE_S,
    OP_I64_LE_U,
    OP_I64_GE_S,
    OP_I64_GE_U,

    OP_F32_EQ,
    OP_F32_NE,
    OP_F32_LT,
    OP_F32_GT,
    OP_F32_LE,
    OP_F32_GE,

    OP_F64_EQ,
    OP_F64_NE,
    OP_F64_LT,
    OP_F64_GT,
    OP_F64_LE,
    OP_F64_GE,

    OP_I32_CLZ,
    OP_I32_CTZ,
    OP_I32_POPCNT,
    OP_I32_ADD,
    OP_I32_SUB,
    OP_I32_MUL,
    OP_I32_DIV_S,
    OP_I32_DIV_U,
    OP_I32_REM_S,
    OP_I32_REM_U,
    OP_I32_AND,
    OP_I32_OR,
    OP_I32_XOR,
    OP_I32_SHL,
    OP_I32_SHR_S,
    OP_I32_SHR_U,
    OP_I32_ROTL,
    OP_I32_ROTR,

    OP_I64_CLZ,
    OP_I64_CTZ,
    OP_I64_POPCNT,
    OP_I64_ADD,
    OP_I64_SUB,
    OP_I64_MUL,
    OP_I64_DIV_S,
    OP_I64_DIV_U,
    OP_I64_REM_S,
    OP_I64_REM_U,
    OP_I64_AND,
    OP_I64_OR,
    OP_I64_XOR,
    OP_I64_SHL,
    OP_I64_SHR_S,
    OP_I64_SHR_U,
    OP_I64_ROTL,
    OP_I64_ROTR,

    OP_F32_ABS,
    OP_F32_NEG,
    OP_F32_CEIL,
    OP_F32_FLOOR,
    OP_F32_TRUNC,
    OP_F32_NEAREST,
    OP_F32_SQRT,
    OP_F32_ADD,
    OP_F32_SUB,
    OP_F32_MUL,
    OP_F32_DIV,
    OP_F32_MIN,
    OP_F32_MAX,
    OP_F32_COPYSIGN,

    OP_F64_ABS,
    OP_F64_NEG,
    OP_F64_CEIL,
    OP_F64_FLOOR,
    OP_F64_TRUNC,
    OP_F64_NEAREST,
    OP_F64_SQRT,
    OP_F64_ADD,
    OP_F64_SUB,
    OP_F64_MUL,
    OP_F64_DIV,
    OP_F64_MIN,
    OP_F64_MAX,
    OP_F64_COPYSIGN,

    OP_I32_WRAP_I64,
    OP_I32_TRUNC_F32_S,
    OP_I32_TRUNC_F32_U,
    OP_I32_TRUNC_F64_S,
    OP_I32_TRUNC_F64_U,
    OP_I64_EXTEND_I32_U,
    OP_I64_EXTEND_I32_S,
    OP_I64_TRUNC_F32_S,
    OP_I64_TRUNC_F32_U,
    OP_I64_TRUNC_F64_S,
    OP_I64_TRUNC_F64_U,
    OP_F32_CONVERT_I32_S, // Converts work opposite
    OP_F32_CONVERT_I32_U, // convert I32_U to F32
    OP_F32_CONVERT_I64_S,
    OP_F32_CONVERT_I64_U,
    OP_F32_DEMOTE_F64,
    OP_F64_CONVERT_I32_S,
    OP_F64_CONVERT_I32_U,
    OP_F64_CONVERT_I64_S,
    OP_F64_CONVERT_I64_U,
    OP_F64_PROMOTE_F32,
    OP_I32_REINTERPRET_F32,
    OP_I64_REINTERPRET_F64,
    OP_F32_REINTERPRET_I32,
    OP_F64_REINTERPRET_I64,

    OP_I32_EXTEND8_S,
    OP_I32_EXTEND16_S,
    OP_I64_EXTEND8_S,
    OP_I64_EXTEND16_S,
    OP_I64_EXTEND32_S,

    OP_I32_TRUNC_SAT_F32_S,
    OP_I32_TRUNC_SAT_F32_U,
    OP_I32_TRUNC_SAT_F64_S,
    OP_I32_TRUNC_SAT_F64_U,
    OP_I64_TRUNC_SAT_F32_S,
    OP_I64_TRUNC_SAT_F32_U,
    OP_I64_TRUNC_SAT_F64_S,
    OP_I64_TRUNC_SAT_F64_U,
}

#[derive(Debug, PartialEq, Clone, Serialize, Deserialize)]
pub struct MemArg {
    pub align: u32,
    pub offset: u32,
}

impl std::convert::From<u8> for ValueType {
    fn from(item: u8) -> Self {
        use log::debug;

        debug!("convert value type {:X}", item);
        match item {
            0x7F => Self::I32,
            0x7E => Self::I64,
            0x7D => Self::F32,
            0x7C => Self::F64,
            _ => panic!("wrong value type"),
        }
    }
}

impl std::convert::From<u8> for Mu {
    fn from(item: u8) -> Self {
        match item {
            0x00 => Self::Const,
            0x01 => Self::Var,
            _ => panic!("Mu failed"),
        }
    }
}
