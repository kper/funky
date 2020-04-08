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

#[derive(Debug, PartialEq, Eq)]
pub enum ValueType {
    I32,
    I64,
    F32,
    F64,
}

#[derive(Debug, PartialEq, Eq)]
pub enum BlockType {
    Empty,
    ValueType(ValueType),
}

#[derive(Debug, PartialEq, Eq)]
pub struct FuncType {
    //form: ValueType,
    //param_count: varuint32,
    pub param_types: Vec<ValueType>,
    //return_count: varuint1,
    pub return_types: Vec<ValueType>,
}

#[derive(Debug)]
pub struct ImportEntry {
    pub module_name: String, //utf8 string
    pub name: String, //utf8 string
    pub desc: ExternalKindType,
}

#[derive(Debug)]
pub enum ExternalKindType {
    Function { ty: u32 },
    Table { ty: u32 },
    Memory { ty: u32 },
    Global { ty: u32 },
}

#[derive(Debug)]
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

#[derive(Debug, PartialEq, Eq)]
pub struct TableType {
    pub element_type: u8, //0x70 for future ref
    pub limits: Limits,
}

#[derive(Debug, PartialEq, Eq)]
pub struct MemoryType {
    pub limits: Limits,
}

#[derive(Debug, PartialEq, Eq)]
pub struct GlobalType {
    pub value_type: ValueType,
    pub mu: Mu,
}

#[derive(Debug, PartialEq, Eq)]
pub enum Mu {
    Const,
    Var,
}

#[derive(Debug)]
pub struct GlobalVariable {
    pub ty: GlobalType,
    pub init: Expr,
}

#[derive(Debug)]
pub struct ExportEntry {
    pub name: String, //utf8 string
    pub kind: ExternalKindType,
    //index: varuint32,
}

#[derive(Debug)]
pub struct ElementSegment {
    pub index: u32,
    pub offset: Expr,
    pub elems: Vec<u32>,
}

#[derive(Debug)]
pub struct DataSegment {
    pub index: u32,
    pub offset: Expr,
    pub data: Vec<u8>,
}

#[derive(Debug)]
pub struct FunctionBody {
    pub locals: Vec<LocalEntry>,
    pub code: Expr,
}

#[derive(Debug)]
pub struct LocalEntry {
    pub count: u32,
    pub ty: ValueType,
}

#[derive(Debug, PartialEq, Eq)]
pub enum Limits {
    Zero(u32),
    One(u32, u32),
}

#[derive(Debug, PartialEq, Eq)]
pub struct Expr;


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
        match item {
            0x7F => Self::I32,
            0x7E => Self::I64,
            0x7D => Self::F32,
            0x7C => Self::F64,
            _ => panic!("wrong value type"),
        }
    }
}

impl std::convert::From<u8> for BlockType {
    fn from(item: u8) -> Self {
        match item {
            0x40 => Self::Empty,
            v => Self::ValueType(v.into()),
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
