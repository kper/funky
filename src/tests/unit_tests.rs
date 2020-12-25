use crate::empty_engine;
use crate::engine::*;
use crate::value::Value;
use crate::value::Value::*;
use crate::wrap_instructions;
use crate::PAGE_SIZE;
use wasm_parser::core::Instruction::*;
use wasm_parser::core::*;

#[test]
#[should_panic(expected = "Function expected different parameters!")]
fn test_invoke_wrong_length_parameters() {
    //env_logger::init();
    let mut e = empty_engine();

    let body = FunctionBody {
        locals: vec![],
        code: wrap_instructions!(vec![OP_LOCAL_GET(0), OP_LOCAL_GET(1), OP_I32_ADD]),
    };

    // We have 2 parameters, but supply 3
    e.store.funcs = vec![FuncInstance {
        ty: FunctionSignature {
            param_types: vec![ValueType::I32, ValueType::I32],
            return_types: vec![],
        },
        //module: Rc::downgrade(&e.module),
        code: body.clone(),
    }];

    e.module.code = vec![body.clone()];

    e.invoke_function(0, vec![Value::I32(1), Value::I32(2), Value::I32(3)])
        .expect("invoke function failed");
}

#[test]
#[should_panic(expected = "Function expected different parameters!")]
fn test_invoke_wrong_ty_parameters() {
    //env_logger::init();
    let mut e = empty_engine();

    let body = FunctionBody {
        locals: vec![],
        code: wrap_instructions!(vec![OP_LOCAL_GET(0), OP_LOCAL_GET(1), OP_I32_ADD]),
    };

    // We have 2 parameters, but supply 3
    e.store.funcs = vec![FuncInstance {
        ty: FunctionSignature {
            param_types: vec![ValueType::F32, ValueType::I32],
            return_types: vec![],
        },
        //module: Rc::downgrade(&e.module),
        code: body.clone(),
    }];

    e.module.code = vec![body.clone()];

    e.invoke_function(0, vec![Value::I32(1), Value::I32(2)])
        .expect("invoke function failed");
}

#[test]
fn test_run_function() {
    let mut e = empty_engine();
    e.store.stack = vec![StackContent::Frame(Frame {
        arity: 1,
        locals: Vec::new(),
        //module_instance: e.downgrade_mod_instance(),
    })];
    e.module.code = vec![FunctionBody {
        locals: vec![],
        code: wrap_instructions!(vec![OP_I32_CONST(42), OP_I32_CONST(42), OP_I32_ADD]),
    }];
    e.run_function(0).unwrap();
    assert_eq!(StackContent::Value(I32(84)), e.store.stack.pop().unwrap());
    e.store.stack = vec![StackContent::Frame(Frame {
        arity: 1,
        locals: Vec::new(),
        //module_instance: e.downgrade_mod_instance(),
    })];
    e.module.code = vec![FunctionBody {
        locals: vec![],
        code: wrap_instructions!(vec![
            OP_I64_CONST(32),
            OP_I64_CONST(32),
            OP_I64_ADD,
            OP_I64_CONST(2),
            OP_I64_MUL,
        ]),
    }];
    e.run_function(0).unwrap();
    assert_eq!(StackContent::Value(I64(128)), e.store.stack.pop().unwrap());
}

#[test]
fn test_function_with_params() {
    let mut e = empty_engine();

    e.store.stack = vec![StackContent::Frame(Frame {
        arity: 1,
        locals: vec![I32(1), I32(4)],
        //module_instance: e.downgrade_mod_instance(),
    })];
    e.module.code = vec![FunctionBody {
        locals: vec![],
        code: wrap_instructions!(vec![OP_LOCAL_GET(0), OP_LOCAL_GET(1), OP_I32_ADD]),
    }];
    e.run_function(0).unwrap();
    assert_eq!(StackContent::Value(I32(5)), e.store.stack.pop().unwrap());
}

#[test]
fn test_function_block() {
    let mut e = empty_engine();
    let mut counter = Counter::default();

    e.store.stack = vec![StackContent::Frame(Frame {
        arity: 1,
        locals: vec![I32(1), I32(1)],
        //module_instance: e.downgrade_mod_instance(),
    })];
    e.module.code = vec![FunctionBody {
        locals: vec![],
        code: wrap_instructions!(vec![OP_BLOCK(
            BlockType::ValueType(ValueType::I32),
            CodeBlock::new(
                &mut counter,
                vec![OP_LOCAL_GET(0), OP_LOCAL_GET(1), OP_I32_ADD]
            ),
        )]),
    }];
    e.run_function(0).unwrap();
    assert_eq!(StackContent::Value(I32(2)), e.store.stack.pop().unwrap());
}

#[test]
fn test_function_if() {
    let mut e = empty_engine();
    let mut counter = Counter::default();

    e.store.stack = vec![
        StackContent::Value(Value::I32(1)),
        StackContent::Frame(Frame {
            arity: 1,
            locals: vec![I32(1), I32(1)], //arguments for LOCAL_GET
                                          //module_instance: e.downgrade_mod_instance(),
        }),
    ];
    e.module.code = vec![FunctionBody {
        locals: vec![],
        code: wrap_instructions!(vec![OP_IF(
            BlockType::ValueType(ValueType::I32),
            CodeBlock::new(
                &mut counter,
                vec![OP_LOCAL_GET(0), OP_LOCAL_GET(1), OP_I32_ADD]
            ),
        )]),
    }];
    e.run_function(0).unwrap();
    assert_eq!(StackContent::Value(I32(2)), e.store.stack.pop().unwrap());
}

#[test]
fn test_function_if_false() {
    let mut e = empty_engine();
    let mut counter = Counter::default();

    e.store.stack = vec![
        StackContent::Value(Value::I32(0)), //THIS CHANGED
        StackContent::Frame(Frame {
            arity: 1,
            locals: vec![I32(1), I32(1)], //arguments for LOCAL_GET
                                          //module_instance: e.downgrade_mod_instance(),
        }),
    ];
    e.module.code = vec![FunctionBody {
        locals: vec![],
        code: wrap_instructions!(vec![OP_IF(
            BlockType::ValueType(ValueType::I32),
            CodeBlock::new(
                &mut counter,
                vec![OP_LOCAL_GET(0), OP_LOCAL_GET(1), OP_I32_ADD],
            ),
        )]),
    }];
    e.run_function(0).unwrap();
    assert_eq!(None, e.store.stack.pop());
}

#[test]
fn test_function_if_else_1() {
    let mut e = empty_engine();
    let mut counter = Counter::default();

    e.store.stack = vec![
        StackContent::Value(Value::I32(1)),
        StackContent::Frame(Frame {
            arity: 1,
            locals: vec![I32(1), I32(1)], //arguments for LOCAL_GET
                                          //module_instance: e.downgrade_mod_instance(),
        }),
    ];
    e.module.code = vec![FunctionBody {
        locals: vec![],
        code: wrap_instructions!(vec![OP_IF_AND_ELSE(
            BlockType::ValueType(ValueType::I32),
            CodeBlock::new(
                &mut counter,
                vec![OP_LOCAL_GET(0), OP_LOCAL_GET(1), OP_I32_ADD],
            ),
            CodeBlock::new(&mut counter, vec![OP_I32_CONST(-1000)]),
        )]),
    }];
    e.run_function(0).unwrap();
    assert_eq!(
        Some(StackContent::Value(Value::I32(2))),
        e.store.stack.pop()
    );
}

#[test]
fn test_function_if_else_2() {
    let mut e = empty_engine();
    let mut counter = Counter::default();

    e.store.stack = vec![
        StackContent::Value(Value::I32(0)), //changed
        StackContent::Frame(Frame {
            arity: 1,
            locals: vec![I32(1), I32(1)], //arguments for LOCAL_GET
                                          //module_instance: e.downgrade_mod_instance(),
        }),
    ];
    e.module.code = vec![FunctionBody {
        locals: vec![],
        code: wrap_instructions!(vec![OP_IF_AND_ELSE(
            BlockType::ValueType(ValueType::I32),
            CodeBlock::new(&mut counter, vec![OP_LOCAL_GET(0), OP_LOCAL_GET(1), OP_I32_ADD],),
            CodeBlock::new(&mut counter, vec![OP_I32_CONST(-1000)]),
        )]),
    }];
    e.run_function(0).unwrap();
    assert_eq!(
        Some(StackContent::Value(Value::I32(-1000))),
        e.store.stack.pop()
    );
}

#[test]
fn test_function_local_set() {
    let mut e = empty_engine();
    e.store.stack = vec![StackContent::Frame(Frame {
        arity: 1,
        locals: vec![I32(1), I32(4)],
        //module_instance: e.downgrade_mod_instance(),
    })];
    e.module.code = vec![FunctionBody {
        locals: vec![],
        code: wrap_instructions!(vec![
            OP_LOCAL_GET(0),
            OP_LOCAL_GET(1),
            OP_I32_ADD,
            OP_LOCAL_SET(0),
            OP_I32_CONST(32),
            OP_LOCAL_GET(0),
            OP_I32_ADD,
        ]),
    }];
    e.run_function(0).unwrap();
    assert_eq!(StackContent::Value(I32(37)), e.store.stack.pop().unwrap());
}

#[test]
fn test_function_globals() {
    let mut e = empty_engine();
    e.store.globals = vec![Variable {
        mutable: true,
        val: I32(69),
    }];
    e.module.code = vec![FunctionBody {
        locals: vec![],
        code: wrap_instructions!(vec![
            OP_GLOBAL_GET(0),
            OP_I32_CONST(351),
            OP_I32_ADD,
            OP_GLOBAL_SET(0),
        ]),
    }];
    e.run_function(0).unwrap();
    assert_eq!(I32(420), e.store.globals[0].val);
}

#[test]
fn test_drop_select() {
    let mut e = empty_engine();
    e.store.globals = vec![Variable {
        mutable: true,
        val: I32(20),
    }];
    e.module.code = vec![FunctionBody {
        locals: vec![],
        code: wrap_instructions!(vec![
            OP_I32_CONST(1),
            OP_I32_CONST(2),
            OP_I32_CONST(0),
            OP_I32_CONST(4),
            OP_DROP,
            OP_SELECT,
            OP_GLOBAL_SET(0),
        ]),
    }];
    e.run_function(0).unwrap();
    assert_eq!(I32(2), e.store.globals[0].val);
}

#[test]
fn test_memory_store_i32() {
    let mut e = empty_engine();
    e.module.memaddrs.push(0);
    e.store.memory = vec![MemoryInstance {
        data: [0; 4].to_vec(),
        max: None,
    }];

    e.module.code = vec![FunctionBody {
        locals: vec![],
        code: wrap_instructions!(vec![
            OP_I32_CONST(0),
            OP_I32_CONST(4),
            OP_I32_STORE(MemArg {
                offset: 0,
                align: 1,
            }),
        ]),
    }];
    e.run_function(0).unwrap();
    assert_eq!((4 as i32).to_le_bytes(), e.store.memory[0].data.as_slice());
}

#[test]
fn test_memory_load_i32() {
    //env_logger::init();
    let mut e = empty_engine();
    e.module.memaddrs.push(0);
    e.store.memory = vec![MemoryInstance {
        data: [0; 10].to_vec(),
        max: None,
    }];

    e.module.code = vec![FunctionBody {
        locals: vec![],
        code: wrap_instructions!(vec![
            OP_I32_CONST(0),
            OP_I32_LOAD(MemArg {
                offset: 0,
                align: 1,
            }),
        ]),
    }];
    e.run_function(0).unwrap();
    assert_eq!(Some(&StackContent::Value(I32(0))), e.store.stack.last());
}

#[test]
fn test_memory_store_i32_in_i8() {
    let mut e = empty_engine();
    e.module.memaddrs.push(0);
    e.store.memory = vec![MemoryInstance {
        data: [0; 1].to_vec(),
        max: None,
    }];

    e.module.code = vec![FunctionBody {
        locals: vec![],
        code: wrap_instructions!(vec![
            OP_I32_CONST(0),
            OP_I32_CONST(4),
            OP_I32_STORE_8(MemArg {
                offset: 0,
                align: 1,
            }),
        ]),
    }];
    e.run_function(0).unwrap();
    assert_eq!((4 as i8).to_le_bytes(), e.store.memory[0].data.as_slice());
}

#[test]
fn test_memory_load_i32_of_u8() {
    //env_logger::init();
    let mut e = empty_engine();
    e.module.memaddrs.push(0);
    e.store.memory = vec![MemoryInstance {
        data: [0; 4].to_vec(),
        max: None,
    }];

    e.module.code = vec![FunctionBody {
        locals: vec![],
        code: wrap_instructions!(vec![
            OP_I32_CONST(0),
            OP_I32_CONST(4),
            OP_I32_STORE_8(MemArg {
                offset: 0,
                align: 1,
            }),
            OP_I32_CONST(0),
            OP_I32_LOAD_8_u(MemArg {
                offset: 0,
                align: 1,
            }),
        ]),
    }];
    e.run_function(0).unwrap();
    assert_eq!((4 as i32).to_le_bytes(), e.store.memory[0].data.as_slice());
    assert_eq!(
        Some(StackContent::Value(I32(4 as i32))),
        e.store.stack.pop()
    );
}

#[test]
fn test_memory_store_i32_in_i16() {
    let mut e = empty_engine();
    e.module.memaddrs.push(0);
    e.store.memory = vec![MemoryInstance {
        data: [0; 2].to_vec(),
        max: None,
    }];

    e.module.code = vec![FunctionBody {
        locals: vec![],
        code: wrap_instructions!(vec![
            OP_I32_CONST(0),
            OP_I32_CONST(9),
            OP_I32_STORE_16(MemArg {
                offset: 0,
                align: 1,
            }),
        ]),
    }];
    e.run_function(0).unwrap();
    assert_eq!((9 as i16).to_le_bytes(), e.store.memory[0].data.as_slice());
}

#[test]
fn test_memory_store_i64() {
    let mut e = empty_engine();
    e.module.memaddrs.push(0);
    e.store.memory = vec![MemoryInstance {
        data: [0; 8].to_vec(),
        max: None,
    }];

    e.module.code = vec![FunctionBody {
        locals: vec![],
        code: wrap_instructions!(vec![
            OP_I32_CONST(0),
            OP_I64_CONST(4),
            OP_I64_STORE(MemArg {
                offset: 0,
                align: 1,
            }),
        ]),
    }];
    e.run_function(0).unwrap();
    assert_eq!((4 as i64).to_le_bytes(), e.store.memory[0].data.as_slice());
}

#[test]
fn test_memory_store_i64_in_i16() {
    let mut e = empty_engine();
    e.module.memaddrs.push(0);
    e.store.memory = vec![MemoryInstance {
        data: [0; 2].to_vec(),
        max: None,
    }];

    e.module.code = vec![FunctionBody {
        locals: vec![],
        code: wrap_instructions!(vec![
            OP_I32_CONST(0),
            OP_I64_CONST(9),
            OP_I64_STORE_16(MemArg {
                offset: 0,
                align: 1,
            }),
        ]),
    }];
    e.run_function(0).unwrap();
    assert_eq!((9 as i16).to_le_bytes(), e.store.memory[0].data.as_slice());
}

#[test]
fn test_memory_store_i64_in_i32() {
    let mut e = empty_engine();
    e.module.memaddrs.push(0);
    e.store.memory = vec![MemoryInstance {
        data: [0; 4].to_vec(),
        max: None,
    }];

    e.module.code = vec![FunctionBody {
        locals: vec![],
        code: wrap_instructions!(vec![
            OP_I32_CONST(0),
            OP_I64_CONST(i64::MAX),
            OP_I64_STORE_32(MemArg {
                offset: 0,
                align: 1,
            }),
        ]),
    }];
    e.run_function(0).unwrap();
    assert_eq!(
        ((i64::MAX % 2_i64.pow(32)) as i32).to_le_bytes(),
        e.store.memory[0].data.as_slice()
    );
}

#[test]
fn test_memory_store_f32() {
    let mut e = empty_engine();
    e.module.memaddrs.push(0);
    e.store.memory = vec![MemoryInstance {
        data: [0; 4].to_vec(),
        max: None,
    }];

    e.module.code = vec![FunctionBody {
        locals: vec![],
        code: wrap_instructions!(vec![
            OP_I32_CONST(0),
            OP_F32_CONST(4.1),
            OP_F32_STORE(MemArg {
                offset: 0,
                align: 1,
            }),
        ]),
    }];
    e.run_function(0).unwrap();
    assert_eq!(
        (4.1 as f32).to_le_bytes(),
        e.store.memory[0].data.as_slice()
    );
}

#[test]
fn test_memory_store_f64() {
    let mut e = empty_engine();
    e.module.memaddrs.push(0);
    e.store.memory = vec![MemoryInstance {
        data: [0; 8].to_vec(),
        max: None,
    }];

    e.module.code = vec![FunctionBody {
        locals: vec![],
        code: wrap_instructions!(vec![
            OP_I32_CONST(0),
            OP_F64_CONST(4.1),
            OP_F64_STORE(MemArg {
                offset: 0,
                align: 1,
            }),
        ]),
    }];
    e.run_function(0).unwrap();
    assert_eq!(
        (4.1 as f64).to_le_bytes(),
        e.store.memory[0].data.as_slice()
    );
}

#[test]
fn test_num_store_f64() {
    let mut e = empty_engine();
    e.module.memaddrs.push(0);
    e.store.memory = vec![MemoryInstance {
        data: [0; 8].to_vec(),
        max: None,
    }];

    e.module.code = vec![FunctionBody {
        locals: vec![],
        code: wrap_instructions!(vec![
            OP_I32_CONST(0),
            OP_F64_CONST(4.1),
            OP_F64_STORE(MemArg {
                offset: 0,
                align: 1,
            }),
        ]),
    }];
    e.run_function(0).unwrap();
    assert_eq!(
        (4.1 as f64).to_le_bytes(),
        e.store.memory[0].data.as_slice()
    );
}

#[test]
fn test_num_wrap_i64_max() {
    let mut e = empty_engine();
    e.store.stack = vec![StackContent::Frame(Frame {
        arity: 1,
        locals: vec![],
        //module_instance: e.downgrade_mod_instance(),
    })];
    e.module.code = vec![FunctionBody {
        locals: vec![],
        code: wrap_instructions!(vec![OP_I64_CONST(i32::MAX as i64), OP_I32_WRAP_I64]),
    }];
    e.run_function(0).unwrap();
    assert_eq!(
        StackContent::Value(I32(i32::MAX)),
        e.store.stack.pop().unwrap()
    );
}

#[test]
fn test_num_wrap_i64_min() {
    let mut e = empty_engine();
    e.store.stack = vec![StackContent::Frame(Frame {
        arity: 1,
        locals: vec![],
        //module_instance: e.downgrade_mod_instance(),
    })];
    e.module.code = vec![FunctionBody {
        locals: vec![],
        code: wrap_instructions!(vec![OP_I64_CONST(i32::MIN as i64), OP_I32_WRAP_I64]),
    }];
    e.run_function(0).unwrap();
    assert_eq!(
        StackContent::Value(I32(i32::MIN)),
        e.store.stack.pop().unwrap()
    );
}

#[test]
fn test_num_wrap_i64_overflow() {
    let mut e = empty_engine();
    e.store.stack = vec![StackContent::Frame(Frame {
        arity: 1,
        locals: vec![],
        //module_instance: e.downgrade_mod_instance(),
    })];
    e.module.code = vec![FunctionBody {
        locals: vec![],
        code: wrap_instructions!(vec![OP_I64_CONST((i32::MAX as i64) + 50), OP_I32_WRAP_I64,]),
    }];
    e.run_function(0).unwrap();
    // account for 0 value
    assert_eq!(
        StackContent::Value(I32(i32::MIN + 49)),
        e.store.stack.pop().unwrap()
    );
}

#[test]
fn test_num_extend_s() {
    let mut e = empty_engine();
    e.store.stack = vec![StackContent::Frame(Frame {
        arity: 1,
        locals: vec![],
        //module_instance: e.downgrade_mod_instance(),
    })];
    e.module.code = vec![FunctionBody {
        locals: vec![],
        code: wrap_instructions!(vec![OP_I32_CONST(-1), OP_I64_EXTEND_I32_S]),
    }];
    e.run_function(0).unwrap();
    assert_eq!(StackContent::Value(I64(-1)), e.store.stack.pop().unwrap());
}
#[test]
fn test_num_extend_u() {
    let mut e = empty_engine();
    e.store.stack = vec![StackContent::Frame(Frame {
        arity: 1,
        locals: vec![],
        //module_instance: e.downgrade_mod_instance(),
    })];
    e.module.code = vec![FunctionBody {
        locals: vec![],
        code: wrap_instructions!(vec![OP_I32_CONST(-1), OP_I64_EXTEND_I32_U]),
    }];
    e.run_function(0).unwrap();
    assert_eq!(
        StackContent::Value(I64(u32::MAX as i64)),
        e.store.stack.pop().unwrap()
    );
}

#[test]
fn test_num_trunc_s() {
    let mut e = empty_engine();
    e.store.stack = vec![StackContent::Frame(Frame {
        arity: 1,
        locals: vec![],
        //module_instance: e.downgrade_mod_instance(),
    })];
    e.module.code = vec![FunctionBody {
        locals: vec![],
        code: wrap_instructions!(vec![OP_F32_CONST(234.923), OP_I32_TRUNC_F32_S]),
    }];
    e.run_function(0).unwrap();
    assert_eq!(StackContent::Value(I32(234)), e.store.stack.pop().unwrap());
}

#[test]
fn test_num_promote() {
    let mut e = empty_engine();
    e.store.stack = vec![StackContent::Frame(Frame {
        arity: 1,
        locals: vec![],
        //module_instance: e.downgrade_mod_instance(),
    })];
    e.module.code = vec![FunctionBody {
        locals: vec![],
        code: wrap_instructions!(vec![OP_F32_CONST(1.1234568357467651), OP_F64_PROMOTE_F32,]),
    }];
    e.run_function(0).unwrap();
    assert_eq!(
        StackContent::Value(F64(1.1234568357467651)),
        e.store.stack.pop().unwrap()
    );
}

#[test]
fn test_num_demote() {
    let mut e = empty_engine();
    e.store.stack = vec![StackContent::Frame(Frame {
        arity: 1,
        locals: vec![],
        //module_instance: e.downgrade_mod_instance(),
    })];
    e.module.code = vec![FunctionBody {
        locals: vec![],
        code: wrap_instructions!(vec![OP_F64_CONST(1.1234568357467651420), OP_F32_DEMOTE_F64,]),
    }];
    e.run_function(0).unwrap();
    // float got demoted - we loose precision
    assert_eq!(
        StackContent::Value(F32(1.1234568357467651)),
        e.store.stack.pop().unwrap()
    );
}

#[test]
fn test_num_convert_s() {
    let mut e = empty_engine();
    e.store.stack = vec![StackContent::Frame(Frame {
        arity: 1,
        locals: vec![],
        //module_instance: e.downgrade_mod_instance(),
    })];
    e.module.code = vec![FunctionBody {
        locals: vec![],
        code: wrap_instructions!(vec![OP_I32_CONST(-1), OP_F32_CONVERT_I32_S]),
    }];
    e.run_function(0).unwrap();
    assert_eq!(StackContent::Value(F32(-1.0)), e.store.stack.pop().unwrap());
}

#[test]
fn test_num_convert_u() {
    let mut e = empty_engine();
    e.store.stack = vec![StackContent::Frame(Frame {
        arity: 1,
        locals: vec![],
        //module_instance: e.downgrade_mod_instance(),
    })];
    e.module.code = vec![FunctionBody {
        locals: vec![],
        code: wrap_instructions!(vec![OP_I32_CONST(-1), OP_F32_CONVERT_I32_U]),
    }];
    e.run_function(0).unwrap();
    assert_eq!(
        StackContent::Value(F32(u32::MAX as f32)),
        e.store.stack.pop().unwrap()
    );
}

#[test]
fn test_memory_grow() {
    let mut e = empty_engine();
    e.module.memaddrs.push(0);
    e.store.memory = vec![MemoryInstance {
        data: [0; 10].to_vec(),
        max: None,
    }];

    e.module.code = vec![FunctionBody {
        locals: vec![],
        code: wrap_instructions!(vec![OP_I32_CONST(1), OP_MEMORY_GROW]),
    }];

    e.run_function(0).unwrap();
    assert_eq!(e.store.memory[0].data.len(), PAGE_SIZE + 10);
}

#[test]
fn test_memory_grow_with_max() {
    let mut e = empty_engine();
    e.module.memaddrs.push(0);
    e.store.memory = vec![MemoryInstance {
        data: [0; 10].to_vec(),
        max: Some(11),
    }];

    e.module.code = vec![FunctionBody {
        locals: vec![],
        code: wrap_instructions!(vec![OP_I32_CONST(i32::MAX), OP_MEMORY_GROW]),
    }];

    e.run_function(0).unwrap();
    assert_eq!(Some(&StackContent::Value(I32(-1))), e.store.stack.last());
}
