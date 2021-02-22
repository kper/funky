use funky::engine::module::ModuleInstance;
use funky::engine::*;
use funky::value::Value::*;
use insta::assert_snapshot;
use std::fs::File;
use std::io::{self, Write};
use wasm_parser::{parse, read_wasm};
use validation::validate;
use crate::ssa::IR;

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

        let mut ir = IR::default();

        ir.visit(&engine.store);

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
fn test_simple_loop() {
    wat!("test_simple_loop", "(module (func (result i32) (block (result i32) (loop (result i32) (i32.const 1) (br 1)))))");
}

#[test]
fn test_simple_if() {
    env_logger::init();
    wat!("test_if", "(module (func (if (i32.const 1) (then nop))))");
}