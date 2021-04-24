use log::debug;
use nom::bytes::complete::take;
use nom::multi::count;
use nom::IResult;

use crate::core::*;
use crate::{take_blocktype, take_f32, take_f64, take_leb_i32, take_leb_i64};

const END_INSTR: &[u8] = &[0x0B];
const END_IF_BLOCK: &[u8] = &[0x05];

pub(crate) fn parse_instr<'a, 'b>(
    i: &'a [u8],
    counter: &'b mut Counter,
) -> IResult<&'a [u8], Instruction> {
    debug!("parse_instr");
    debug!("---------------");
    let (i, instr) = take(1u8)(i)?;
    debug!("HEAD {:x?}", instr);

    let (i, expr) = match instr[0] {
        0x00 => (i, Instruction::OP_UNREACHABLE),
        0x01 => (i, Instruction::OP_NOP),
        0x02 => take_block(i, counter)?,
        0x03 => take_loop(i, counter)?,
        0x04 => take_conditional(i, counter)?,
        0x0C => take_br(i)?,
        0x0D => take_br_if(i)?,
        0x0E => take_br_table(i)?,
        0x0F => (i, Instruction::OP_RETURN),
        0x10 => take_call(i)?,
        0x11 => take_call_indirect(i)?,
        // Parametric
        0x1A => (i, Instruction::OP_DROP),
        0x1B => (i, Instruction::OP_SELECT),
        // Var
        0x20 => {
            let (i, idx) = crate::take_leb_u32(i)?;
            let block = Instruction::OP_LOCAL_GET(idx);
            (i, block)
        }
        0x21 => {
            let (i, idx) = crate::take_leb_u32(i)?;
            let block = Instruction::OP_LOCAL_SET(idx);
            (i, block)
        }
        0x22 => {
            let (i, idx) = crate::take_leb_u32(i)?;
            let block = Instruction::OP_LOCAL_TEE(idx);
            (i, block)
        }
        0x23 => {
            let (i, idx) = crate::take_leb_u32(i)?;
            let block = Instruction::OP_GLOBAL_GET(idx);
            (i, block)
        }
        0x24 => {
            let (i, idx) = crate::take_leb_u32(i)?;
            let block = Instruction::OP_GLOBAL_SET(idx);
            (i, block)
        }
        // Memory
        0x28 => {
            let (i, m) = take_memarg(i)?;
            let block = Instruction::OP_I32_LOAD(m);
            (i, block)
        }
        0x29 => {
            let (i, m) = take_memarg(i)?;
            let block = Instruction::OP_I64_LOAD(m);
            (i, block)
        }
        0x2A => {
            let (i, m) = take_memarg(i)?;
            let block = Instruction::OP_F32_LOAD(m);
            (i, block)
        }
        0x2B => {
            let (i, m) = take_memarg(i)?;
            let block = Instruction::OP_F64_LOAD(m);
            (i, block)
        }
        0x2C => {
            let (i, m) = take_memarg(i)?;
            let block = Instruction::OP_I32_LOAD_8_s(m);
            (i, block)
        }
        0x2D => {
            let (i, m) = take_memarg(i)?;
            let block = Instruction::OP_I32_LOAD_8_u(m);
            (i, block)
        }
        0x2E => {
            let (i, m) = take_memarg(i)?;
            let block = Instruction::OP_I32_LOAD_16_s(m);
            (i, block)
        }
        0x2F => {
            let (i, m) = take_memarg(i)?;
            let block = Instruction::OP_I32_LOAD_16_u(m);
            (i, block)
        }
        0x30 => {
            let (i, m) = take_memarg(i)?;
            let block = Instruction::OP_I64_LOAD_8_s(m);
            (i, block)
        }
        0x31 => {
            let (i, m) = take_memarg(i)?;
            let block = Instruction::OP_I64_LOAD_8_u(m);
            (i, block)
        }
        0x32 => {
            let (i, m) = take_memarg(i)?;
            let block = Instruction::OP_I64_LOAD_16_s(m);
            (i, block)
        }
        0x33 => {
            let (i, m) = take_memarg(i)?;
            let block = Instruction::OP_I64_LOAD_16_u(m);
            (i, block)
        }
        0x34 => {
            let (i, m) = take_memarg(i)?;
            let block = Instruction::OP_I64_LOAD_32_s(m);
            (i, block)
        }
        0x35 => {
            let (i, m) = take_memarg(i)?;
            let block = Instruction::OP_I64_LOAD_32_u(m);
            (i, block)
        }
        0x36 => {
            let (i, m) = take_memarg(i)?;
            let block = Instruction::OP_I32_STORE(m);
            (i, block)
        }
        0x37 => {
            let (i, m) = take_memarg(i)?;
            let block = Instruction::OP_I64_STORE(m);
            (i, block)
        }
        0x38 => {
            let (i, m) = take_memarg(i)?;
            let block = Instruction::OP_F32_STORE(m);
            (i, block)
        }
        0x39 => {
            let (i, m) = take_memarg(i)?;
            let block = Instruction::OP_F64_STORE(m);
            (i, block)
        }
        0x3a => {
            let (i, m) = take_memarg(i)?;
            let block = Instruction::OP_I32_STORE_8(m);
            (i, block)
        }
        0x3b => {
            let (i, m) = take_memarg(i)?;
            let block = Instruction::OP_I32_STORE_16(m);
            (i, block)
        }
        0x3c => {
            let (i, m) = take_memarg(i)?;
            let block = Instruction::OP_I64_STORE_8(m);
            (i, block)
        }
        0x3d => {
            let (i, m) = take_memarg(i)?;
            let block = Instruction::OP_I64_STORE_16(m);
            (i, block)
        }
        0x3e => {
            let (i, m) = take_memarg(i)?;
            let block = Instruction::OP_I64_STORE_32(m);
            (i, block)
        }
        0x3f => {
            let (i, m) = take(1u8)(i)?;
            assert_eq!([0x00], m);
            let block = Instruction::OP_MEMORY_SIZE;
            (i, block)
        }
        0x40 => {
            let (i, m) = take(1u8)(i)?;
            assert_eq!([0x00], m);
            let block = Instruction::OP_MEMORY_GROW;
            (i, block)
        }
        // Numeric Instructions
        0x41 => {
            let (i, m) = take_leb_i32(i)?;
            let block = Instruction::OP_I32_CONST(m);
            (i, block)
        }
        0x42 => {
            let (i, m) = take_leb_i64(i)?;
            let block = Instruction::OP_I64_CONST(m);
            (i, block)
        }
        0x43 => {
            let (i, m) = take_f32(i)?;
            let block = Instruction::OP_F32_CONST(m);
            (i, block)
        }
        0x44 => {
            let (i, m) = take_f64(i)?;
            let block = Instruction::OP_F64_CONST(m);
            (i, block)
        }
        0x45 => (i, Instruction::OP_I32_EQZ),
        0x46 => (i, Instruction::OP_I32_EQ),
        0x47 => (i, Instruction::OP_I32_NE),
        0x48 => (i, Instruction::OP_I32_LT_S),
        0x49 => (i, Instruction::OP_I32_LT_U),
        0x4a => (i, Instruction::OP_I32_GT_S),
        0x4b => (i, Instruction::OP_I32_GT_U),
        0x4c => (i, Instruction::OP_I32_LE_S),
        0x4d => (i, Instruction::OP_I32_LE_U),
        0x4e => (i, Instruction::OP_I32_GE_S),
        0x4f => (i, Instruction::OP_I32_GE_U),
        0x50 => (i, Instruction::OP_I64_EQZ),
        0x51 => (i, Instruction::OP_I64_EQ),
        0x52 => (i, Instruction::OP_I64_NE),
        0x53 => (i, Instruction::OP_I64_LT_S),
        0x54 => (i, Instruction::OP_I64_LT_U),
        0x55 => (i, Instruction::OP_I64_GT_S),
        0x56 => (i, Instruction::OP_I64_GT_U),
        0x57 => (i, Instruction::OP_I64_LE_S),
        0x58 => (i, Instruction::OP_I64_LE_U),
        0x59 => (i, Instruction::OP_I64_GE_S),
        0x5a => (i, Instruction::OP_I64_GE_U),

        0x5b => (i, Instruction::OP_F32_EQ),
        0x5c => (i, Instruction::OP_F32_NE),
        0x5d => (i, Instruction::OP_F32_LT),
        0x5e => (i, Instruction::OP_F32_GT),
        0x5f => (i, Instruction::OP_F32_LE),
        0x60 => (i, Instruction::OP_F32_GE),

        0x61 => (i, Instruction::OP_F64_EQ),
        0x62 => (i, Instruction::OP_F64_NE),
        0x63 => (i, Instruction::OP_F64_LT),
        0x64 => (i, Instruction::OP_F64_GT),
        0x65 => (i, Instruction::OP_F64_LE),
        0x66 => (i, Instruction::OP_F64_GE),

        0x67 => (i, Instruction::OP_I32_CLZ),
        0x68 => (i, Instruction::OP_I32_CTZ),
        0x69 => (i, Instruction::OP_I32_POPCNT),
        0x6a => (i, Instruction::OP_I32_ADD),
        0x6b => (i, Instruction::OP_I32_SUB),
        0x6c => (i, Instruction::OP_I32_MUL),
        0x6d => (i, Instruction::OP_I32_DIV_S),
        0x6e => (i, Instruction::OP_I32_DIV_U),
        0x6f => (i, Instruction::OP_I32_REM_S),
        0x70 => (i, Instruction::OP_I32_REM_U),
        0x71 => (i, Instruction::OP_I32_AND),
        0x72 => (i, Instruction::OP_I32_OR),
        0x73 => (i, Instruction::OP_I32_XOR),
        0x74 => (i, Instruction::OP_I32_SHL),
        0x75 => (i, Instruction::OP_I32_SHR_S),
        0x76 => (i, Instruction::OP_I32_SHR_U),
        0x77 => (i, Instruction::OP_I32_ROTL),
        0x78 => (i, Instruction::OP_I32_ROTR),

        0x79 => (i, Instruction::OP_I64_CLZ),
        0x7a => (i, Instruction::OP_I64_CTZ),
        0x7b => (i, Instruction::OP_I64_POPCNT),
        0x7c => (i, Instruction::OP_I64_ADD),
        0x7d => (i, Instruction::OP_I64_SUB),
        0x7e => (i, Instruction::OP_I64_MUL),
        0x7f => (i, Instruction::OP_I64_DIV_S),
        0x80 => (i, Instruction::OP_I64_DIV_U),
        0x81 => (i, Instruction::OP_I64_REM_S),
        0x82 => (i, Instruction::OP_I64_REM_U),
        0x83 => (i, Instruction::OP_I64_AND),
        0x84 => (i, Instruction::OP_I64_OR),
        0x85 => (i, Instruction::OP_I64_XOR),
        0x86 => (i, Instruction::OP_I64_SHL),
        0x87 => (i, Instruction::OP_I64_SHR_S),
        0x88 => (i, Instruction::OP_I64_SHR_U),
        0x89 => (i, Instruction::OP_I64_ROTL),
        0x8a => (i, Instruction::OP_I64_ROTR),

        0x8b => (i, Instruction::OP_F32_ABS),
        0x8c => (i, Instruction::OP_F32_NEG),
        0x8d => (i, Instruction::OP_F32_CEIL),
        0x8e => (i, Instruction::OP_F32_FLOOR),
        0x8f => (i, Instruction::OP_F32_TRUNC),
        0x90 => (i, Instruction::OP_F32_NEAREST),
        0x91 => (i, Instruction::OP_F32_SQRT),
        0x92 => (i, Instruction::OP_F32_ADD),
        0x93 => (i, Instruction::OP_F32_SUB),
        0x94 => (i, Instruction::OP_F32_MUL),
        0x95 => (i, Instruction::OP_F32_DIV),
        0x96 => (i, Instruction::OP_F32_MIN),
        0x97 => (i, Instruction::OP_F32_MAX),
        0x98 => (i, Instruction::OP_F32_COPYSIGN),

        0x99 => (i, Instruction::OP_F64_ABS),
        0x9a => (i, Instruction::OP_F64_NEG),
        0x9b => (i, Instruction::OP_F64_CEIL),
        0x9c => (i, Instruction::OP_F64_FLOOR),
        0x9d => (i, Instruction::OP_F64_TRUNC),
        0x9e => (i, Instruction::OP_F64_NEAREST),
        0x9f => (i, Instruction::OP_F64_SQRT),
        0xa0 => (i, Instruction::OP_F64_ADD),
        0xa1 => (i, Instruction::OP_F64_SUB),
        0xa2 => (i, Instruction::OP_F64_MUL),
        0xa3 => (i, Instruction::OP_F64_DIV),
        0xa4 => (i, Instruction::OP_F64_MIN),
        0xa5 => (i, Instruction::OP_F64_MAX),
        0xa6 => (i, Instruction::OP_F64_COPYSIGN),

        0xa7 => (i, Instruction::OP_I32_WRAP_I64),
        0xa8 => (i, Instruction::OP_I32_TRUNC_F32_S),
        0xa9 => (i, Instruction::OP_I32_TRUNC_F32_U),
        0xaa => (i, Instruction::OP_I32_TRUNC_F64_S),
        0xab => (i, Instruction::OP_I32_TRUNC_F64_U),
        0xac => (i, Instruction::OP_I64_EXTEND_I32_S),
        0xad => (i, Instruction::OP_I64_EXTEND_I32_U),
        0xae => (i, Instruction::OP_I64_TRUNC_F32_S),
        0xaf => (i, Instruction::OP_I64_TRUNC_F32_U),
        0xb0 => (i, Instruction::OP_I64_TRUNC_F64_S),
        0xb1 => (i, Instruction::OP_I64_TRUNC_F64_U),
        0xb2 => (i, Instruction::OP_F32_CONVERT_I32_S),
        0xb3 => (i, Instruction::OP_F32_CONVERT_I32_U),
        0xb4 => (i, Instruction::OP_F32_CONVERT_I64_S),
        0xb5 => (i, Instruction::OP_F32_CONVERT_I64_U),
        0xb6 => (i, Instruction::OP_F32_DEMOTE_F64),
        0xb7 => (i, Instruction::OP_F64_CONVERT_I32_S),
        0xb8 => (i, Instruction::OP_F64_CONVERT_I32_U),
        0xb9 => (i, Instruction::OP_F64_CONVERT_I64_S),
        0xba => (i, Instruction::OP_F64_CONVERT_I64_U),
        0xbb => (i, Instruction::OP_F64_PROMOTE_F32),
        0xbc => (i, Instruction::OP_I32_REINTERPRET_F32),
        0xbd => (i, Instruction::OP_I64_REINTERPRET_F64),
        0xbe => (i, Instruction::OP_F32_REINTERPRET_I32),
        0xbf => (i, Instruction::OP_F64_REINTERPRET_I64),
        0xc0 => (i, Instruction::OP_I32_EXTEND8_S),
        0xc1 => (i, Instruction::OP_I32_EXTEND16_S),
        0xc2 => (i, Instruction::OP_I64_EXTEND8_S),
        0xc3 => (i, Instruction::OP_I64_EXTEND16_S),
        0xc4 => (i, Instruction::OP_I64_EXTEND32_S),

        0xfc => {
            let (i, m) = take(1u8)(i)?;
            (
                i,
                match m {
                    [0x00] => Instruction::OP_I32_TRUNC_SAT_F32_S,
                    [0x01] => Instruction::OP_I32_TRUNC_SAT_F32_U,
                    [0x02] => Instruction::OP_I32_TRUNC_SAT_F64_S,
                    [0x03] => Instruction::OP_I32_TRUNC_SAT_F64_U,

                    [0x04] => Instruction::OP_I64_TRUNC_SAT_F32_S,
                    [0x05] => Instruction::OP_I64_TRUNC_SAT_F32_U,
                    [0x06] => Instruction::OP_I64_TRUNC_SAT_F64_S,
                    [0x07] => Instruction::OP_I64_TRUNC_SAT_F64_U,
                    _ => panic!("Invalid saturating instruction"),
                },
            )
        }
        _ => panic!("unknown instruction {}", instr[0]),
    };

    Ok((i, expr))
}

fn take_block<'a, 'b>(i: &'b [u8], counter: &'a mut Counter) -> IResult<&'b [u8], Instruction> {
    let (mut i, block_ty) = take_blocktype(i)?;

    //let (i, instructions) = take_expr(i)?;
    let mut instructions = Vec::new();

    loop {
        let (_, k) = take(1u8)(i)?; //0x0B

        if k == END_INSTR {
            break;
        }

        let (w, ii) = parse_instr(i, counter)?;
        i = w;
        instructions.push(ii);
    }

    let (i, e) = take(1u8)(i)?; //0x0B
    assert_eq!(e, END_INSTR);

    let block = Instruction::OP_BLOCK(block_ty, CodeBlock::new(counter, instructions));

    Ok((i, block))
}

fn take_loop<'a, 'b>(i: &'b [u8], counter: &'a mut Counter) -> IResult<&'b [u8], Instruction> {
    let (mut i, block_ty) = take_blocktype(i)?;

    let mut instructions = Vec::new();

    loop {
        let (_, k) = take(1u8)(i)?; //0x0B

        if k == END_INSTR {
            break;
        }

        let (w, ii) = parse_instr(i, counter)?;
        i = w;
        instructions.push(ii);
    }

    let (i, e) = take(1u8)(i)?; //0x0B
    assert_eq!(e, END_INSTR);

    let block = Instruction::OP_LOOP(block_ty, CodeBlock::new(counter, instructions));

    Ok((i, block))
}

fn take_conditional<'a, 'b>(
    i: &'b [u8],
    counter: &'a mut Counter,
) -> IResult<&'b [u8], Instruction> {
    debug!("take_conditional");

    //unreachable!("not correctly implemented!");

    let (mut i, blockty) = take_blocktype(i)?;

    let mut instructions = Vec::new();
    let mut else_instructions = Vec::new();

    loop {
        let (_, k) = take(1u8)(i)?; //0x0B or 0x05

        if k == END_IF_BLOCK || k == END_INSTR {
            break;
        }

        let (w, ii) = parse_instr(i, counter)?;
        i = w;
        instructions.push(ii);
    }

    let (mut i, k) = take(1u8)(i)?; //0x0B or 0x05

    if k == END_IF_BLOCK {
        //THIS IS THE ELSE BLOCK
        loop {
            let (_, k) = take(1u8)(i)?; //0x0B

            if k == END_INSTR {
                break;
            }

            let (w, ii) = parse_instr(i, counter)?;
            i = w;
            else_instructions.push(ii);
        }

        let (i, e) = take(1u8)(i)?; //0x0B
        assert_eq!(END_INSTR, e);

        return Ok((
            i,
            Instruction::OP_IF_AND_ELSE(
                blockty,
                CodeBlock::new(counter, instructions),
                CodeBlock::new(counter, else_instructions),
            ),
        ));
    }

    Ok((
        i,
        Instruction::OP_IF(blockty, CodeBlock::new(counter, instructions)),
    ))
}

fn take_br(i: &[u8]) -> IResult<&[u8], Instruction> {
    let (i, labelidx) = crate::take_leb_u32(i)?;

    let block = Instruction::OP_BR(labelidx);

    Ok((i, block))
}

fn take_br_if(i: &[u8]) -> IResult<&[u8], Instruction> {
    let (i, labelidx) = crate::take_leb_u32(i)?;

    let block = Instruction::OP_BR_IF(labelidx);

    Ok((i, block))
}

fn take_br_table(i: &[u8]) -> IResult<&[u8], Instruction> {
    let (i, n) = crate::take_leb_u32(i)?;
    let (i, ids) = count(crate::take_leb_u32, n as usize)(i)?;

    let (i, l_n) = crate::take_leb_u32(i)?;

    let block = Instruction::OP_BR_TABLE(ids, l_n);

    Ok((i, block))
}

fn take_call(i: &[u8]) -> IResult<&[u8], Instruction> {
    let (i, func_idx) = crate::take_leb_u32(i)?;

    let block = Instruction::OP_CALL(func_idx);

    Ok((i, block))
}

fn take_call_indirect(i: &[u8]) -> IResult<&[u8], Instruction> {
    let (i, type_idx) = crate::take_leb_u32(i)?;
    let (i, b) = take(1u8)(i)?;

    assert_eq!(b, &[0x00]);

    let block = Instruction::OP_CALL_INDIRECT(type_idx);

    Ok((i, block))
}

fn take_memarg(i: &[u8]) -> IResult<&[u8], MemArg> {
    let (i, n) = crate::take_leb_u32(i)?;
    let (i, o) = crate::take_leb_u32(i)?;

    Ok((
        i,
        MemArg {
            align: n,
            offset: o,
        },
    ))
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_instruction_block() {
        //env_logger::init();

        let mut payload = Vec::new();
        //payload.push(0x02); // block
        payload.push(0x40); // empty
        payload.push(0x01); //nop
        payload.push(0x01); //nop
        payload.push(0x0B); //end

        let mut counter = Counter::default();

        let instructions = take_block(&payload, &mut counter).unwrap();
        assert!(instructions.0 != [11]);

        let mut counter = Counter::default();

        assert_eq!(
            instructions.1,
            Instruction::OP_BLOCK(
                BlockType::Empty,
                CodeBlock::new(&mut counter, vec![Instruction::OP_NOP, Instruction::OP_NOP])
            )
        );
    }

    #[test]
    fn test_instruction_block_val_type() {
        //env_logger::init();

        let mut payload = Vec::new();
        //payload.push(0x02); // block
        payload.push(0x7C); // empty
        payload.push(0x01); //nop
        payload.push(0x01); //nop
        payload.push(0x0B); //end

        let mut counter = Counter::default();

        let instructions = take_block(&payload, &mut counter).unwrap();
        assert!(instructions.0 != [11]);
        assert_eq!(0, instructions.0.len());

        let mut counter = Counter::default();

        assert_eq!(
            instructions.1,
            Instruction::OP_BLOCK(
                BlockType::ValueType(ValueType::F64),
                CodeBlock::new(&mut counter, vec![Instruction::OP_NOP, Instruction::OP_NOP])
            )
        );
    }

    #[test]
    fn test_instruction_block_s33() {
        //env_logger::init();

        let mut payload = Vec::new();
        //payload.push(0x02); // block
        payload.push(0x80); // s33
        payload.push(0x7f); // s33
        payload.push(0x01); //nop
        payload.push(0x01); //nop
        payload.push(0x0B); //end

        let mut counter = Counter::default();

        let instructions = take_block(&payload, &mut counter).unwrap();
        assert!(instructions.0 != [11]);
        assert_eq!(0, instructions.0.len());

        let mut counter = Counter::default();

        assert_eq!(
            instructions.1,
            Instruction::OP_BLOCK(
                BlockType::FuncTy(-128),
                CodeBlock::new(&mut counter, vec![Instruction::OP_NOP, Instruction::OP_NOP])
            )
        );
    }

    #[test]
    fn test_instruction_block_nested_2() {
        let mut payload = Vec::new();
        //payload.push(0x02); // block
        payload.push(0x40); // empty
        payload.push(0x01); //nop
        payload.push(0x01); //nop
        payload.push(0x02); // block
        payload.push(0x40); // empty
        payload.push(0x01); //nop
        payload.push(0x0B); //end
        payload.push(0x01); //nop
        payload.push(0x0B); //end

        let mut counter = Counter::default();

        let instructions = take_block(&payload, &mut counter).unwrap();
        assert!(instructions.0 != [11]);

        let mut counter = Counter::default();

        let innerblock = CodeBlock::new(&mut counter, vec![Instruction::OP_NOP]);

        assert_eq!(
            instructions.1,
            Instruction::OP_BLOCK(
                BlockType::Empty,
                CodeBlock::new(
                    &mut counter,
                    vec![
                        Instruction::OP_NOP,
                        Instruction::OP_NOP,
                        Instruction::OP_BLOCK(BlockType::Empty, innerblock),
                        Instruction::OP_NOP,
                    ]
                )
            )
        );
    }

    #[test]
    fn test_instruction_block_nested_3() {
        let mut payload = Vec::new();
        //payload.push(0x02); // block
        payload.push(0x40); // empty
        payload.push(0x02); // block
        payload.push(0x40); // empty
        payload.push(0x02); // block
        payload.push(0x40); // empty
        payload.push(0x0B); //end
        payload.push(0x0B); //end
        payload.push(0x0B); //end

        let mut counter = Counter::default();

        let instructions = take_block(&payload, &mut counter).unwrap();
        assert!(instructions.0 != [11]);

        let mut counter = Counter::default();

        let block1 = CodeBlock::new(&mut counter, vec![]);

        let block3 = CodeBlock::new(
            &mut counter,
            vec![Instruction::OP_BLOCK(BlockType::Empty, block1)],
        );

        let block2 = CodeBlock::new(
            &mut counter,
            vec![Instruction::OP_BLOCK(BlockType::Empty, block3)],
        );

        assert_eq!(
            instructions.1,
            Instruction::OP_BLOCK(BlockType::Empty, block2)
        );
    }

    #[test]
    fn test_instruction_if() {
        //env_logger::init();

        let mut payload = Vec::new();
        //payload.push(0x02); // block
        payload.push(0x40); // empty
        payload.push(0x01); //nop
        payload.push(0x01); //nop
        payload.push(0x0B); //end

        let mut counter = Counter::default();

        let instructions = take_conditional(&payload, &mut counter).unwrap();

        //debug!("{:?}", instructions);
        assert!(instructions.0 != [11]);

        let mut counter = Counter::default();

        assert_eq!(
            instructions.1,
            Instruction::OP_IF(
                BlockType::Empty,
                CodeBlock::new(&mut counter, vec![Instruction::OP_NOP, Instruction::OP_NOP])
            )
        );
    }

    #[test]
    fn test_instruction_if_conditionals() {
        //env_logger::init();

        let mut payload = Vec::new();
        //payload.push(0x02); // block
        payload.push(0x40); // empty
        payload.push(0x01); //nop
        payload.push(0x05); //else
        payload.push(0x01); //nop
        payload.push(0x0B); //end

        let mut counter = Counter::default();

        let instructions = take_conditional(&payload, &mut counter).unwrap();

        //debug!("{:?}", instructions);
        assert!(instructions.0 != [11]);

        let mut counter = Counter::default();

        assert_eq!(
            instructions.1,
            Instruction::OP_IF_AND_ELSE(
                BlockType::Empty,
                CodeBlock::new(&mut counter, vec![Instruction::OP_NOP]),
                CodeBlock::new(&mut counter, vec![Instruction::OP_NOP])
            )
        );
    }

    #[test]
    fn test_instruction_loop() {
        let mut payload = Vec::new();
        //payload.push(0x02); // block
        payload.push(0x40); // empty
        payload.push(0x01); //nop
        payload.push(0x01); //nop
        payload.push(0x0B); //end

        let mut counter = Counter::default();

        let instructions = take_loop(&payload, &mut counter).unwrap();

        assert!(instructions.0 != [11]);

        let mut counter = Counter::default();

        assert_eq!(
            instructions.1,
            Instruction::OP_LOOP(
                BlockType::Empty,
                CodeBlock::new(&mut counter, vec![Instruction::OP_NOP, Instruction::OP_NOP]),
            )
        );
    }

    #[test]
    fn test_instruction_loop_nested() {
        let mut payload = Vec::new();
        //payload.push(0x02); // block
        payload.push(0x40); // empty
        payload.push(0x01); //nop
        payload.push(0x01); //nop
        payload.push(0x03); // empty
        payload.push(0x40); // empty
        payload.push(0x01); //nop
        payload.push(0x01); //nop
        payload.push(0x0B); //end
        payload.push(0x0B); //end

        let mut counter = Counter::default();

        let instructions = take_loop(&payload, &mut counter).unwrap();

        assert!(instructions.0 != [11]);

        let mut counter = Counter::default();

        let innerblock =
            CodeBlock::new(&mut counter, vec![Instruction::OP_NOP, Instruction::OP_NOP]);

        assert_eq!(
            instructions.1,
            Instruction::OP_LOOP(
                BlockType::Empty,
                CodeBlock::new(
                    &mut counter,
                    vec![
                        Instruction::OP_NOP,
                        Instruction::OP_NOP,
                        Instruction::OP_LOOP(BlockType::Empty, innerblock)
                    ]
                ),
            )
        );
    }
}
