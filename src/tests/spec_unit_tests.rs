/// This module is for debugging spec tests, which
/// failed in the `test_runner`.
use crate::construct_engine;
use crate::engine::*;
use wasm_parser::core::ValueType;


#[test]
fn test_f32_add_minus_0_and_nan() {
    // -0.0 + NaN = NaN

    let mut engine = construct_engine!(
        vec![OP_LOCAL_GET(0), OP_LOCAL_GET(1), OP_F32_ADD],
        vec![ValueType::F32, ValueType::F32],
        vec![ValueType::F32]
    );

    engine
        .invoke_exported_function(0, vec![Value::F32(-0.0), Value::F32(f32::NAN)])
        .unwrap();

    if let Some(StackContent::Value(Value::F32(f1))) = engine.store.stack.pop() {
        assert!(f1.is_nan());
    } else {
        panic!("Not NAN");
    }
}

#[test]
fn test_f32_min_minus_0_and_nan() {
    // min(-0.0,NaN) = NaN

    let mut engine = construct_engine!(
        vec![OP_LOCAL_GET(0), OP_LOCAL_GET(1), OP_F32_MIN],
        vec![ValueType::F32, ValueType::F32],
        vec![ValueType::F32]
    );

    engine
        .invoke_exported_function(0, vec![Value::F32(-0.0), Value::F32(f32::NAN)])
        .unwrap();

    if let Some(StackContent::Value(Value::F32(f1))) = engine.store.stack.pop() {
        assert!(f1.is_nan());
    } else {
        panic!("Not NAN");
    }
}

#[test]
fn test_f32_nearest_minus_0point5() {
    // nearest(-0.5) = -0.0

    let mut engine = construct_engine!(
        vec![OP_LOCAL_GET(0), OP_F32_NEAREST],
        vec![ValueType::F32],
        vec![ValueType::F32]
    );

    engine
        .invoke_exported_function(0, vec![Value::F32(-0.5)])
        .unwrap();

    assert_eq!(
        Some(StackContent::Value(Value::F32(-0.0))),
        engine.store.stack.pop()
    );
}

#[test]
fn test_f32_nearest_minus_1() {
    // nearest(-1.0) = -1.0

    let mut engine = construct_engine!(
        vec![OP_LOCAL_GET(0), OP_F32_NEAREST],
        vec![ValueType::F32],
        vec![ValueType::F32]
    );

    engine
        .invoke_exported_function(0, vec![Value::F32(-1.0)])
        .unwrap();

    assert_eq!(
        Some(StackContent::Value(Value::F32(-1.0))),
        engine.store.stack.pop()
    );
}

#[test]
fn test_as_mixed_operands() {
    use wasm_parser::core::Instruction::*;
    use wasm_parser::core::*;

    //env_logger::init();

    let mut e = crate::empty_engine();

    let mixed = FunctionBody {
        locals: vec![],
        code: vec![
            OP_I32_CONST(3),
            OP_I32_CONST(4),
            OP_CALL(1),
            OP_I32_CONST(5),
            OP_I32_ADD,
            OP_I32_MUL,
        ],
    };

    let swap = FunctionBody {
        locals: vec![],
        code: vec![OP_LOCAL_GET(1), OP_LOCAL_GET(0)],
    };

    e.store.funcs = vec![
        FuncInstance {
            ty: FunctionSignature {
                param_types: vec![],
                return_types: vec![ValueType::I32],
            },
            code: mixed.clone(),
        },
        FuncInstance {
            ty: FunctionSignature {
                param_types: vec![ValueType::I32, ValueType::I32],
                return_types: vec![ValueType::I32, ValueType::I32],
            },
            code: swap.clone(),
        },
    ];

    // Set the code section
    e.module.code = vec![mixed.clone(), swap.clone()];

    // Export the function
    e.module.funcaddrs.push(0);
    e.module.funcaddrs.push(1);
    e.module.exports = vec![ExportInstance {
        name: "test".to_string(),
        value: ExternalKindType::Function { ty: 0 },
    }];

    e.invoke_exported_function(0, vec![]).unwrap();

    assert_eq!(
        Some(StackContent::Value(Value::I32(32))),
        e.store.stack.pop()
    );
}

/*#[test]
fn test_local_set_write() {
    env_logger::init();

    use wasm_parser::core::ValueType::*;

    let mut engine = construct_engine!(
        vec![
            OP_F32_CONST(-0.3),
            OP_LOCAL_SET(1),
            OP_I32_CONST(40),
            OP_LOCAL_SET(3),
            OP_I32_CONST(-7),
            OP_LOCAL_SET(4),
            OP_F32_CONST(5.5),
            OP_LOCAL_SET(5),
            OP_I64_CONST(6),
            OP_LOCAL_SET(6),
            OP_F64_CONST(8.0),
            OP_LOCAL_SET(8),
            OP_LOCAL_GET(0),
            OP_F64_CONVERT_I64_U,
            OP_LOCAL_GET(1),
            OP_F64_PROMOTE_F32,
            OP_LOCAL_GET(2),
            OP_LOCAL_GET(3),
            OP_F64_CONVERT_I32_U,
            OP_LOCAL_GET(4),
            OP_F64_CONVERT_I32_S,
            OP_LOCAL_GET(5),
            OP_F64_PROMOTE_F32,
            OP_LOCAL_GET(6),
            OP_F64_CONVERT_I64_U,
            OP_LOCAL_GET(7),
            OP_F64_CONVERT_I64_U,
            OP_LOCAL_GET(8),
            OP_F64_ADD,
            OP_F64_ADD,
            OP_F64_ADD,
            OP_F64_ADD,
            OP_F64_ADD,
            OP_F64_ADD,
            OP_F64_ADD,
            OP_F64_ADD,
            OP_I64_TRUNC_F64_S
        ],
        vec![I64, F32, F64, I32, I32],
        vec![I64]
    );

    engine
        .invoke_exported_function(
            0,
            vec![
                Value::I64(1),
                Value::F32(2.0),
                Value::F64(3.3),
                Value::I32(4),
                Value::I32(5),
            ],
        )
        .unwrap();

    assert_eq!(
        Some(StackContent::Value(Value::I64(56))),
        engine.store.stack.pop()
    );
}*/

#[test]
fn test_f32_copy_sign() {
    let mut engine = construct_engine!(
        vec![
            OP_LOCAL_GET(0),
            OP_LOCAL_GET(1),
            OP_F32_COPYSIGN
        ],
        vec![ValueType::F32, ValueType::F32],
        vec![ValueType::F32]
    );

    engine
        .invoke_exported_function(0, vec![Value::F32(-0.0), Value::F32(-0.0)])
        .unwrap();

    assert_eq!(
        Some(StackContent::Value(Value::F32(-0.0))),
        engine.store.stack.pop()
    );
}

#[test]
fn test_memory_grow() {
    //env_logger::init();
    let mut engine = construct_engine!(
        vec![
            OP_I32_CONST(3),
            OP_MEMORY_GROW,
            OP_I32_CONST(2048),
            OP_MEMORY_GROW,
            OP_I32_CONST(65536),
            OP_MEMORY_GROW,
        ],
        vec![],
        vec![ValueType::I32]
    );

    engine.init_memory(0).expect("init memory failed");

    engine.invoke_exported_function(0, vec![]).unwrap();

    assert_eq!(
        Some(StackContent::Value(Value::I32(-1))),
        engine.store.stack.pop()
    );
}

#[test]
fn test_br_return() {
    env_logger::init();
    let mut engine = construct_engine!(
        vec![OP_BLOCK(
            BlockType::ValueTypeTy(0),
            CodeBlock::with(
                0,
                vec![
                    OP_F64_CONST(4.0),
                    OP_F64_CONST(5.0),
                    OP_BR(0),
                    OP_F64_ADD,
                    OP_F64_CONST(6.0)
                ],
            ),
        ),],
        vec![],
        vec![ValueType::F64, ValueType::F64]
    );

    let index = engine
        .module
        .add_func_type(vec![ValueType::F64, ValueType::F64])
        .unwrap();

    assert_eq!(
        index, 0,
        "Please adjust the index in BlockType::ValueTypeTy with this value"
    );

    engine.invoke_exported_function(0, vec![]).unwrap();

    assert_eq!(
        Some(StackContent::Value(Value::F64(4.0))),
        engine.store.stack.pop()
    );
    assert_eq!(
        Some(StackContent::Value(Value::F64(5.0))),
        engine.store.stack.pop()
    );
}
