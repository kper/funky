use crate::debugger::RelativeProgramCounter;
use crate::engine::export::ExportInstance;
use crate::engine::module::ModuleInstance;
use crate::engine::*;
use crate::value::Value::*;
use crate::value::*;
use crate::wrap_instructions;
use insta::assert_snapshot;
use validation::validate;
use wasm_parser::{parse, read_wasm, Module};
use std::fs::File;
use std::io::{self, Write};

macro_rules! wat {
    ($name:expr, $input:expr, $invoke:expr, $init:expr) => {
        {
            use tempdir::TempDir;
            use std::process::Command;

            let tmp_dir = TempDir::new("wat_tests").unwrap();
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
            let mut engine = Engine::new(
                instance,
                &module,
                Box::new(crate::debugger::RelativeProgramCounter::default()),
                &imports,
            ).unwrap();

            if let Err(err) = engine.invoke_exported_function_by_name($invoke, $init) {
                error!("ERROR: {}", err);
                err.chain()
                    .skip(1)
                    .for_each(|cause| error!("because: {}", cause));

                panic!("Test failed");
            }

            assert_snapshot!($name, format!("{:#?}", engine));

            engine
        }
    };
}

#[test]
fn test_block() {
    wat!("test_block", "(module (func (result i32) (block (result i32) i32.const 1)) (export \"main\" (func 0)))", "main", vec![]);
}

#[test]
fn test_block_add() {
    wat!("test_block_add", "(module (func (result i32) (block (result i32) i32.const 1 i32.const 1 i32.add)) (export \"main\" (func 0)))", "main", vec![]);
}