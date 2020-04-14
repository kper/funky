extern crate validation;

use docopt::Docopt;
use serde::Deserialize;
use wasm_parser::{parse, read_wasm};

const USAGE: &'static str = "
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

    let module = read_file(args.arg_input);

    if !args.flag_no_output {
        println!("{:#?}", module);
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
