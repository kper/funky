use serde::{Serialize, Deserialize};

pub enum SectionType {
    Type,
    Import,
    Function,
    Table,
    Memory,
    Global,
    Export,
    Start,
    Element,
    Code,
    Data,
    Custom,
}

#[derive(Debug, PartialEq, Eq, Clone, Serialize, Deserialize)]
pub enum ValueType {
    I32,
    I64,
    F32,
    F64,
}

#[derive(Debug, PartialEq, Eq, Clone, Serialize, Deserialize)]
pub enum BlockType {
    Empty,
    ValueType(ValueType),
    S33(i64), //actually signed 33
}

#[derive(Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct FuncType {
    //form: ValueType,
    //param_count: varuint32,
    pub param_types: Vec<ValueType>,
    //return_count: varuint1,
    pub return_types: Vec<ValueType>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ImportEntry {
    pub module_name: String, //utf8 string
    pub name: String,        //utf8 string
    pub desc: ExternalKindType,
}

#[derive(Debug, Serialize, Deserialize)]
pub enum ExternalKindType {
    Function { ty: u32 },
    Table { ty: u32 },
    Memory { ty: u32 },
    Global { ty: u32 },
}

#[derive(Debug, Serialize, Deserialize)]
pub enum Section {
    Custom {
        name: String,
    },
    Type {
        //count: varuint32,
        entries: Vec<FuncType>,
    },
    Import {
        entries: Vec<ImportEntry>,
    },
    Function {
        types: Vec<u32>,
    },
    Table {
        entries: Vec<TableType>,
    },
    Memory {
        entries: Vec<MemoryType>,
    },
    Global {
        globals: Vec<GlobalVariable>,
    },
    Export {
        entries: Vec<ExportEntry>,
    },
    Start {
        index: u32,
    },
    Element {
        entries: Vec<ElementSegment>,
    },
    Code {
        entries: Vec<FunctionBody>,
    },
    Data {
        entries: Vec<DataSegment>,
    },
    Name {
        name_type: u8,
        name_payload_len: u32,
        name_payload_data: Vec<u8>,
    },
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

#[derive(Debug, Serialize, Deserialize)]
pub struct GlobalVariable {
    pub ty: GlobalType,
    pub init: Expr,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ExportEntry {
    pub name: String, //utf8 string
    pub kind: ExternalKindType,
    //index: varuint32,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ElementSegment {
    pub index: u32,
    pub offset: Expr,
    pub elems: Vec<u32>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct DataSegment {
    pub index: u32,
    pub offset: Expr,
    pub data: Vec<u8>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FunctionBody {
    pub locals: Vec<LocalEntry>,
    pub code: Expr,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LocalEntry {
    pub count: u32,
    pub ty: ValueType,
}

#[derive(Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum Limits {
    Zero(u32),
    One(u32, u32),
}

type Expr = Vec<Instruction>;

#[derive(Debug, PartialEq, Clone, Serialize, Deserialize)]
#[allow(non_camel_case_types)]
pub enum Instruction {
    Ctrl(CtrlInstructions),
    Param(ParamInstructions),
    Var(VarInstructions),
    Mem(MemoryInstructions),
    Num(NumericInstructions),
}

pub type LabelIdx = u32;
pub type FuncIdx = u32;
pub type LocalIdx = u32;

#[derive(Debug, PartialEq, Clone, Serialize, Deserialize)]
#[allow(non_camel_case_types)]
pub enum CtrlInstructions {
    OP_UNREACHABLE,
    OP_NOP,
    OP_BLOCK(BlockType, Vec<Instruction>),
    OP_LOOP(BlockType, Vec<Instruction>),
    OP_IF(BlockType, Vec<Instruction>),
    OP_IF_AND_ELSE(BlockType, Vec<Instruction>, Vec<Instruction>),
    OP_BR(LabelIdx),    //label_id
    OP_BR_IF(LabelIdx), //label_id
    OP_BR_TABLE(Vec<LabelIdx>, LabelIdx),
    OP_RETURN,
    OP_CALL(FuncIdx),
    OP_CALL_INDIRECT(FuncIdx),
    OP_END,
}

#[derive(Debug, PartialEq, Clone, Serialize, Deserialize)]
#[allow(non_camel_case_types)]
pub enum ParamInstructions {
    OP_DROP,
    OP_SELECT,
}

#[derive(Debug, PartialEq, Clone, Serialize, Deserialize)]
#[allow(non_camel_case_types)]
pub enum VarInstructions {
    OP_LOCAL_GET(LocalIdx),
    OP_LOCAL_SET(LocalIdx),
    OP_LOCAL_TEE(LocalIdx),
    OP_GLOBAL_GET(LocalIdx),
    OP_GLOBAL_SET(LocalIdx),
}

#[derive(Debug, PartialEq, Clone, Serialize, Deserialize)]
pub struct MemArg {
    pub align: u32,
    pub offset: u32,
}

#[derive(Debug, PartialEq, Clone, Serialize, Deserialize)]
#[allow(non_camel_case_types)]
pub enum MemoryInstructions {
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
}

#[derive(Debug, PartialEq, Clone, Serialize, Deserialize)]
#[allow(non_camel_case_types)]
pub enum NumericInstructions {
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
    OP_F32_CONVERT_I32_S,
    OP_F32_CONVERT_I32_U,
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
}

impl std::convert::From<u8> for SectionType {
    fn from(item: u8) -> Self {
        match item {
            0 => Self::Custom,
            1 => Self::Type,
            2 => Self::Import,
            3 => Self::Function,
            4 => Self::Table,
            5 => Self::Memory,
            6 => Self::Global,
            7 => Self::Export,
            8 => Self::Start,
            9 => Self::Element,
            10 => Self::Code,
            11 => Self::Data,
            _ => panic!("wrong section id"),
        }
    }
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

/*
impl std::convert::From<u8> for BlockType {
    fn from(item: u8) -> Self {
        match item {
            0x40 => Self::Empty,
            0x7 => Self::ValueType(v.into()),
        }
    }
}
*/

impl std::convert::From<u8> for Mu {
    fn from(item: u8) -> Self {
        match item {
            0x00 => Self::Const,
            0x01 => Self::Var,
            _ => panic!("Mu failed"),
        }
    }
}
