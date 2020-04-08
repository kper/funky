use log::debug;
use nom::bytes::complete::{take, take_until};
use nom::multi::{count};
use nom::IResult;

use crate::core::*;
use crate::take_blocktype;

const END_INSTR: &[u8] = &[0x0B];

/*
pub(crate) fn parse_op_code(i: u8) -> Expr {
    match i {
        _ => unimplemented!("todo")
    }
}
*/

pub(crate) fn parse_instr(i: &[u8]) -> IResult<&[u8], Expr> {
    let (i, instr) = take(1u8)(i)?;

    let (i, expr) = match instr[0] {
        0x00 => (i, Expr::Ctrl(CtrlInstructions::OP_UNREACHABLE)),
        0x01 => (i, Expr::Ctrl(CtrlInstructions::OP_NOP)),
        0x02 => take_block(i)?,
        0x03 => take_loop(i)?,
        0x04 => take_conditional(i)?,
        0x0C => take_br(i)?,
        0x0D => take_br_if(i)?,
        0x0E => take_br_table(i)?,
        0x0F => (i, Expr::Ctrl(CtrlInstructions::OP_RETURN)),
        0x10 => take_call(i)?,
        0x11 => take_call_indirect(i)?,

        _ => panic!("unknown instruction"),
    };

    Ok((i, expr))
}

fn take_block(i: &[u8]) -> IResult<&[u8], Expr> {
    let (i, block_ty) = take_blocktype(i)?;

    let (i, ii) = parse_instr(i)?;
    let (i, b) = take(1u8)(i)?; //0x0B

    assert_eq!(b, END_INSTR);

    let block = Expr::Ctrl(CtrlInstructions::OP_BLOCK(block_ty, Box::new(ii)));

    Ok((i, block))
}

fn take_loop(i: &[u8]) -> IResult<&[u8], Expr> {
    let (i, block_ty) = take_blocktype(i)?;

    let (i, ii) = parse_instr(i)?;
    let (i, b) = take(1u8)(i)?; //0x0B

    assert_eq!(b, END_INSTR);

    let block = Expr::Ctrl(CtrlInstructions::OP_LOOP(block_ty, Box::new(ii)));

    Ok((i, block))
}

fn take_conditional(i: &[u8]) -> IResult<&[u8], Expr> {
    panic!("NOT IMPLEMENTED BECAUSE OVERLOADING")
}

fn take_br(i: &[u8]) -> IResult<&[u8], Expr> {
    let (i, labelidx) = crate::take_leb_u32(i)?;

    let block = Expr::Ctrl(CtrlInstructions::OP_BR(labelidx));

    Ok((i, block))
}

fn take_br_if(i: &[u8]) -> IResult<&[u8], Expr> {
    let (i, labelidx) = crate::take_leb_u32(i)?;

    let block = Expr::Ctrl(CtrlInstructions::OP_BR_IF(labelidx));

    Ok((i, block))
}

fn take_br_table(i: &[u8]) -> IResult<&[u8], Expr> {
    let (i, n) = crate::take_leb_u32(i)?;
    let (i, ids) = count(crate::take_leb_u32, n as usize)(i)?;
    
    let (i, l_n) = crate::take_leb_u32(i)?;

    let block = Expr::Ctrl(CtrlInstructions::OP_BR_TABLE(ids, l_n));

    Ok((i, block))
}

fn take_call(i: &[u8]) -> IResult<&[u8], Expr> {
    let (i, func_idx) = crate::take_leb_u32(i)?;

    let block = Expr::Ctrl(CtrlInstructions::OP_CALL(func_idx));

    Ok((i, block))
}

fn take_call_indirect(i: &[u8]) -> IResult<&[u8], Expr> {
    let (i, type_idx) = crate::take_leb_u32(i)?;
    let (i, b) = take(1u8)(i)?;

    assert_eq!(b, &[0x00]);

    let block = Expr::Ctrl(CtrlInstructions::OP_CALL_INDIRECT(type_idx));

    Ok((i, block))
}
