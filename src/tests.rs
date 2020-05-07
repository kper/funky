use crate::engine::Value::*;
use crate::engine::*;
use insta::assert_snapshot;
use std::cell::RefCell;
use std::rc::Rc;
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

macro_rules! test_run_engine {
    ($fs_name:expr, $num_f:expr, $init:expr) => {{
        let file = read_wasm!(&format!("tests/{}", $fs_name));
        let module = parse(file).expect("Parsing failed");
        assert!(validate(&module).is_ok());

        let instance = ModuleInstance::new(&module);
        let mut engine = Engine::new(instance, &module);

        assert_snapshot!($fs_name, format!("{:#?}", engine));

        engine.invoke_exported_function($num_f, $init);
        engine
    }};
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
    // Module instance has an entry for funcaddrs

    let mi = engine.module.borrow();
    assert_eq!(1, mi.fn_types.len());
    assert_eq!(sig, mi.fn_types[0]);
    assert_eq!(1, mi.code.len());
    assert_eq!(body, mi.code[0]);
    assert_eq!(1, mi.funcaddrs.len());
    assert_eq!(Some(&0), mi.funcaddrs.get(0));

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
fn test_allocation_exports() {
    let engine = allocation!(vec![
        Section::Memory(MemorySection {
            entries: vec![MemoryType {
                limits: Limits::Zero(10)
            }]
        }),
        Section::Export(ExportSection {
            entries: vec![ExportEntry {
                name: "memory".to_string(),
                kind: ExternalKindType::Memory { ty: 0 }
            }]
        })
    ]);

    // Module instance has an entry for exporsts

    assert_eq!(1, engine.module.borrow().exports.len());
    assert_eq!(
        ExportInstance {
            name: "memory".to_string(),
            value: ExternalKindType::Memory { ty: 0 }
        },
        engine.module.borrow().exports[0]
    );
}

#[test]
fn test_empty_wasm() {
    test_file_engine!("empty.wasm");
}

#[test]
fn test_sum_loop() {
    test_file_engine!("sum_loop.wasm");
}

#[test]
fn test_return_i32() {
    test_file_engine!("return_i32.wasm");
}

#[test]
fn test_return_i64() {
    test_file_engine!("return_i64.wasm");
}

#[test]
fn test_function_call() {
    test_file_engine!("function_call.wasm");
}

#[test]
fn test_arithmetic() {
    test_file_engine!("arithmetic.wasm");
}

#[test]
fn test_block_add_i32() {
    test_file_engine!("block_add_i32.wasm");
}

#[test]
fn test_loop_mult() {
    test_file_engine!("loop_mult.wasm");
}

#[test]
fn test_unreachable() {
    test_file_engine!("unreachable.wasm");
}

//#[test]
fn test_if_loop() {
    test_file_engine!("if_loop.wasm");
}

#[test]
fn test_logic() {
    test_file_engine!("logic.wasm");
}

#[test]
fn test_gcd() {
    test_file_engine!("gcd.wasm");
}

#[test]
fn test_run_add() {
    let engine = test_run_engine!("add.wasm", 0, vec![I32(1), I32(2)]);
    assert_eq!(
        Some(&StackContent::Value(I32(3))),
        engine.store.stack.last()
    )
}

#[test]
fn test_run_call() {
    /*
    (module
    (func $getAnswer (result i32) i32.const 42)
    (func (export "getAnswerPlus1") (result i32)
        call $getAnswer
        i32.const 1
        i32.add))
     */

    let engine = test_run_engine!("call.wasm", 0, vec![]);
    assert_eq!(
        Some(&StackContent::Value(I32(43))),
        engine.store.stack.last()
    )
}

fn test_run_gcd() {
    //env_logger::init();
    let engine = test_run_engine!("gcd.wasm", 2, vec![I32(50), I32(10)]);
    assert_eq!(
        Some(&StackContent::Value(I32(10))),
        engine.store.stack.last()
    )
}

//#[test]
fn test_run_incr_counter() {
    env_logger::init();
    let engine = test_run_engine!("incr_counter.wasm", 0, vec![]);
    assert_eq!(
        None,
        engine.store.stack.last()
    )
}

#[test]
fn test_run_gt() {
    let engine = test_run_engine!("gt.wasm", 0, vec![I32(10), I32(11)]);
    assert_eq!(
        Some(&StackContent::Value(I32(0))),
        engine.store.stack.last()
    )
}

#[test]
fn test_run_gt_2() {
    let engine = test_run_engine!("gt.wasm", 0, vec![I32(11), I32(10)]);
    assert_eq!(
        Some(&StackContent::Value(I32(1))),
        engine.store.stack.last()
    )
}

#[test]
fn test_run_sub() {
    let engine = test_run_engine!("sub.wasm", 0, vec![I32(10), I32(11)]);
    assert_eq!(
        Some(&StackContent::Value(I32(-1))),
        engine.store.stack.last()
    )
}

#[test]
fn test_run_sub2() {
    let engine = test_run_engine!("sub.wasm", 0, vec![I32(11), I32(10)]);
    assert_eq!(
        Some(&StackContent::Value(I32(1))),
        engine.store.stack.last()
    )
}

    /*
     (module
  (func (export "type-local-i32") (result i32) (local i32) (local.tee 0 (i32.const 0)))
  (func (export "type-local-i64") (result i64) (local i64) (local.tee 0 (i64.const 0)))
  (func (export "type-local-f32") (result f32) (local f32) (local.tee 0 (f32.const 0)))
  (func (export "type-local-f64") (result f64) (local f64) (local.tee 0 (f64.const 0)))

  (func (export "type-param-i32") (param i32) (result i32) (local.tee 0 (i32.const 10)))
  (func (export "type-param-i64") (param i64) (result i64) (local.tee 0 (i64.const 11)))
  (func (export "type-param-f32") (param f32) (result f32) (local.tee 0 (f32.const 11.1)))
  (func (export "type-param-f64") (param f64) (result f64) (local.tee 0 (f64.const 12.2)))

  (func (export "type-mixed") (param i64 f32 f64 i32 i32) (local f32 i64 i64 f64)
    (drop (i64.eqz (local.tee 0 (i64.const 0))))
    (drop (f32.neg (local.tee 1 (f32.const 0))))
    (drop (f64.neg (local.tee 2 (f64.const 0))))
    (drop (i32.eqz (local.tee 3 (i32.const 0))))
    (drop (i32.eqz (local.tee 4 (i32.const 0))))
    (drop (f32.neg (local.tee 5 (f32.const 0))))
    (drop (i64.eqz (local.tee 6 (i64.const 0))))
    (drop (i64.eqz (local.tee 7 (i64.const 0))))
    (drop (f64.neg (local.tee 8 (f64.const 0))))
  ))*/

//#[test]
fn test_run_local_tee() {
    let engine = test_run_engine!("local.tee.wasm", 0, vec![]);
    assert_eq!(
        Some(&StackContent::Value(I32(0))),
        engine.store.stack.last()
    )
}

//#[test]
fn test_run_local_tee_1() {
    let engine = test_run_engine!("local.tee.wasm", 1, vec![]);
    assert_eq!(
        Some(&StackContent::Value(I64(0))),
        engine.store.stack.last()
    )
}

//#[test]
fn test_run_local_tee_2() {
    let engine = test_run_engine!("local.tee.wasm", 2, vec![]);
    assert_eq!(
        Some(&StackContent::Value(F32(0.0))),
        engine.store.stack.last()
    )
}

//#[test]
fn test_run_local_tee_3() {
    let engine = test_run_engine!("local.tee.wasm", 3, vec![]);
    assert_eq!(
        Some(&StackContent::Value(F64(0.0))),
        engine.store.stack.last()
    )
}

//#[test]
fn test_run_local_tee_4() {
    let engine = test_run_engine!("local.tee.wasm", 4, vec![I32(2)]);
    assert_eq!(
        Some(&StackContent::Value(I32(10))),
        engine.store.stack.last()
    )
}

//#[test]
fn test_run_local_tee_5() {
    let engine = test_run_engine!("local.tee.wasm", 5, vec![I64(3)]);
    assert_eq!(
        Some(&StackContent::Value(I64(11))),
        engine.store.stack.last()
    )
}

//#[test]
fn test_run_local_tee_6() {
    let engine = test_run_engine!("local.tee.wasm", 6, vec![F32(4.4)]);
    assert_eq!(
        Some(&StackContent::Value(F32(11.1))),
        engine.store.stack.last()
    )
}

//#[test]
fn test_run_local_tee_7() {
    let engine = test_run_engine!("local.tee.wasm", 7, vec![F64(5.5)]);
    assert_eq!(
        Some(&StackContent::Value(F64(12.2))),
        engine.store.stack.last()
    )
}

//#[test]
fn test_run_as_loop_first_br_if_1() {
    /*
      (func (export "as-loop-first") (param i32) (result i32)
        (block (loop (br_if 1 (local.get 0)) (return (i32.const 2)))) (i32.const 3)
      )
    */
    let engine = test_run_engine!("as_loop_br_if.wasm", 0, vec![I32(0)]);
    assert_eq!(
        Some(&StackContent::Value(I32(2))),
        engine.store.stack.last()
    )
}


//#[test]
fn test_run_as_loop_first_br_if_2() {
    /*
      (func (export "as-loop-first") (param i32) (result i32)
        (block (loop (br_if 1 (local.get 0)) (return (i32.const 2)))) (i32.const 3)
      )
    */
//    env_logger::init();
    let engine = test_run_engine!("as_loop_br_if.wasm", 0, vec![I32(1)]);
    assert_eq!(
        Some(&StackContent::Value(I32(3))),
        engine.store.stack.last()
    )
}

/*
(func (export "br_if0") (result i32)
    (local $i i32)
    (local.set $i (i32.const 0))
    (block $outer (result i32)
      (block $inner
        (br_if $inner (i32.const 0))
        (local.set $i (i32.or (local.get $i) (i32.const 0x1)))
        (br_if $inner (i32.const 1))
        (local.set $i (i32.or (local.get $i) (i32.const 0x2)))
      )
      (drop (br_if $outer
        (block (result i32)
          (local.set $i (i32.or (local.get $i) (i32.const 0x4)))
          (local.get $i)
        )
        (i32.const 0)
      ))
      (local.set $i (i32.or (local.get $i) (i32.const 0x8)))
      (drop (br_if $outer
        (block (result i32)
          (local.set $i (i32.or (local.get $i) (i32.const 0x10)))
          (local.get $i)
        )
        (i32.const 1)
      ))
      (local.set $i (i32.or (local.get $i) (i32.const 0x20))) (local.get $i)
    )
  )

  (func (export "br_if1") (result i32)
    (block $l0 (result i32)
      (drop
        (br_if $l0
          (block $l1 (result i32) (br $l1 (i32.const 1)))
          (i32.const 1)
        )
      )
      (i32.const 0)
    )
  )

  (func (export "br_if2") (result i32)
    (block $l0 (result i32)
      (if (i32.const 1)
        (then
          (drop
            (br_if $l0
              (block $l1 (result i32) (br $l1 (i32.const 1)))
              (i32.const 1)
            )
          )
        )
      )
      (i32.const 0)
    )
  )

  (func (export "br_if3") (result i32)
    (local $i1 i32)
    (drop
      (i32.add
        (block $l0 (result i32)
          (drop (br_if $l0
            (block (result i32) (local.set $i1 (i32.const 1)) (local.get $i1))
            (block (result i32) (local.set $i1 (i32.const 2)) (local.get $i1))
          ))
          (i32.const 0)
        )
        (i32.const 0)
      )
    )
    (local.get $i1)
  )

  (func (export "br") (result i32)
    (block $l0 (result i32)
      (if (i32.const 1)
        (then (br $l0 (block $l1 (result i32) (br $l1 (i32.const 1)))))
        (else (block (drop (block $l1 (result i32) (br $l1 (i32.const 1))))))
      )
      (i32.const 1)
    )
  ))*/

//#[test]
fn test_run_br_if0() {
    //env_logger::init();
    let engine = test_run_engine!("labels.wasm", 0, vec![]);
    assert_eq!(
        Some(&StackContent::Value(I32(0x1d))),
        engine.store.stack.last()
    )
}

//#[test]
fn test_run_br_if1() {
    //env_logger::init();
    let engine = test_run_engine!("labels.wasm", 1, vec![]);
    assert_eq!(
        Some(&StackContent::Value(I32(0x1))),
        engine.store.stack.last()
    )
}

//#[test]
fn test_run_br_if2() {
    //env_logger::init();
    let engine = test_run_engine!("labels.wasm", 2, vec![]);
    assert_eq!(
        Some(&StackContent::Value(I32(1))),
        engine.store.stack.last()
    )
}

fn test_run_br_if3() {
    //env_logger::init();
    let engine = test_run_engine!("labels.wasm", 3, vec![]);
    assert_eq!(
        Some(&StackContent::Value(I32(1))),
        engine.store.stack.last()
    )
}

fn test_run_br_if4() {
    //env_logger::init();
    let engine = test_run_engine!("labels.wasm", 4, vec![]);
    assert_eq!(
        Some(&StackContent::Value(I32(2))),
        engine.store.stack.last()
    )
}

/*
(func (export "as-loop-mid") (param i32) (result i32)
    (block (loop (call $dummy) (br_if 1 (local.get 0)) (return (i32.const 2))))
    (i32.const 4)
  )
*/

//#[test]
fn test_run_as_loop_mid_br1() {
    //env_logger::init();
    let engine = test_run_engine!("as_loop_mid_br_if.wasm", 0, vec![I32(0)]);
    assert_eq!(
        Some(&StackContent::Value(I32(2))),
        engine.store.stack.last()
    )
}

//#[test]
fn test_run_as_loop_mid_br2() {
    //env_logger::init();
    let engine = test_run_engine!("as_loop_mid_br_if.wasm", 0, vec![I32(1)]);
    assert_eq!(
        Some(&StackContent::Value(I32(4))),
        engine.store.stack.last()
    )
}

/*
  (func (export "as-loop-last") (param i32)
    (loop (call $dummy) (br_if 1 (local.get 0)))
  )
*/

//#[test]
/*
fn test_run_as_loop_last_br_if1() {
    env_logger::init();
    let engine = test_run_engine!("as_loop_last_br_if.wasm", 0, vec![I32(5)]);
    assert_eq!(
        None,
        engine.store.stack.last()
    )
}*/

//#[test]
fn test_run_as_loop_last_br_if2() {
    //env_logger::init();
    let engine = test_run_engine!("as_loop_last_br_if.wasm", 0, vec![I32(1)]);
    assert_eq!(
        None,
        engine.store.stack.last()
    )
}


/*
#[test]
fn test_run_call_indirect() {
    /*
     (module
      (table 2 anyfunc)
      (func $f1 (result i32)
        i32.const 42)
      (func $f2 (result i32)
        i32.const 13)
      (elem (i32.const 0) $f1 $f2)
      (type $return_i32 (func (result i32)))
      (func (export "callByIndex") (param $i i32) (result i32)
        local.get $i
        call_indirect (type $return_i32))
    )*/

    env_logger::init();
    let engine = test_run_engine!("wasm-table.wasm", 0, vec![I32(1)]);
    assert_eq!(
        Some(&StackContent::Value(I32(43))),
        engine.store.stack.last()
    )
}
*/
