#![feature(vec_drain_as_slice)]

extern crate log;

use log::debug;

mod core;
mod instructions;
pub mod isa;
mod leb128;

use self::core::*;
use self::leb128::*;

use nom::bytes::complete::take;
use nom::combinator::complete;
use nom::multi::{count, many0};
use nom::IResult;

use byteorder::{ByteOrder, LittleEndian};

pub const MAGIC_NUMBER: &'static [u8] = &[0, 97, 115, 109];
const END_INSTR: &[u8] = &[0x0B];

#[derive(Debug)]
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

pub fn parse(content: Vec<u8>) {
    let slice = content.as_slice();

    println!("{:#?}", parse_module(slice).unwrap());
}

fn parse_module(i: &[u8]) -> IResult<&[u8], Vec<Section>> {
    debug!("START {:?}", i);

    let (i, magic) = take_until_magic_number(i)?;

    assert_eq!(MAGIC_NUMBER, magic);

    let (i, _version) = take_version_number(i)?;

    let s = complete(many0(parse_section));

    let (i, k) = s(i)?;

    Ok((i, k))
}

fn parse_section(i: &[u8]) -> IResult<&[u8], Section> {
    debug!("parse_section {:?}", i);

    let (i, n) = take(1u8)(i)?;
    let (i, size) = take_leb_u32(i)?;

    println!("SECTION {:?} {:?}", n, size);

    debug!("{:?}", i);

    let (i, m) = match n[0] {
        0 => parse_custom_section(i, size)?,
        1 => parse_type_section(i, size)?,
        2 => parse_import_section(i, size)?,
        3 => parse_function_section(i, size)?,
        4 => parse_table_section(i, size)?,
        5 => parse_memory_section(i, size)?,
        6 => parse_global_section(i, size)?,
        7 => parse_export_section(i, size)?,
        8 => parse_start_section(i, size)?,
        9 => parse_element_section(i, size)?,
        10 => parse_code_section(i, size)?,
        11 => parse_data_section(i, size)?,
        _ => panic!("invalid section id"),
    };

    Ok((i, m))
}

fn take_until_magic_number(i: &[u8]) -> IResult<&[u8], &[u8]> {
    debug!("take_until_magic_number");
    take(4u8)(i)
}

fn take_version_number(i: &[u8]) -> IResult<&[u8], &[u8]> {
    debug!("take_version_number");
    take(4u8)(i)
}

fn parse_custom_section(i: &[u8], size: u32) -> IResult<&[u8], Section> {
    debug!("parse custom section");
    let (k, name) = take_name(i)?;

    let str_len = i.len() - k.len();

    let (i, _) = take(size as usize - str_len)(k)?; //consume empty bytes

    Ok((
        i,
        Section::Custom {
            name: name.to_string(),
        },
    ))
}

fn parse_type_section(i: &[u8], _size: u32) -> IResult<&[u8], Section> {
    debug!("parse type section");

    let (i, times) = take_leb_u32(i)?;
    let (i, vec) = count(take_functype, times as usize)(i)?;

    Ok((i, Section::Type { entries: vec }))
}

fn parse_import_section(i: &[u8], _size: u32) -> IResult<&[u8], Section> {
    debug!("parse import section");
    let (i, times) = take_leb_u32(i)?;
    let (i, import) = count(take_import, times as usize)(i)?;

    Ok((i, Section::Import { entries: import }))
}

fn parse_function_section(i: &[u8], _size: u32) -> IResult<&[u8], Section> {
    debug!("parse function section");
    let (i, times) = take_leb_u32(i)?;
    let (i, functions) = count(take_leb_u32, times as usize)(i)?;

    Ok((i, Section::Function { types: functions }))
}

fn parse_table_section(i: &[u8], _size: u32) -> IResult<&[u8], Section> {
    debug!("parse table function");
    let (i, times) = take_leb_u32(i)?;
    let (i, tables) = count(take_tabletype, times as usize)(i)?;

    Ok((i, Section::Table { entries: tables }))
}

fn parse_memory_section(i: &[u8], _size: u32) -> IResult<&[u8], Section> {
    debug!("parse memory function");
    let (i, times) = take_leb_u32(i)?;
    let (i, mem) = count(take_memtype, times as usize)(i)?;

    Ok((i, Section::Memory { entries: mem }))
}

fn parse_global_section(i: &[u8], _size: u32) -> IResult<&[u8], Section> {
    debug!("parse global function");
    let (i, times) = take_leb_u32(i)?;
    let (i, globals) = count(take_global, times as usize)(i)?;

    Ok((i, Section::Global { globals: globals }))
}

fn parse_export_section(i: &[u8], _size: u32) -> IResult<&[u8], Section> {
    debug!("parse export function");
    let (i, times) = take_leb_u32(i)?;
    let (i, exports) = count(take_export, times as usize)(i)?;

    Ok((i, Section::Export { entries: exports }))
}

fn parse_start_section(i: &[u8], _size: u32) -> IResult<&[u8], Section> {
    debug!("parse start function");
    let (i, func_idx) = take_leb_u32(i)?;

    Ok((i, Section::Start { index: func_idx }))
}

fn parse_element_section(i: &[u8], _size: u32) -> IResult<&[u8], Section> {
    debug!("parse_element_section");
    let (i, times) = take_leb_u32(i)?;
    let (i, elements) = count(take_elem, times as usize)(i)?;

    Ok((i, Section::Element { entries: elements }))
}

fn parse_data_section(i: &[u8], _size: u32) -> IResult<&[u8], Section> {
    debug!("parse_data_section");
    let (i, times) = take_leb_u32(i)?;
    let (i, k) = count(take_data, times as usize)(i)?;

    Ok((i, Section::Data { entries: k }))
}

fn parse_code_section(i: &[u8], size: u32) -> IResult<&[u8], Section> {
    debug!("parse_code_section");
    //let (i, _code) = take(size)(i)?;

    let (i, times) = take_leb_u32(i)?;
    let (i, codes) = count(take_code, times as usize)(i)?;

    Ok((i, Section::Code { entries: codes }))
}

fn take_code(i: &[u8]) -> IResult<&[u8], FunctionBody> {
    debug!("parse_code");

    let (i, times) = take_leb_u32(i)?;
    let (i, k) = take_func(i)?;

    Ok((i, k))
}

fn take_func(i: &[u8]) -> IResult<&[u8], FunctionBody> {
    debug!("take_func");

    let (i, times) = take_leb_u32(i)?;
    let (i, locals) = count(take_local, times as usize)(i)?;

    let (i, expr) = take_expr(i)?;

    Ok((
        i,
        FunctionBody {
            locals: locals,
            code: expr,
        },
    ))
}

fn take_local(i: &[u8]) -> IResult<&[u8], LocalEntry> {
    debug!("take_local");

    let (i, n) = take_leb_u32(i)?;
    let (i, t) = take_valtype(i)?;

    Ok((i, LocalEntry { count: n, ty: t }))
}

fn take_data(i: &[u8]) -> IResult<&[u8], DataSegment> {
    debug!("take_data");

    let (i, mem_idx) = take_leb_u32(i)?;
    let (i, e) = take_expr(i)?;

    let (i, times) = take_leb_u32(i)?;
    let (i, b) = count(take(1u8), times as usize)(i)?;

    Ok((
        i,
        DataSegment {
            index: mem_idx,
            offset: e,
            data: b.into_iter().map(|w| w[0]).collect(),
        },
    ))
}

fn take_elem(i: &[u8]) -> IResult<&[u8], ElementSegment> {
    debug!("take_elem");

    let (i, table_idx) = take_leb_u32(i)?;
    let (i, e) = take_expr(i)?;
    let (i, times) = take_leb_u32(i)?;
    let (i, y_vec) = count(take_leb_u32, times as usize)(i)?;

    Ok((
        i,
        ElementSegment {
            index: table_idx,
            offset: e,
            elems: y_vec,
        },
    ))
}

fn take_export(i: &[u8]) -> IResult<&[u8], ExportEntry> {
    debug!("take_export");

    let (i, name) = take_name(i)?;
    let (i, desc) = take_desc(i)?;

    Ok((
        i,
        ExportEntry {
            name: name,
            kind: desc,
        },
    ))
}

fn take_global(i: &[u8]) -> IResult<&[u8], GlobalVariable> {
    let (i, ty) = take_globaltype(i)?;
    let (i, e) = take_expr(i)?;

    Ok((i, GlobalVariable { ty, init: e }))
}

fn take_expr(i: &[u8]) -> IResult<&[u8], Expr> {
    let (i, ii) = instructions::parse_instr(i)?;
    let (i, e) = take(1u8)(i)?; //0x0B

    assert_eq!(e, END_INSTR);

    Ok((i, ii))
}

fn take_import(i: &[u8]) -> IResult<&[u8], ImportEntry> {
    debug!("take_import");

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
    debug!("take_desc");

    let (i, b) = take(1u8)(i)?;

    let (i, desc) = match b[0] {
        0x00 => {
            let (i, t) = take_leb_u32(&i)?;
            (i, ExternalKindType::Function { ty: t })
        }
        0x01 => {
            let (i, t) = take_leb_u32(&i)?;
            (i, ExternalKindType::Table { ty: t })
        }
        0x02 => {
            let (i, t) = take_leb_u32(&i)?;
            (i, ExternalKindType::Memory { ty: t })
        }
        0x03 => {
            let (i, t) = take_leb_u32(&i)?;
            (i, ExternalKindType::Global { ty: t })
        }
        _ => panic!("desc failed"),
    };

    Ok((i, desc))
}

fn take_memtype(i: &[u8]) -> IResult<&[u8], MemoryType> {
    debug!("take_memtype");
    let (i, l) = take_limits(i)?;

    Ok((i, MemoryType { limits: l }))
}

fn take_tabletype(i: &[u8]) -> IResult<&[u8], TableType> {
    debug!("take_tabletype");
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
    debug!("take_globaltype");
    let (i, val) = take_valtype(i)?;
    let (i, b) = take_byte(i, 1)?;

    let mu = match b[0] {
        0x00 => Mu::Const,
        0x01 => Mu::Var,
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
    debug!("take_limits");
    let (i, n) = take(1u8)(i)?;

    Ok(match n[0] {
        0x00 => {
            let (i, n) = take_leb_u32(i)?;

            (i, Limits::Zero(n))
        }
        0x01 => {
            let (i, n) = take_leb_u32(i)?;
            let (i, m) = take_leb_u32(i)?;

            (i, Limits::One(n, m))
        }
        _ => panic!("Limit has wrong tag"),
    })
}

fn take_functype(i: &[u8]) -> IResult<&[u8], FuncType> {
    debug!("take_functype");

    let (i, offset) = take(1u8)(i)?; //0x60

    assert_eq!(offset[0], 0x60);

    let (i, times) = take_leb_u32(i)?;
    let (i, t1) = count(take(1u8), times as usize)(i)?;
    let (i, times) = take_leb_u32(i)?;
    let (i, t2) = count(take(1u8), times as usize)(i)?;

    let parameters: Vec<_> = t1.into_iter().map(|w| w[0].into()).collect();
    let return_types: Vec<_> = t2.into_iter().map(|w| w[0].into()).collect();

    Ok((
        i,
        FuncType {
            param_types: parameters,
            return_types: return_types,
        },
    ))
}

fn take_valtype(i: &[u8]) -> IResult<&[u8], ValueType> {
    debug!("take_valtype");
    let (i, n) = take(1u8)(i)?;

    Ok((i, n[0].into()))
}

fn take_blocktype(i: &[u8]) -> IResult<&[u8], BlockType> {
    debug!("take_blocktype");
    let (i, n) = take(1u8)(i)?;

    Ok((i, n[0].into()))
}

fn take_byte(i: &[u8], len: u32) -> IResult<&[u8], &[u8]> {
    debug!("take_byte");
    take(len as usize)(i)
}

fn take_f32(i: &[u8]) -> IResult<&[u8], f32> {
    debug!("take_f32");
    let (i, bytes) = take(4u8)(i)?;

    Ok((i, LittleEndian::read_f32(bytes)))
}

fn take_f64(i: &[u8]) -> IResult<&[u8], f64> {
    debug!("take_f64");
    let (i, bytes) = take(8u8)(i)?;

    Ok((i, LittleEndian::read_f64(bytes)))
}

fn take_name(i: &[u8]) -> IResult<&[u8], String> {
    debug!("take_name");
    let (i, times) = take_leb_u32(i)?;
    let (i, vec) = count(take(1u8), times as usize)(i)?;

    let vec2: Vec<_> = vec.into_iter().map(|w| w[0]).collect();

    Ok((i, String::from_utf8(vec2).unwrap()))
}

pub(crate) fn take_leb_u32(i: &[u8]) -> IResult<&[u8], u32> {
    debug!("take_leb_u32");
    let (_, bytes) = take(4u8)(i)?;
    let leb = read_u32_leb128(bytes);
    let (i, _) = take(leb.1)(i)?; //skip the bytes, which contain leb

    Ok((i, leb.0))
}

fn take_leb_u8(i: &[u8]) -> IResult<&[u8], u8> {
    debug!("take_leb_u8");
    let (_, bytes) = take(1u8)(i)?;
    let leb = read_u8_leb128(bytes);
    let (i, _) = take(leb.1)(i)?;

    Ok((i, leb.0))
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

    /*
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
    */

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
            mu: Mu::_const,
        };

        assert_eq!(g, take_globaltype(payload.as_slice()).unwrap().1);
    }

    #[test]
    fn test_globaltype_var() {
        let mut payload = vec![0x7F as u8, 0x1];

        let g = GlobalType {
            value_type: ValueType::I32,
            mu: Mu::_var,
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
            limits: Limits::zero(VarUInt32(4)),
        };

        assert_eq!(t, take_tabletype(payload.as_slice()).unwrap().1);
    }
}
