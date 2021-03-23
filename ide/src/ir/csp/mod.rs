use anyhow::{Context, Result};
use funky::engine::Engine;
use wasm_parser::core::Instruction::*;
use wasm_parser::core::*;

pub enum ConstraintSolvingResult {
    Trap,
    Unsolveable,
    Solveable(usize, usize),
}

enum Annotation {
    Unknown,
    Value(usize),
}

/// Calculate for every statement the probable stack size
pub fn calculate(engine: &Engine, function_index: u32) -> Result<()> {
    let addr = engine.get_function_addr_by_index(function_index)?;
    let instance = engine.get_function_instance(&addr)?;

    Ok(())
}

fn run(engine: &mut Engine, instructions: &[Instruction]) -> Result<()> {
    let mut annotated = Vec::with_capacity(instructions.len());
    let mut size = 0;
    for instruction in instructions.iter() {
        match instruction {
            OP_DROP | OP_BR_IF(_) => {
                size = size - 1;
                annotated.push((Annotation::Value(size), instruction));
            }
            OP_SELECT => {
                size = size - 3 + 1;
                annotated.push((Annotation::Value(size), instruction));
            }
            OP_BR(_) => {
                annotated.push((Annotation::Value(size), instruction));
            }
            OP_LOCAL_GET(_) => {
                size = size + 1;
                annotated.push((Annotation::Value(size), instruction));
            }
            OP_LOCAL_TEE(_) | OP_GLOBAL_GET(_) => {
                size = size + 1;
                annotated.push((Annotation::Value(size), instruction));
            }
            OP_GLOBAL_SET(_) => {
                size = size - 1;
                annotated.push((Annotation::Value(size), instruction));
            }
            OP_LOCAL_SET(_) => {
                size = size - 1;
                annotated.push((Annotation::Value(size), instruction));
            }
            OP_CALL(index) => {
                let addr = engine.get_function_addr_by_index(*index)?;
                let instance = engine.get_function_instance(&addr)?;

                size = size - instance.ty.param_types.len() + instance.ty.return_types.len();
                annotated.push((Annotation::Value(size), instruction));
            }
            OP_BLOCK(ty, _code) => {
                let param = (engine).get_param_count_block(&ty)? as usize;
                let ret = (engine).get_return_count_block(&ty)? as usize;
                size = size - param + ret;
                annotated.push((Annotation::Value(size), instruction));
            }
            OP_LOOP(ty, _code) => {
                let param = (engine).get_param_count_block(&ty)? as usize;
                let ret = (engine).get_return_count_block(&ty)? as usize;
                size = size - param + ret;
                annotated.push((Annotation::Value(size), instruction));
            }
            OP_IF(ty, _code) => {
                let param = (engine).get_param_count_block(&ty)? as usize;
                let ret = (engine).get_return_count_block(&ty)? as usize;
                size = size - 1 - param + ret;
                annotated.push((Annotation::Value(size), instruction));
            }
            OP_IF_AND_ELSE(ty, _code, _code2) => {
                let param = (engine).get_param_count_block(&ty)? as usize;
                let ret = (engine).get_return_count_block(&ty)? as usize;
                size = size - 1 - param + ret;
                annotated.push((Annotation::Value(size), instruction));
            }
            OP_I32_CLZ
            | OP_I32_CTZ
            | OP_I32_POPCNT
            | OP_I64_CLZ
            | OP_I64_CTZ
            | OP_I64_POPCNT
            | OP_F32_ABS
            | OP_F32_NEG
            | OP_F32_CEIL
            | OP_F32_FLOOR
            | OP_F32_TRUNC
            | OP_F32_NEAREST
            | OP_F32_SQRT
            | OP_F64_ABS
            | OP_F64_NEG
            | OP_F64_CEIL
            | OP_F64_FLOOR
            | OP_F64_TRUNC
            | OP_F64_NEAREST
            | OP_F64_SQRT
            | OP_I32_WRAP_I64
            | OP_I32_TRUNC_F32_S
            | OP_I32_TRUNC_F32_U
            | OP_I32_TRUNC_F64_S
            | OP_I32_TRUNC_F64_U
            | OP_I64_EXTEND_I32_U
            | OP_I64_EXTEND_I32_S
            | OP_I64_TRUNC_F32_S
            | OP_I64_TRUNC_F32_U
            | OP_I64_TRUNC_F64_S
            | OP_I64_TRUNC_F64_U
            | OP_F32_CONVERT_I32_S
            | OP_F32_CONVERT_I32_U
            | OP_F32_CONVERT_I64_S
            | OP_F32_CONVERT_I64_U
            | OP_F32_DEMOTE_F64
            | OP_F64_CONVERT_I32_S
            | OP_F64_CONVERT_I32_U
            | OP_F64_CONVERT_I64_S
            | OP_F64_CONVERT_I64_U
            | OP_F64_PROMOTE_F32
            | OP_I32_REINTERPRET_F32
            | OP_I64_REINTERPRET_F64
            | OP_F32_REINTERPRET_I32
            | OP_F64_REINTERPRET_I64
            | OP_I32_EXTEND8_S
            | OP_I32_EXTEND16_S
            | OP_I64_EXTEND8_S
            | OP_I64_EXTEND16_S
            | OP_I64_EXTEND32_S
            | OP_I32_TRUNC_SAT_F32_S
            | OP_I32_TRUNC_SAT_F32_U
            | OP_I32_TRUNC_SAT_F64_S
            | OP_I32_TRUNC_SAT_F64_U
            | OP_I64_TRUNC_SAT_F32_S
            | OP_I64_TRUNC_SAT_F32_U
            | OP_I64_TRUNC_SAT_F64_S
            | OP_I64_TRUNC_SAT_F64_U => {
                annotated.push((Annotation::Value(size), instruction));
            }
            OP_I32_ADD | OP_I32_SUB | OP_I32_MUL | OP_I32_DIV_S | OP_I32_DIV_U | OP_I32_REM_S
            | OP_I32_REM_U | OP_I32_AND | OP_I32_OR | OP_I32_XOR | OP_I32_SHL | OP_I32_SHR_S
            | OP_I32_SHR_U | OP_I32_ROTL | OP_I32_ROTR | OP_I64_ADD | OP_I64_SUB | OP_I64_MUL
            | OP_I64_DIV_S | OP_I64_DIV_U | OP_I64_REM_S | OP_I64_REM_U | OP_I64_AND
            | OP_I64_OR | OP_I64_XOR | OP_I64_SHL | OP_I64_SHR_S | OP_I64_SHR_U | OP_I64_ROTL
            | OP_I64_ROTR | OP_I32_EQZ | OP_I32_EQ | OP_I32_NE | OP_I32_LT_S | OP_I32_LT_U
            | OP_I32_GT_S | OP_I32_GT_U | OP_I32_LE_S | OP_I32_LE_U | OP_I32_GE_S | OP_I32_GE_U
            | OP_I64_EQZ | OP_I64_EQ | OP_I64_NE | OP_I64_LT_S | OP_I64_LT_U | OP_I64_GT_S
            | OP_I64_GT_U | OP_I64_LE_S | OP_I64_LE_U | OP_I64_GE_S | OP_I64_GE_U | OP_F32_EQ
            | OP_F32_NE | OP_F32_LT | OP_F32_GT | OP_F32_LE | OP_F32_GE | OP_F64_EQ | OP_F64_NE
            | OP_F64_LT | OP_F64_GT | OP_F64_LE | OP_F64_GE | OP_F32_ADD | OP_F32_SUB
            | OP_F32_MUL | OP_F32_DIV | OP_F64_ADD | OP_F64_SUB | OP_F64_MUL | OP_F64_DIV
            | OP_F32_MIN | OP_F32_MAX | OP_F32_COPYSIGN | OP_F64_MIN | OP_F64_MAX
            | OP_F64_COPYSIGN => {
                size = size - 1;
                annotated.push((Annotation::Value(size), instruction));
            }
            _ => {}
        }
    }

    Ok(())
}
