#[macro_use]
extern crate log;
extern crate env_logger;
extern crate funky;

use docopt::Docopt;
use funky::engine::{Engine, ModuleInstance, Store};
use funky::engine::Value::*;
use serde::Deserialize;
use validation::validate;
use wasm_parser::{parse, read_wasm};

const USAGE: &str = "
Funky - a WebAssembly Interpreter

Usage:
  ./funky <input> [--stage0 | --stage1]
  ./funky (-h | --help)
  ./funky --version

Options:
  -h --help     Show this screen.
  --version     Show version.
  --stage0      Stop at Parser
  --stage1      Stop at Validation
";

#[derive(Debug, Deserialize)]
struct Args {
    flag_stage0: bool,
    flag_stage1: bool,
    arg_input: String,
}

fn main() {
    env_logger::init();

    let args: Args = Docopt::new(USAGE)
        .and_then(|d| d.deserialize())
        .unwrap_or_else(|e| e.exit());

    let reader = read_wasm!(args.arg_input);

    info!("Parsing wasm file");

    let module = parse(reader).unwrap();

    if args.flag_stage0 {
        println!("{:#?}", module);

        return;
    }

    let validation = validate(&module);

    if args.flag_stage1 {
        println!("{:#?}", validation);

        return;
    }

    let store = Store {
        funcs: Vec::new(),
        tables: Vec::new(),
        stack: Vec::new(),
        globals: Vec::new(),
        memory: Vec::new(),
    };

    let mi = ModuleInstance::new(module, store);
    info!("Constructing engine");
    let mut e = Engine::new(mi);
    info!("Invoking function {:?}", 1);
    e.invoke_function(1, vec![I32(2)]);
    println!("Last value on stack was: {:?}", e.store.stack.last())
}
