use crate::engine::*;
use insta::assert_snapshot;
use validation::validate;
use wasm_parser::core::*;
use wasm_parser::{parse, read_wasm, Module};

macro_rules! test_file_engine {
    ($fs_name:expr) => {
        let file = read_wasm!(&format!("tests/{}", $fs_name));
        let module = parse(file).expect("Parsing failed");
        assert!(validate(&module).is_ok());

        let instance = ModuleInstance::new(&module);
        let engine = Engine::new(instance, &module);

        assert_snapshot!($fs_name, format!("{:#?}", engine));
    };
}

macro_rules! allocation {
    ($sections:expr) => {{
        let module = Module {
            sections: $sections,
        };

        let instance = ModuleInstance::new(&module);
        let engine = Engine::new(instance, &module);

        engine
    }};
}

#[test]
fn test_allocation() {
    allocation!(vec![]); //no panic
}

#[test]
fn test_allocation_funcs() {
    let sig = FunctionSignature {
        param_types: vec![ValueType::I32],
        return_types: vec![ValueType::I32],
    };

    let body = FunctionBody {
        locals: vec![],
        code: vec![Instruction::Ctrl(CtrlInstructions::OP_NOP)],
    };

    let engine = allocation!(vec![
        Section::Type(TypeSection {
            entries: vec![sig.clone()]
        }),
        Section::Function(FunctionSection { types: vec![0] }),
        Section::Code(CodeSection {
            entries: vec![body.clone()]
        })
    ]);

    // Module instance has an entry for type
    // Module instance has an entry for code

    assert_eq!(1, engine.module.borrow().fn_types.len());
    assert_eq!(sig, engine.module.borrow().fn_types[0]);
    assert_eq!(1, engine.module.borrow().code.len());
    assert_eq!(body, engine.module.borrow().code[0]);

    // Store has an entry for func instance

    assert_eq!(1, engine.store.funcs.len());
    assert_eq!(sig, engine.store.funcs[0].ty);
    assert_eq!(body, engine.store.funcs[0].code);
}

#[test]
fn test_allocation_tables_zero() {
    let engine = allocation!(vec![Section::Table(TableSection {
        entries: vec![TableType {
            element_type: 0x70,
            limits: Limits::Zero(10)
        }]
    })]);

    // Module instance has an entry in tableaddrs with 0
    // Store has a table instance

    assert_eq!(1, engine.module.borrow().tableaddrs.len());
    assert_eq!(Some(&0), engine.module.borrow().tableaddrs.get(0));

    assert_eq!(1, engine.store.tables.len());
    assert_eq!(10, engine.store.tables[0].elem.len());
    assert!(engine.store.tables[0].elem.iter().all(|w| w == &None));
    assert_eq!(None, engine.store.tables[0].max);
}

#[test]
fn test_allocation_tables_one() {
    let engine = allocation!(vec![Section::Table(TableSection {
        entries: vec![TableType {
            element_type: 0x70,
            limits: Limits::One(10, 20)
        }]
    })]);

    // Module instance has an entry in tableaddrs with 0
    // Store has a table instance

    assert_eq!(1, engine.module.borrow().tableaddrs.len());
    assert_eq!(Some(&0), engine.module.borrow().tableaddrs.get(0));

    assert_eq!(1, engine.store.tables.len());
    assert_eq!(10, engine.store.tables[0].elem.len());
    assert!(engine.store.tables[0].elem.iter().all(|w| w == &None));
    assert_eq!(Some(20), engine.store.tables[0].max);
}

#[test]
fn test_allocation_memories_zero() {
    let engine = allocation!(vec![Section::Memory(MemorySection {
        entries: vec![MemoryType {
            limits: Limits::Zero(10)
        }]
    })]);

    // Module instance has an entry in memaddrs with 0
    // Store has a memory instance

    assert_eq!(1, engine.module.borrow().memaddrs.len());
    assert_eq!(Some(&0), engine.module.borrow().memaddrs.get(0));

    assert_eq!(1, engine.store.memory.len());
    assert_eq!(10 * 1024 * 64, engine.store.memory[0].data.len());
    assert!(engine.store.memory[0].data.iter().all(|w| w == &0u8));
    assert_eq!(None, engine.store.memory[0].max);
}

#[test]
fn test_allocation_memories_one() {
    let engine = allocation!(vec![Section::Memory(MemorySection {
        entries: vec![MemoryType {
            limits: Limits::One(10, 20)
        }]
    })]);

    // Module instance has an entry in memaddrs with 0
    // Store has a memory instance

    assert_eq!(1, engine.module.borrow().memaddrs.len());
    assert_eq!(Some(&0), engine.module.borrow().memaddrs.get(0));

    assert_eq!(1, engine.store.memory.len());
    assert_eq!(10 * 1024 * 64, engine.store.memory[0].data.len());
    assert!(engine.store.memory[0].data.iter().all(|w| w == &0u8));
    assert_eq!(Some(20), engine.store.memory[0].max);
}

#[test]
fn test_allocation_globals() {
    let engine = allocation!(vec![Section::Global(GlobalSection {
        globals: vec![GlobalVariable {
            ty: GlobalType {
                value_type: ValueType::I32,
                mu: Mu::Const
            },
            init: vec![Instruction::Num(NumericInstructions::OP_I32_CONST(10))]
        }]
    })]);

    // Module instance has an entry in globaladdrs with 0
    // Store has a global instance

    assert_eq!(1, engine.module.borrow().globaladdrs.len());
    assert_eq!(Some(&0), engine.module.borrow().globaladdrs.get(0));

    assert_eq!(1, engine.store.globals.len());
    assert_eq!(
        Variable {
            mutable: false,
            val: Value::I32(10)
        },
        engine.store.globals[0]
    );
}

#[test]
fn test_empty_wasm() {
    test_file_engine!("empty.wasm");
}

/*
#[test]
fn test_return_i32() {
    test_file_engine!("return_i32.wasm");
}
*/

/*
#[test]
fn test_return_i64() {
    test_file_module_instance!("return_i64.wasm");
}

#[test]
fn test_function_call() {
    test_file_module_instance!("function_call.wasm");
}

#[test]
fn test_arithmetic() {
    test_file_module_instance!("arithmetic.wasm");
}

#[test]
fn test_block_add_i32() {
    test_file_module_instance!("block_add_i32.wasm");
}

#[test]
fn test_loop_mult() {
    test_file_module_instance!("loop_mult.wasm");
}

#[test]
fn test_unreachable() {
    test_file_module_instance!("unreachable.wasm");
}

#[test]
fn test_if_loop() {
    test_file_module_instance!("if_loop.wasm");
}

#[test]
fn test_logic() {
    test_file_module_instance!("logic.wasm");
}

#[test]
fn test_gcd() {
    test_file_module_instance!("gcd.wasm");
}
*/
