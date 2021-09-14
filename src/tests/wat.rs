use crate::engine::module::ModuleInstance;
use crate::engine::*;
use crate::value::Value::*;
use insta::assert_snapshot;
use std::fs::File;
use std::io::{self, Write};
use validation::validate;
use wasm_parser::{parse, read_wasm};

macro_rules! wat {
    ($name:expr, $input:expr, $invoke:expr, $init:expr) => {{
        use std::process::Command;
        use tempfile::tempdir;

        let tmp_dir = tempdir().unwrap();
        let file_path = tmp_dir.path().join("file.wat");

        {
            let mut tmp_file = File::create(file_path.clone()).unwrap();
            writeln!(tmp_file, $input).unwrap();
            tmp_file.flush().unwrap();

            println!("tmp_file {}", $input);
        }

        let output = Command::new("wat2wasm")
            .arg(file_path)
            .arg("-o")
            .arg(tmp_dir.path().join("file.wasm").to_str().unwrap())
            .output()
            .expect("Command failed");

        io::stdout().write_all(&output.stdout).unwrap();
        io::stderr().write_all(&output.stderr).unwrap();

        assert!(output.status.success());

        let file_path = tmp_dir.path().join("file.wasm");

        // Read it
        let file = read_wasm!(file_path);
        let module = parse(file).expect("Parsing failed");
        assert!(validate(&module).is_ok());

        let imports = Vec::new();

        let (instance, functions) = ModuleInstance::new(&module);
        let mut engine = Engine::new(
            instance,
            &functions,
            &module,
            Box::new(crate::debugger::RelativeProgramCounter::default()),
            &imports,
        )
        .unwrap();

        if let Err(err) = engine.invoke_exported_function_by_name($invoke, $init) {
            error!("ERROR: {}", err);
            err.chain()
                .skip(1)
                .for_each(|cause| error!("because: {}", cause));

            panic!("Test failed");
        }

        assert_snapshot!($name, format!("{:#?}", engine));

        engine
    }};
}

#[test]
fn test_block() {
    wat!(
        "test_block",
        "(module (func (result i32) (block (result i32) i32.const 1)) (export \"main\" (func 0)))",
        "main",
        vec![]
    );
}

#[test]
fn test_block_add() {
    wat!("test_block_add", "(module (func (result i32) (block (result i32) i32.const 1 i32.const 1 i32.add)) (export \"main\" (func 0)))", "main", vec![]);
}

#[test]
fn test_call_indirect_mid() {
    wat!(
        "indirect_mid",
        "
    (module  
    (func $func (param i32 i32) (result i32) (local.get 0))
    (type $check (func (param i32 i32) (result i32)))
    (table funcref (elem $func))
    (func (export \"as-call_indirect-mid\") (param i32) (result i32)
    (block (result i32)
      (call_indirect (type $check)
        (i32.const 1) (select (i32.const 2) (i32.const 3) (local.get 0)) (i32.const 0)
      )
    ))
  )",
        "as-call_indirect-mid",
        vec![I32(1)]
    );
}

#[test]
fn test_nested_br_value() {
    wat!(
        "nested_br_value",
        "(module (func (export \"nested-br_table-value-index\") (result i32)
    (i32.add
      (i32.const 1)
      (block (result i32)
        (drop (i32.const 2))
        (br_table 0 (i32.const 4) (br 0 (i32.const 8)))
        (i32.const 16)
      )
    )
  ))",
        "nested-br_table-value-index",
        vec![]
    );
}

#[test]
fn test_br_table_index() {
    wat!(
        "br_table_index",
        "(module 
    (func (export \"nested-br_table-value-index\") (result i32)
    (i32.add
      (i32.const 1)
      (block (result i32)
        (drop (i32.const 2))
        (br_table 0 (i32.const 4) (br 0 (i32.const 8)))
        (i32.const 16)
      )))   
    )",
        "nested-br_table-value-index",
        vec![]
    );
}

#[test]
fn test_br_table_value() {
    wat!(
        "br_table_value",
        "(module 
     (func (export \"nested-br_table-value\") (result i32)
    (i32.add
      (i32.const 1)
      (block (result i32)
        (drop (i32.const 2))
        (drop
          (block (result i32)
            (drop (i32.const 4))
            (br_table 0 (br 1 (i32.const 8)) (i32.const 1))
          )
        )
        (i32.const 16)
      )
    )) 
    )",
        "nested-br_table-value",
        vec![]
    );
}

#[test]
fn test_nested_block_value() {
    wat!(
        "nested_block_value",
        "(module
     (func (export \"nested-block-value\") (param i32) (result i32)
    (block (result i32)
      (drop (i32.const -1))
      (i32.add
        (i32.const 1)
        (block (result i32)
          (i32.add
            (i32.const 2)
            (block (result i32)
              (drop (i32.const 4))
              (i32.add
                (i32.const 8)
                (br_table 0 1 2 (i32.const 16) (local.get 0))
              )
            )
          )
        )))))",
        "nested-block-value",
        vec![I32(1)]
    );
}

#[test]
fn test_as_if_then() {
    wat!(
        "as-if-then",
        "(module
    (func (export \"as-if-then\") (param i32 i32) (result i32)
    (block (result i32)
      (if (result i32)
        (local.get 0)
        (then (br_table 1 (i32.const 3) (i32.const 0)))
        (else (local.get 1))
      )
    )
  ))",
        "as-if-then",
        vec![I32(1), I32(6)]
    );
}

#[test]
fn test_br_table_num_num() {
    wat!(
        "break-br_table_num_num",
        "(module
     (func (export \"break-br_table-num-num\") (param i32) (result i32 i64)
        (br_table 0 0 (i32.const 50) (i64.const 51) (local.get 0))
        (i32.const 51) (i64.const 52)
    ))",
        "break-br_table-num-num",
        vec![I32("4294967196".parse::<u32>().unwrap() as i32)]
    );
}

#[test]
fn test_params_break_id() {
    wat!(
        "params_break_id",
        "(module (func (export \"params-id-break\") (result i32)
    (local $x i32)
    (local.set $x (i32.const 0))
    (i32.const 1)
    (i32.const 2)
    (loop (param i32 i32) (result i32 i32)
      (local.set $x (i32.add (local.get $x) (i32.const 1)))
      (br_if 0 (i32.lt_u (local.get $x) (i32.const 10)))
    )
    (i32.add)
  ))",
        "params-id-break",
        vec![]
    );
}

#[test]
fn test_fac_ssa() {
    wat!(
        "fac_ssa",
        "(module (func $pick0 (param i64) (result i64 i64)
    (local.get 0) (local.get 0)
  )
  (func $pick1 (param i64 i64) (result i64 i64 i64)
    (local.get 0) (local.get 1) (local.get 0)
  )
  (func (export \"fac-ssa\") (param i64) (result i64)
    (i64.const 1) (local.get 0)
    (loop $l (param i64 i64) (result i64)
      (call $pick1) (call $pick1) (i64.mul)
      (call $pick1) (i64.const 1) (i64.sub)
      (call $pick0) (i64.const 0) (i64.gt_u)
      (br_if $l)
      (drop) (return)
    )
  ))",
        "fac-ssa",
        vec![I64(3)]
    );
}

#[test]
fn test_as_return_values() {
    wat!(
        "as_return_values",
        "(module
    (func (export \"as-return-values\") (result i32 i64)
    (i32.const 2)
    (block (result i64) (return (br 0 (i32.const 1) (i64.const 7))))
  ))
    ",
        "as-return-values",
        vec![]
    );
}

#[test]
fn test_add64_sat_u() {
    wat!(
        "add64_sat_u",
        "(module
   (func $add64_u_with_carry (export \"add64_u_with_carry\")
    (param $i i64) (param $j i64) (param $c i32) (result i64 i32)
    (local $k i64)
    (local.set $k
      (i64.add
        (i64.add (local.get $i) (local.get $j))
        (i64.extend_i32_u (local.get $c))
      )
    )
    (return (local.get $k) (i64.lt_u (local.get $k) (local.get $i)))
  )

  (func $add64_u_saturated (export \"add64_u_saturated\")
    (param i64 i64) (result i64)
    (call $add64_u_with_carry (local.get 0) (local.get 1) (i32.const 0))
    (if (param i64) (result i64)
      (then (drop) (i64.const -1))
    )
  ) 
) 
    ",
        "add64_u_saturated",
        vec![
            I64("9223372036854775808".parse::<u64>().unwrap() as i64),
            I64("9223372036854775808".parse::<u64>().unwrap() as i64)
        ]
    );
}
