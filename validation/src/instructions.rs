use wasm_parser::core::NumericInstructions::*;
use wasm_parser::core::*;

type IResult<T> = Result<T, &'static str>;

//https://webassembly.github.io/spec/core/valid/instructions.html#numeric-instructions
//https://webassembly.github.io/spec/core/syntax/instructions.html#syntax-cvtop

pub fn get_ty_of_const(i: NumericInstructions) -> FuncType {
    let create_func_type = |k| FuncType {
        param_types: vec![],
        return_types: vec![k],
    };

    match i {
        OP_I32_CONST(_) => create_func_type(ValueType::I32),
        OP_I64_CONST(_) => create_func_type(ValueType::I64),
        OP_F32_CONST(_) => create_func_type(ValueType::F32),
        OP_F64_CONST(_) => create_func_type(ValueType::F64),
        _ => panic!("cannot check for other numeric ops"),
    }
}

pub fn get_ty_of_unop(i: NumericInstructions) -> FuncType {
    let create_func_type = |k: ValueType| FuncType {
        param_types: vec![k.clone()],
        return_types: vec![k],
    };

    match i {
        OP_I32_CLZ => create_func_type(ValueType::I32),
        OP_I64_CLZ => create_func_type(ValueType::I64),
        //OP_F32_CLZ => create_func_type(ValueType::F32),
        //OP_F64_CLZ => create_func_type(ValueType::F64),
        OP_I32_CTZ => create_func_type(ValueType::I32),
        OP_I64_CTZ => create_func_type(ValueType::I64),
        //OP_F32_CTZ => create_func_type(ValueType::F32),
        //OP_F64_CTZ => create_func_type(ValueType::F64),
        OP_I32_POPCNT => create_func_type(ValueType::I32),
        OP_I64_POPCNT => create_func_type(ValueType::I64),
        //OP_F32_POPCNT => create_func_type(ValueType::F32),
        //OP_F64_POPCNT => create_func_type(ValueType::F64),
        _ => panic!("cannot check for other numeric ops"),
    }
}

pub fn get_ty_of_binop(i: NumericInstructions) -> FuncType {
    let create_func_type = |k: ValueType| FuncType {
        param_types: vec![k.clone(), k.clone()],
        return_types: vec![k],
    };

    match i {
        OP_I32_ADD => create_func_type(ValueType::I32),
        OP_I64_ADD => create_func_type(ValueType::I64),
        OP_F32_ADD => create_func_type(ValueType::F32),
        OP_F64_ADD => create_func_type(ValueType::F64),
        OP_I32_SUB => create_func_type(ValueType::I32),
        OP_I64_SUB => create_func_type(ValueType::I64),
        OP_F32_SUB => create_func_type(ValueType::F32),
        OP_F64_SUB => create_func_type(ValueType::F64),
        OP_I32_MUL => create_func_type(ValueType::I32),
        OP_I64_MUL => create_func_type(ValueType::I64),
        OP_F32_MUL => create_func_type(ValueType::F32),
        OP_F64_MUL => create_func_type(ValueType::F64),
        OP_I32_DIV_S => create_func_type(ValueType::I32),
        OP_I64_DIV_S => create_func_type(ValueType::I64),
        OP_F32_DIV => create_func_type(ValueType::F32),
        OP_F64_DIV => create_func_type(ValueType::F64),
        OP_I32_DIV_U => create_func_type(ValueType::I32),
        OP_I64_DIV_U => create_func_type(ValueType::I64),
        OP_I32_REM_S => create_func_type(ValueType::I32),
        OP_I64_REM_S => create_func_type(ValueType::I64),
        //OP_F32_REM_S => create_func_type(ValueType::F32),
        //OP_F64_REM_S => create_func_type(ValueType::F64),
        OP_I32_REM_U => create_func_type(ValueType::I32),
        OP_I64_REM_U => create_func_type(ValueType::I64),
        //OP_F32_REM_U => create_func_type(ValueType::F32),
        //OP_F64_REM_U => create_func_type(ValueType::F64),
        OP_I32_OR => create_func_type(ValueType::I32),
        OP_I64_OR => create_func_type(ValueType::I64),
        //OP_F32_OR => create_func_type(ValueType::F32),
        //OP_F64_OR => create_func_type(ValueType::F64),
        OP_I32_XOR => create_func_type(ValueType::I32),
        OP_I64_XOR => create_func_type(ValueType::I64),
        //OP_F32_XOR => create_func_type(ValueType::F32),
        //OP_F64_XOR => create_func_type(ValueType::F64),
        OP_I32_SHL => create_func_type(ValueType::I32),
        OP_I64_SHL => create_func_type(ValueType::I64),
        //OP_F32_SHL => create_func_type(ValueType::F32),
        //OP_F64_SHL => create_func_type(ValueType::F64),
        OP_I32_SHR_S => create_func_type(ValueType::I32),
        OP_I64_SHR_S => create_func_type(ValueType::I64),
        //OP_F32_SHR_S => create_func_type(ValueType::F32),
        //OP_F64_SHR_S => create_func_type(ValueType::F64),
        OP_I32_SHR_U => create_func_type(ValueType::I32),
        OP_I64_SHR_U => create_func_type(ValueType::I64),
        //OP_F32_SHR_U => create_func_type(ValueType::F32),
        //OP_F64_SHR_U => create_func_type(ValueType::F64),
        OP_I32_ROTL => create_func_type(ValueType::I32),
        OP_I64_ROTL => create_func_type(ValueType::I64),
        //OP_F32_ROTL => create_func_type(ValueType::F32),
        //OP_F64_ROTL => create_func_type(ValueType::F64),
        OP_I32_ROTR => create_func_type(ValueType::I32),
        OP_I64_ROTR => create_func_type(ValueType::I64),
        //OP_F32_ROTR => create_func_type(ValueType::F32),
        //OP_F64_ROTR => create_func_type(ValueType::F64),
        _ => panic!("cannot check for other numeric ops"),
    }
}

pub fn get_ty_of_testop(i: NumericInstructions) -> FuncType {
    let create_func_type = |k| FuncType {
        param_types: vec![k],
        return_types: vec![ValueType::I32],
    };

    match i {
        OP_I32_EQZ => create_func_type(ValueType::I32),
        OP_I64_EQZ => create_func_type(ValueType::I64),
        _ => panic!("cannot check for other numeric ops"),
    }
}

pub fn get_ty_of_relop(i: NumericInstructions) -> FuncType {
    let create_func_type = |k: ValueType| FuncType {
        param_types: vec![k.clone(), k],
        return_types: vec![ValueType::I32],
    };

    //eq | ne | lt_sx | gt_sx | le_sx | ge_s
    match i {
        OP_I32_EQ => create_func_type(ValueType::I32),
        OP_I64_EQ => create_func_type(ValueType::I64),
        OP_F32_EQ => create_func_type(ValueType::I32),
        OP_F64_EQ => create_func_type(ValueType::I64),
        OP_I32_NE => create_func_type(ValueType::I32),
        OP_I64_NE => create_func_type(ValueType::I64),
        OP_F32_NE => create_func_type(ValueType::I32),
        OP_F64_NE => create_func_type(ValueType::I64),
        OP_I32_LT_S => create_func_type(ValueType::I32),
        OP_I64_LT_U => create_func_type(ValueType::I64),
        OP_I32_GT_S => create_func_type(ValueType::I32),
        OP_I64_GT_U => create_func_type(ValueType::I64),
        OP_I32_LE_S => create_func_type(ValueType::I32),
        OP_I64_LE_U => create_func_type(ValueType::I64),
        OP_I32_GE_S => create_func_type(ValueType::I32),
        OP_I64_GE_U => create_func_type(ValueType::I64),
        OP_F32_LT => create_func_type(ValueType::I32),
        OP_F64_LT => create_func_type(ValueType::I64),
        OP_F32_GT => create_func_type(ValueType::I32),
        OP_F64_GT => create_func_type(ValueType::I64),
        OP_F32_LE => create_func_type(ValueType::I32),
        OP_F64_LE => create_func_type(ValueType::I64),
        OP_F32_GE => create_func_type(ValueType::I32),
        OP_F64_GE => create_func_type(ValueType::I64),
        _ => panic!("cannot check for other numeric ops"),
    }
}

pub fn get_ty_cvtop(i: NumericInstructions) -> FuncType {
    use ValueType::*;

    let create_func_type = |k: ValueType, w: ValueType| FuncType {
        param_types: vec![k],
        return_types: vec![w],
    };

    match i {
        OP_I32_WRAP_I64 => create_func_type(I32, I64),
        OP_I32_TRUNC_F32_S => create_func_type(I32, F32),
        OP_I32_TRUNC_F32_U => create_func_type(I32, F32),
        OP_I32_TRUNC_F64_S => create_func_type(I32, F64),
        OP_I32_TRUNC_F64_U => create_func_type(I32, F64),
        //OP_I64_TRUNC_I32_S => create_func_type(I64, I32),
        //OP_I64_TRUNC_I32_U => create_func_type(I64, I32),
        OP_I64_EXTEND_I32_S => create_func_type(I64, I32),
        OP_I64_EXTEND_I32_U => create_func_type(I64, I32),
        OP_I64_TRUNC_F32_S => create_func_type(I64, F32),
        OP_I64_TRUNC_F32_U => create_func_type(I64, F32),
        OP_I64_TRUNC_F64_S => create_func_type(I64, F64),
        OP_I64_TRUNC_F64_U => create_func_type(I64, F64),
        OP_F32_CONVERT_I32_S => create_func_type(F32, I32),
        OP_F32_CONVERT_I32_U => create_func_type(F32, I32),
        OP_F32_CONVERT_I64_S => create_func_type(F32, I64),
        OP_F32_CONVERT_I64_U => create_func_type(F32, I64),
        OP_F32_DEMOTE_F64 => create_func_type(F32, F64),
        OP_F64_CONVERT_I32_S => create_func_type(F64, I32),
        OP_F64_CONVERT_I32_U => create_func_type(F64, I32),
        OP_F64_CONVERT_I64_S => create_func_type(F64, I64),
        OP_F64_CONVERT_I64_U => create_func_type(F64, F64),
        OP_F64_PROMOTE_F32 => create_func_type(F64, F32),
        OP_I32_REINTERPRET_F32 => create_func_type(I32, F32),
        OP_I64_REINTERPRET_F64 => create_func_type(I64, F64),
        OP_F32_REINTERPRET_I32 => create_func_type(F32, I32),
        OP_F64_REINTERPRET_I64 => create_func_type(F64, I64),
        _ => panic!("cannot check for other numeric ops"),
    }
}

pub fn get_ty_of_param(i: ParamInstructions, v: ValueType) -> FuncType {
    use ParamInstructions::*;

    let create_func_type = |k: ValueType| FuncType {
        param_types: vec![k],
        return_types: vec![],
    };

    let create_func_type2 = |k: ValueType| FuncType {
        param_types: vec![k.clone(), k.clone(), ValueType::I32],
        return_types: vec![k],
    };

    match i {
        OP_DROP => create_func_type(v),
        OP_SELECT => create_func_type2(v),
        _ => panic!("cannot check for other numeric ops"),
    }
}
