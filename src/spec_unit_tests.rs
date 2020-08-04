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

    /*
    assert_eq!(
        Some(StackContent::Value(Value::F32(f32::NAN))),
        engine.store.stack.pop()
    );
    */
}
