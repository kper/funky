use funky::engine::module::ModuleInstance;
use funky::engine::*;
use insta::assert_snapshot;
use std::fs::File;
use std::io::{self, Write};
use wasm_parser::{parse, read_wasm};
use validation::validate;
use crate::ssa::wasm_ast::IR;

macro_rules! wat {
    ($name:expr, $input:expr) => {{
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

        let instance = ModuleInstance::new(&module);
        let engine = Engine::new(
            instance,
            &module,
            Box::new(funky::debugger::RelativeProgramCounter::default()),
            &imports,
        )
        .unwrap();

        let mut ir = IR::new();

        ir.visit(&engine).unwrap();

        assert_snapshot!($name, format!("{}", ir.buffer()));

        ir
    }};
}

#[test]
fn test_empty_function() {
    wat!("test_empty_function", "(module (func))");
}

#[test]
fn test_simple_block() {
    wat!("test_simple_block", "(module (func (result i32) (block (result i32) i32.const 1)))");
}

#[test]
fn test_nested_block() {
    wat!("test_nested_block", "(module (func (result i32) (block (result i32) (block (result i32) i32.const 1))))");
}

#[test]
fn test_simple_loop() {
    wat!("test_simple_loop", "(module (func (result i32) (block (result i32) (loop (result i32) (i32.const 1) (br 1)))))");
}

#[test]
fn test_simple_if() {
    wat!("test_if", "(module (func (if (i32.const 1) (then nop))))");
}

#[test]
fn test_simple_if_and_else() {
    wat!("test_if_else", "(module (func (if (i32.const 1) (then nop) (else nop))))");
}

#[test]
fn test_simple_br_if() {
    wat!("test_br_if", "(module (func (i32.const 1) (br_if 0)))");
}

#[test]
fn test_simple_br_table() {
    wat!("test_br_table", "(module (func (param i32) (result i32)
    (block
      (block
        (br_table 1 0 (local.get 0))
        (return (i32.const 21))
      )
      (return (i32.const 20))
    )
    (i32.const 22)
  ))");
}

#[test]
fn test_nested_block_value() {
    wat!("test_nested_block_value", "(module
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
        )))))");
}

#[test]
fn test_local_tee() {
  wat!("local_tee", "(module
    (func (param i32) (result i32) (i32.const 1) (local.tee 0)) 
  )");
}


#[test]
fn test_call() {
    wat!("test_call", "(module
    (func $fib (export \"fib\") (param i64) (result i64)
    (if (result i64) (i64.le_u (local.get 0) (i64.const 1))
      (then (i64.const 1))
      (else
        (i64.add
          (call $fib (i64.sub (local.get 0) (i64.const 2)))
          (call $fib (i64.sub (local.get 0) (i64.const 1)))
        )
      )
    )
  )

  (func $even (export \"even\") (param i64) (result i32)
    (if (result i32) (i64.eqz (local.get 0))
      (then (i32.const 44))
      (else (call $odd (i64.sub (local.get 0) (i64.const 1))))
    )
  )
  (func $odd (export \"odd\") (param i64) (result i32)
    (if (result i32) (i64.eqz (local.get 0))
      (then (i32.const 99))
      (else (call $even (i64.sub (local.get 0) (i64.const 1))))
    )
  )
    )");
}