use std::io::Cursor;
use byteorder::{LittleEndian, ReadBytesExt};
use crate::leb128::*;

#[derive(Debug, Clone)]
pub struct VarUInt1(pub u8);
#[derive(Debug, Clone)]
pub struct VarInt7(pub u8);
#[derive(Debug, Clone)]
pub struct VarUInt8(pub u8);
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct VarUInt32(pub u32);

impl VarUInt32 {
    pub fn get_u32(&self) -> u32 {
        let mut v : Vec<u8> = Vec::new();
        write_u32_leb128(&mut v, self.0);

        let mut rdr = Cursor::new(v);

        rdr.read_u32::<LittleEndian>().unwrap()
    }

    pub fn get_usize(&self) -> usize {
        self.0 as usize
    }
}

type varuint32 = VarUInt32;
type varint7 = VarInt7;
type varuint1 = VarUInt1;
type varuint8 = VarUInt8;

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
    empty,
    value_type(ValueType),
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
    Function { ty: varuint32 },
    Table { ty: TableType },
    Memory { ty: MemoryType },
    Global { ty: GlobalType },
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
        types: Vec<varuint32>,
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
        index: varuint32,
    },
    Element {
        entries: Vec<ElementSegment>,
    },
    Code {
        bodies: Vec<FunctionBody>,
    },
    Data {
        entries: Vec<DataSegment>,
    },
    Name {
        name_type: varint7,
        name_payload_len: varuint32,
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
    _const,
    _var,
}

#[derive(Debug)]
pub struct GlobalVariable {
    pub ty: GlobalType,
    pub init: InitExpr,
}

#[derive(Debug)]
pub struct ExportEntry {
    pub name: String, //utf8 string
    pub kind: ExternalKindType,
    //index: varuint32,
}

#[derive(Debug)]
pub struct ElementSegment {
    pub index: varuint32,
    pub offset: InitExpr,
    pub elems: Vec<varuint32>,
}

#[derive(Debug)]
pub struct DataSegment {
    pub index: varuint32,
    pub offset: InitExpr,
    pub data: Vec<u8>,
}

#[derive(Debug)]
pub struct FunctionBody {
    body_size: varuint32,
    local_count: varuint32,
    locals: Vec<LocalEntry>,
    code: Vec<u8>,
    end: u8, //0x0b
}

#[derive(Debug)]
pub struct LocalEntry {
    count: varuint32,
    ty: ValueType,
}

#[derive(Debug, PartialEq, Eq)]
pub enum Limits {
    zero(varuint32),
    one(varuint32, varuint32),
}

/*
pub struct FuncType<T, W> {
    t1: VecTy<T>,
    t1_ty: ValueType,
    t2: VecTy<W>,
    t2_ty: ValueType,
}
*/

#[derive(Debug, PartialEq, Eq)]
pub struct InitExpr;

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
            0x40 => Self::empty,
            (v) => Self::value_type(v.into()),
        }
    }
}

impl std::convert::From<u8> for Mu {
    fn from(item: u8) -> Self {
        match item {
            0x00 => Self::_const,
            0x01 => Self::_var,
            _ => panic!("Mu failed"),
        }
    }
}
