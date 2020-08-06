/// This module is for debugging spec tests, which
/// failed in the `test_runner`.
use crate::construct_engine;
use crate::engine::*;

#[test]
fn test_f32_add_minus_0_and_nan() {
    // -0.0 + NaN = NaN

    let mut engine = construct_engine!(
        vec![Var(OP_LOCAL_GET(0)), Var(OP_LOCAL_GET(1)), Num(OP_F32_ADD)],
        vec![ValueType::F32, ValueType::F32],
        vec![ValueType::F32]
    );

    engine.invoke_exported_function(0, vec![Value::F32(-0.0), Value::F32(f32::NAN)]);

    if let Some(StackContent::Value(Value::F32(f1))) = engine.store.stack.pop() {
        assert!(f1.is_nan());
    }
    else {
        panic!("Not NAN");
    }
}

#[test]
fn test_f32_min_minus_0_and_nan() {
    // min(-0.0,NaN) = NaN

    let mut engine = construct_engine!(
        vec![Var(OP_LOCAL_GET(0)), Var(OP_LOCAL_GET(1)), Num(OP_F32_MIN)],
        vec![ValueType::F32, ValueType::F32],
        vec![ValueType::F32]
    );

    engine.invoke_exported_function(0, vec![Value::F32(-0.0), Value::F32(f32::NAN)]);

    if let Some(StackContent::Value(Value::F32(f1))) = engine.store.stack.pop() {
        assert!(f1.is_nan());
    }
    else {
        panic!("Not NAN");
    }
}

#[test]
fn test_f32_nearest_minus_0point5() {
    // nearest(-0.5) = -0.0

    let mut engine = construct_engine!(
        vec![Var(OP_LOCAL_GET(0)), Num(OP_F32_NEAREST)],
        vec![ValueType::F32],
        vec![ValueType::F32]
    );

    engine.invoke_exported_function(0, vec![Value::F32(-0.5)]);

    assert_eq!(Some(StackContent::Value(Value::F32(-0.0))), engine.store.stack.pop());
}

#[test]
fn test_f32_nearest_minus_1() {
    // nearest(-1.0) = -1.0

    let mut engine = construct_engine!(
        vec![Var(OP_LOCAL_GET(0)), Num(OP_F32_NEAREST)],
        vec![ValueType::F32],
        vec![ValueType::F32]
    );

    engine.invoke_exported_function(0, vec![Value::F32(-1.0)]);

    assert_eq!(Some(StackContent::Value(Value::F32(-1.0))), engine.store.stack.pop());
}
