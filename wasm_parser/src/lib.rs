#![feature(vec_drain_as_slice)]

#[macro_use]
extern crate log;

use log::{debug, info, warn};

mod core;
mod leb128;

use self::core::*;
//use crate::core::{VarInt7, VarUInt32, VarUInt1};
use self::leb128::*;

use nom::bytes::complete::{take, take_until};
use nom::combinator::{complete, opt};
use nom::multi::{length_data, many0};
use nom::number::complete::be_u16;
use nom::number::complete::be_u32;
use nom::{alt, eof, named};
//use nom::sequence::preceded;
use nom::preceded;
use nom::IResult;

use byteorder::{ByteOrder, LittleEndian};

pub const MAGIC_NUMBER: &'static [u8] = &[0, 97, 115, 109];

struct Module {
    sections: Vec<Section>,
}

#[macro_export]
macro_rules! read_wasm {
    ($fs_name:expr) => {{
        use std::fs::File;
        use std::io::prelude::*;

        let mut fs = File::open($fs_name).unwrap();
        let mut reader = Vec::new();

        fs.read_to_end(&mut reader).unwrap();

        reader
    }};
}

pub fn parse(mut content: Vec<u8>) {
    let slice = content.as_slice();

    parse_module(slice).unwrap();
}

fn parse_module(i: &[u8]) -> IResult<&[u8], Vec<Section>> {
    use nom::sequence::preceded;

    let magic = take_until_magic_number(i).unwrap();
    let version = take_version_number(i).unwrap();

    let s = complete(many0(parse_section));

    let k = s(i)?.1;

    Ok((i, k))
}

fn parse_section(i: &[u8]) -> IResult<&[u8], Section> {
    let N = take(1u8)(i)?.1;
    let size = take_leb_u32(i)?.1;

    Ok((
        i,
        match N[0] {
            0 => parse_custom_section(i, size)?.1,
            1 => parse_type_section(i, size)?.1,
            2 => parse_import_section(i, size)?.1,
            3 => parse_function_section(i, size)?.1,
            4 => parse_table_section(i, size)?.1,
            5 => parse_memory_section(i, size)?.1,
            6 => parse_global_section(i, size)?.1,
            7 => parse_export_section(i, size)?.1,
            8 => parse_start_section(i, size)?.1,
            9 => parse_element_section(i, size)?.1,
            10 => parse_code_section(i, size)?.1,
            11 => parse_data_section(i, size)?.1,
            _ => panic!("invalid section id"),
        },
    ))
}

fn take_until_magic_number(i: &[u8]) -> IResult<&[u8], &[u8]> {
    nom::bytes::complete::take_until(MAGIC_NUMBER)(i)
}

fn take_version_number(i: &[u8]) -> IResult<&[u8], &[u8]> {
    take(4u8)(i)
}

fn take_vec(i: &[u8]) -> IResult<&[u8], &[u8]> {
    use byteorder::{LittleEndian, ReadBytesExt};

    let (i, mut n) = take(1u32)(i)?;

    let k = read_u32_leb128(n).0;

    take(k)(i)
}

fn parse_custom_section(i: &[u8], size: VarUInt32) -> IResult<&[u8], Section> {
    let (len, name) = take_name(i)?;

    let len = LittleEndian::read_u32(len);

    let (i, _) = take(size.get_u32() - len)(i)?; //consume empty bytes

    Ok((
        i,
        Section::Custom {
            name: name.to_string(),
        },
    ))
}

fn parse_type_section(i: &[u8], size: VarUInt32) -> IResult<&[u8], Section> {
    let (i, vec) = take_vec(i)?;
    let (i, entries) = many0(take_functype)(vec)?;

    Ok((i, Section::Type { entries: entries }))
}

fn parse_import_section(i: &[u8], size: VarUInt32) -> IResult<&[u8], Section> {
    let (i, imports) = take_vec(i)?;

    let (i, import) = many0(take_import)(imports)?;

    Ok((i, Section::Import { entries: import }))
}

fn parse_function_section(i: &[u8], size: VarUInt32) -> IResult<&[u8], Section> {
    let (i, functions_v) = take_vec(i)?;

    let (i, functions) = many0(take_leb_u32)(functions_v)?;

    Ok((i, Section::Function { types: functions }))
}

fn parse_table_section(i: &[u8], size: VarUInt32) -> IResult<&[u8], Section> {
    let (i, vec) = take_vec(i)?;

    let (i, tables) = many0(take_tabletype)(vec)?;

    Ok((i, Section::Table { entries: tables }))
}

fn parse_memory_section(i: &[u8], size: VarUInt32) -> IResult<&[u8], Section> {
    let (i, vec) = take_vec(i)?;

    let (i, mem) = many0(take_memtype)(vec)?;

    Ok((i, Section::Memory { entries: mem }))
}

fn parse_global_section(i: &[u8], size: VarUInt32) -> IResult<&[u8], Section> {
    let (i, vec) = take_vec(i)?;

    let (i, tables) = many0(take_tabletype)(vec)?;

    Ok((i, Section::Table { entries: tables }))
}

fn parse_export_section(i: &[u8], size: VarUInt32) -> IResult<&[u8], Section> {
    let (i, vec) = take_vec(i)?;

    let (i, exports) = many0(take_export)(vec)?;

    Ok((i, Section::Export { entries: exports }))
}

fn parse_start_section(i: &[u8], size: VarUInt32) -> IResult<&[u8], Section> {
    let (i, func_idx) = take_leb_u32(i)?;

    Ok((i, Section::Start { index: func_idx }))
}

fn parse_element_section(i: &[u8], size: VarUInt32) -> IResult<&[u8], Section> {
    let (i, vec) = take_vec(i)?;

    let (i, elements) = many0(take_elem)(i)?;

    Ok((i, Section::Element { entries: elements }))
}

fn parse_data_section(i: &[u8], size: VarUInt32) -> IResult<&[u8], Section> {
    let (i, vec) = take_vec(i)?;
    let (i, k) = many0(take_data)(vec)?;

    Ok((i, Section::Data { entries: k }))
}

fn parse_code_section(i: &[u8], size: VarUInt32) -> IResult<&[u8], Section> {
    let len = size.get_u32();

    //TODO
    let (i, _) = take(len)(i)?;

    Ok((
        i,
        Section::Custom {
            name: "code".to_string(),
        },
    ))
}

fn take_data(i: &[u8]) -> IResult<&[u8], DataSegment> {
    let (i, mem_idx) = take_leb_u32(i)?;
    let (i, e) = take_expr(i)?;
    let (i, b) = take_vec(i)?;

    Ok((
        i,
        DataSegment {
            index: mem_idx,
            offset: e,
            data: b.to_vec(),
        },
    ))
}

fn take_elem(i: &[u8]) -> IResult<&[u8], ElementSegment> {
    let (i, table_idx) = take_leb_u32(i)?;
    let (i, e) = take_expr(i)?;
    let (i, y_vec) = take_vec(i)?;

    let (i, y) = many0(take_leb_u32)(i)?;

    Ok((
        i,
        ElementSegment {
            index: table_idx,
            offset: e,
            elems: y,
        },
    ))
}

fn take_export(i: &[u8]) -> IResult<&[u8], ExportEntry> {
    let (i, name) = take_name(i)?;
    let (i, desc) = take_desc(i)?;

    Ok((
        i,
        ExportEntry {
            name: name.to_string(),
            kind: desc,
        },
    ))
}

fn take_global(i: &[u8]) -> IResult<&[u8], GlobalVariable> {
    let (i, ty) = take_globaltype(i)?;
    let (i, e) = take_expr(i)?;

    Ok((i, GlobalVariable { ty, init: e }))
}

fn take_expr(i: &[u8]) -> IResult<&[u8], InitExpr> {
    unimplemented!("todo")
}

fn take_import(i: &[u8]) -> IResult<&[u8], ImportEntry> {
    let (i, mod_name) = take_name(i)?;
    let (i, name) = take_name(i)?;
    let (i, desc) = take_desc(i)?;

    Ok((
        i,
        ImportEntry {
            module_name: mod_name.to_string(),
            name: name.to_string(),
            desc,
        },
    ))
}

fn take_desc(i: &[u8]) -> IResult<&[u8], ExternalKindType> {
    let (i, b) = take(1u8)(i)?;

    let (i, desc) = match b[0] {
        0x00 => {
            let (i, t) = take_leb_u32(&i)?;
            (i, ExternalKindType::Function { ty: t })
        }
        0x01 => {
            let (i, t) = take_tabletype(&i)?;
            (i, ExternalKindType::Table { ty: t })
        }
        0x02 => {
            let (i, t) = take_memtype(&i)?;
            (i, ExternalKindType::Memory { ty: t })
        }
        0x03 => {
            let (i, t) = take_globaltype(&i)?;
            (i, ExternalKindType::Global { ty: t })
        }
        _ => panic!("desc failed"),
    };

    Ok((i, desc))
}

fn take_memtype(i: &[u8]) -> IResult<&[u8], MemoryType> {
    let (i, l) = take_limits(i)?;

    Ok((i, MemoryType { limits: l }))
}

fn take_tabletype(i: &[u8]) -> IResult<&[u8], TableType> {
    let (i, element_type) = take(1u8)(i)?;

    assert_eq!(0x70, element_type[0]);

    let (i, limits) = take_limits(i)?;

    Ok((
        i,
        TableType {
            element_type: 0x70,
            limits: limits,
        },
    ))
}

fn take_globaltype(i: &[u8]) -> IResult<&[u8], GlobalType> {
    let (i, val) = take_valtype(i)?;

    let (i, b) = take_byte(i, 1)?;

    let mu = match b[0] {
        0x00 => Mu::_const,
        0x01 => Mu::_var,
        _ => panic!("wrong mu"),
    };

    Ok((
        i,
        GlobalType {
            value_type: val.into(),
            mu: mu,
        },
    ))
}

fn take_limits(i: &[u8]) -> IResult<&[u8], Limits> {
    let (i, n) = take(1u8)(i)?;

    Ok((
        i,
        match n[0] {
            0x00 => {
                let (i, n) = take_leb_u32(i)?;
                println!("n {:?}", n);

                Limits::zero(n)
            }
            0x01 => {
                let (i, n) = take_leb_u32(i)?;
                let (i, m) = take_leb_u32(i)?;
                println!("n {:?}", n);
                println!("m {:?}", m);

                Limits::one(n, m)
            }
            _ => panic!("Limit has wrong tag"),
        },
    ))
}

fn take_functype(i: &[u8]) -> IResult<&[u8], FuncType> {
    let (i, offset) = take(1u8)(i)?; //0x60

    assert_eq!(offset[0], 0x60);

    let (i, mut t1) = take_vec(i)?;

    let (i, mut t2) = take_vec(i)?;

    let parameters: Vec<ValueType> = t1.to_vec().into_iter().map(|w| w.into()).collect();
    let return_types: Vec<ValueType> = t2.to_vec().into_iter().map(|w| w.into()).collect();

    Ok((
        i,
        FuncType {
            param_types: parameters,
            return_types: return_types,
        },
    ))
}

fn take_valtype(i: &[u8]) -> IResult<&[u8], ValueType> {
    let (i, mut n) = take(1u8)(i)?;

    Ok((i, n[0].into()))
}

fn take_blocktype(i: &[u8]) -> IResult<&[u8], BlockType> {
    let (i, mut n) = take(1u8)(i)?;

    Ok((i, n[0].into()))
}

fn take_size(i: &[u8]) -> IResult<&[u8], &[u8]> {
    take(4u8)(i)
}

fn take_byte(i: &[u8], len: u32) -> IResult<&[u8], &[u8]> {
    take(len as usize)(i)
}

fn take_f32(i: &[u8]) -> IResult<&[u8], f32> {
    let (i, bytes) = take(4u8)(i)?;

    Ok((i, LittleEndian::read_f32(bytes)))
}

fn take_f64(i: &[u8]) -> IResult<&[u8], f64> {
    let (i, bytes) = take(8u8)(i)?;

    Ok((i, LittleEndian::read_f64(bytes)))
}

fn take_name(i: &[u8]) -> IResult<&[u8], &str> {
    let (i, vec) = take_vec(i)?;

    Ok((i, std::str::from_utf8(&vec).unwrap()))
}

fn take_leb_u32(i: &[u8]) -> IResult<&[u8], VarUInt32> {
    let (i, bytes) = take(4u8)(i)?;

    let leb = read_u32_leb128(bytes);

    Ok((i, VarUInt32(leb.0)))
}

fn take_leb_u8(i: &[u8]) -> IResult<&[u8], VarUInt8> {
    let (i, bytes) = take(1u8)(i)?;

    let leb = read_u8_leb128(bytes);

    Ok((i, VarUInt8(leb.0)))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_f32() {
        let n: f32 = 32.2;

        let nn = n.to_le_bytes();
        let nn2 = nn.to_vec();

        assert_eq!(32.2, take_f32(&nn2).unwrap().1);
    }

    #[test]
    fn test_parse_f64() {
        let n: f64 = 32.2;

        let nn = n.to_le_bytes();
        let nn2 = nn.to_vec();

        assert_eq!(32.2, take_f64(&nn2).unwrap().1);
    }

    #[test]
    fn test_parse_vec() {
        let k = 4 as u32;

        let mut kk = Vec::new();
        write_u32_leb128(&mut kk, k);
        let mut w = kk.to_vec();

        let mut payload = vec![1, 2, 3, 4];

        w.extend(payload);

        assert_eq!(vec![1, 2, 3, 4], take_vec(&w).unwrap().1);
    }

    #[test]
    fn test_parse_name() {
        let k = 4 as u32;

        let mut kk = Vec::new();
        write_u32_leb128(&mut kk, k);
        let mut w = kk.to_vec();

        let n = "test";
        let mut nn = n.as_bytes();

        let nn2 = nn.to_vec();

        w.extend(nn2);

        assert_eq!("test", take_name(&w).unwrap().1); //reverse
    }

    #[test]
    fn test_limit_zero() {
        let k = 4 as u32;

        let mut kk = Vec::new();
        write_u32_leb128(&mut kk, k);
        let mut w = kk.to_vec();

        let mut payload = vec![0 as u8];

        w.push(0);
        w.push(0);
        w.push(0);

        payload.extend(w);

        assert_eq!(Limits::zero(VarUInt32(4)), take_limits(&payload).unwrap().1);
    }

    #[test]
    fn test_limit_one() {
        let k = 4 as u32;
        let k2 = 8 as u32;

        let mut kk = Vec::new();
        let mut kk2 = Vec::new();

        write_u32_leb128(&mut kk, k);
        write_u32_leb128(&mut kk2, k2);

        let mut w = kk.to_vec();
        let mut w2 = kk2.to_vec();

        w.push(0);
        w.push(0);
        w.push(0);

        w2.push(0);
        w2.push(0);
        w2.push(0);

        let mut payload = vec![1 as u8];

        payload.extend(w);
        payload.extend(w2);

        assert_eq!(
            Limits::one(VarUInt32(4), VarUInt32(8)),
            take_limits(&payload).unwrap().1
        );
    }

    #[test]
    fn test_func_types() {
        let mut payload = vec![0x60 as u8];

        let mut kk = Vec::new();
        write_u32_leb128(&mut kk, 2); //size

        payload.extend(kk.clone());

        payload.push(0x7F); //i32
        payload.push(0x7F); //i32

        payload.extend(kk); //second vec

        payload.push(0x7F); //i32
        payload.push(0x7F); //i32

        let expected = FuncType {
            param_types: vec![ValueType::I32, ValueType::I32],
            return_types: vec![ValueType::I32, ValueType::I32],
        };

        assert_eq!(expected, take_functype(payload.as_slice()).unwrap().1);
    }

    #[test]
    fn test_blocktype_empty() {
        let mut payload = vec![0x40 as u8];

        assert_eq!(
            BlockType::empty,
            take_blocktype(payload.as_slice()).unwrap().1
        );
    }

    #[test]
    fn test_blocktype_valtype() {
        let mut payload = vec![0x7F as u8];

        assert_eq!(
            BlockType::value_type(ValueType::I32),
            take_blocktype(payload.as_slice()).unwrap().1
        );
    }

    #[test]
    fn test_globaltype_const() {
        let mut payload = vec![0x7F as u8, 0x0];

        let g = GlobalType {
            value_type: ValueType::I32,
            mu: Mu::_const
        };

        assert_eq!(g, take_globaltype(payload.as_slice()).unwrap().1);
    }

    #[test]
    fn test_globaltype_var() {
        let mut payload = vec![0x7F as u8, 0x1];

        let g = GlobalType {
            value_type: ValueType::I32,
            mu: Mu::_var
        };

        assert_eq!(g, take_globaltype(payload.as_slice()).unwrap().1);

    }

    #[test]
    fn test_tabletype() {
        let mut payload = vec![0x70 as u8, 0x00 as u8];

        let k = 4 as u32;

        let mut kk = Vec::new();

        write_u32_leb128(&mut kk, k);

        let mut w = kk.to_vec();

        w.push(0);
        w.push(0);
        w.push(0);

        payload.extend(w);

        let t = TableType {
            element_type: 0x70,
            limits: Limits::zero(VarUInt32(4))
        };

        assert_eq!(t, take_tabletype(payload.as_slice()).unwrap().1);
    }
}
