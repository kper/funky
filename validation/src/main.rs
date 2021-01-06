extern crate validation;

use docopt::Docopt;
use serde::Deserialize;
use wasm_parser::Module;

use validation::validate;

const USAGE: &str = "
WebAssembly validator for an AST.

Usage:
  ./validation <input> [--no-output]
  ./validation (-h | --help)
  ./validation --version

Options:
  -h --help     Show this screen.
  --version     Show version.
  --no-output   Don't print
";

#[derive(Debug, Deserialize)]
struct Args {
    flag_no_output: bool,
    arg_input: String,
}

fn main() {
    env_logger::init();

    let args: Args = Docopt::new(USAGE)
        .and_then(|d| d.deserialize())
        .unwrap_or_else(|e| e.exit());

    let module : Module = serde_json::from_str(&read_file(args.arg_input)).expect("Converting file failed. Is the AST in json?");

    let result = validate(&module).unwrap();

    if !args.flag_no_output {
        println!("{:#?}", result);
    }
}

fn read_file(fs_name: String) -> String {
    use std::fs::File;
    use std::io::prelude::*;

    let mut fs = File::open(fs_name).unwrap();
    let mut reader = String::new();

    fs.read_to_string(&mut reader).unwrap();

    reader
}
