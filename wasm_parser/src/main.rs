extern crate wasm_parser;

use docopt::Docopt;
use serde::Deserialize;
use wasm_parser::{parse, read_wasm};

const USAGE: &str = "
WebAssembly parser for binary files.

Usage:
  ./wasm_parser <input> [--no-output, --json]
  ./wasm_parser (-h | --help)
  ./wasm_parser --version

Options:
  -h --help     Show this screen.
  --version     Show version.
  --json        Output in json
  --no-output   Don't print
";

#[derive(Debug, Deserialize)]
struct Args {
    flag_no_output: bool,
    flag_json: bool,
    arg_input: String,
}

fn main() {
    env_logger::init();

    let args: Args = Docopt::new(USAGE)
        .and_then(|d| d.deserialize())
        .unwrap_or_else(|e| e.exit());

    let reader = read_wasm!(args.arg_input);

    let module = parse(reader).unwrap();

    if !args.flag_no_output {
        if args.flag_json {
            println!("{}", serde_json::to_string_pretty(&module).unwrap()); 
        }
        else {
            println!("{:#?}", module);
        }
    }

    /*
    //let reader = read_wasm!("./test_files/if_loop.wasm");
    let reader = read_wasm!("./test_files/arithmetic_i32.wasm");
    //let reader = read_wasm!("./test_files/empty.wasm");

    let _module = parse(reader);
    println!("{:#?}", module);
    */
}
