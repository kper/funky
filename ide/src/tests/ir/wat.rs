use crate::ir::wasm_ast::IR;
use funky::engine::module::ModuleInstance;
use funky::engine::*;
use insta::assert_snapshot;
use std::fs::File;
use std::io::{self, Write};
use validation::validate;
use wasm_parser::{parse, read_wasm};

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
    wat!(
        "test_simple_block",
        "(module (func (result i32) (block (result i32) i32.const 1)))"
    );
}

#[test]
fn test_nested_block() {
    wat!(
        "test_nested_block",
        "(module (func (result i32) (block (result i32) (block (result i32) i32.const 1))))"
    );
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
    wat!(
        "test_if_else",
        "(module (func (if (i32.const 1) (then nop) (else nop))))"
    );
}

#[test]
fn test_simple_br_if() {
    wat!("test_br_if", "(module (func (i32.const 1) (br_if 0)))");
}

#[test]
fn test_simple_br_table() {
    wat!(
        "test_br_table",
        "(module (func (param i32) (result i32)
    (block
      (block
        (br_table 1 0 (local.get 0))
        (return (i32.const 21))
      )
      (return (i32.const 20))
    )
    (i32.const 22)
  ))"
    );
}

#[test]
fn test_nested_block_value() {
    wat!(
        "test_nested_block_value",
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
        )))))"
    );
}

#[test]
fn test_local_tee() {
    wat!(
        "local_tee",
        "(module
    (func (param i32) (result i32) (i32.const 1) (local.tee 0)) 
  )"
    );
}

#[test]
fn test_call() {
    wat!(
        "test_call",
        "(module
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
    )"
    );
}

#[test]
fn test_phi() {
    wat!(
        "test_phi",
        "(module
	        (func (export \"singular\") (param i32) (result i32)
	          (i32.const 3)	
            (if (result i32) (local.get 0) (then (i32.const 1)) (else (i32.const 2)))
	          (i32.add)
          ))"
    );
}
#[test]
fn test_phi2() {
    wat!(
        "test_phi2",
        "(module
	        (func (export \"singular\") (param i32) (result i32)
	          (i32.const 4)	
            (if (result i32 i32) (local.get 0) (then (i32.const 1) (i32.const 1)) (else (i32.const 2) (i32.const 2)))
	          (i32.add)
	          (i32.add)
          ))"
    );
}

#[test]
fn test_fac() {
    wat!(
        "test_fac",
        "(module
          (func (export \"fac-rec\") (param i64) (result i64)
            (if (result i64) (i64.eq (local.get 0) (i64.const 0))
              (then (i64.const 1))
              (else
                (i64.mul (local.get 0) (call 0 (i64.sub (local.get 0) (i64.const 1))))
              )
            )
          ))"
    );
}

#[test]
fn test_fac_reversed() {
    wat!(
        "test_fac_reversed",
        "(module
          (func (export \"fac-rec\") (param i64) (result i64)
            (if (result i64) (i64.eq (local.get 0) (i64.const 0))
              (then (i64.mul (local.get 0) (call 0 (i64.sub (local.get 0) (i64.const 1)))))
              (else
                (i64.const 1)
              )
            )
          ))"
    );
}

#[test]
fn test_call_indirect() {
    wat!(
        "test_call_indirect",
        "(module
          (type $out-i32 (func (result i32)))
          (type $out-i64 (func (result i64)))
          (type $out-f32 (func (result f32)))
          (type $out-f64 (func (result f64)))
          (func $const-i32 (type $out-i32) (i32.const 0x132))
          (func $const-i64 (type $out-i64) (i64.const 0x164))
          (func $const-f32 (type $out-f32) (f32.const 0xf32))
          (func $const-f64 (type $out-f64) (f64.const 0xf64))
          (table funcref
          (elem
            $const-i32 $const-i64 $const-f32 $const-f64  ;; 0..3
          ))
          (func (export \"type-i32\") (result i32)
            (call_indirect (type $out-i32) (i32.const 0))
          )
          (func (export \"type-i64\") (result i64)
            (call_indirect (type $out-i64) (i32.const 1))
          )
          (func (export \"type-f32\") (result f32)
            (call_indirect (type $out-f32) (i32.const 2))
          )
          (func (export \"type-f64\") (result f64)
            (call_indirect (type $out-f64) (i32.const 3))
          ))"
    );
}

#[test]
fn test_call_indirect_with_param() {
    wat!(
        "test_call_indirect_with_param",
        "(module
          
        (type $over-i32 (func (param i32) (result i32)))
        (type $over-i64 (func (param i64) (result i64)))
        (type $over-f32 (func (param f32) (result f32)))
        (type $over-f64 (func (param f64) (result f64)))

        (func $id-i32 (type $over-i32) (local.get 0))
        (func $id-i64 (type $over-i64) (local.get 0))
        (func $id-f32 (type $over-f32) (local.get 0))
        (func $id-f64 (type $over-f64) (local.get 0))

        (table funcref
          (elem
            $id-i32 $id-i64 $id-f32 $id-f64
          )
        )

        (func (export \"type-first-i32\") (result i32)
          (call_indirect (type $over-i32) (i32.const 32) (i32.const 0))
        )
        (func (export \"type-first-i64\") (result i64)
          (call_indirect (type $over-i64) (i64.const 64) (i32.const 1))
        )
        (func (export \"type-first-f32\") (result f32)
          (call_indirect (type $over-f32) (f32.const 1.32) (i32.const 2))
        )
        (func (export \"type-first-f64\") (result f64)
          (call_indirect (type $over-f64) (f64.const 1.64) (i32.const 3))
        ))"
    );
}


#[test]
fn test_global_get() {
    wat!(
        "test_global_get",
        "(module

            (global $a i32 (i32.const -2))
            (global (;1;) f32 (f32.const -3))
            (global (;2;) f64 (f64.const -4))
            (global $b i64 (i64.const -5))

            (global $x (mut i32) (i32.const -12))
            (global (;5;) (mut f32) (f32.const -13))
            (global (;6;) (mut f64) (f64.const -14))
            (global $y (mut i64) (i64.const -15))

            (func (export \"get-a\") (result i32) (global.get $a))
            (func (export \"get-b\") (result i64) (global.get $b))
            (func (export \"get-x\") (result i32) (global.get $x))
            (func (export \"get-y\") (result i64) (global.get $y))
        )"
    );
}

#[test]
fn test_global_set() {
    wat!(
        "test_global_set",
        "(module

            (global $a i32 (i32.const -2))
            (global (;1;) f32 (f32.const -3))
            (global (;2;) f64 (f64.const -4))
            (global $b i64 (i64.const -5))

            (global $x (mut i32) (i32.const -12))
            (global (;5;) (mut f32) (f32.const -13))
            (global (;6;) (mut f64) (f64.const -14))
            (global $y (mut i64) (i64.const -15))

            (func (export \"set-x\") (param i32) (global.set $x (local.get 0)))
            (func (export \"set-y\") (param i64) (global.set $y (local.get 0)))
        )"
    );
}