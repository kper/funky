#[macro_use]
extern crate log;
extern crate env_logger;
extern crate funky;
extern crate regex;

use docopt::Docopt;
use funky::cli::parse_args;
use funky::engine::{Engine, ModuleInstance};
use funky::value::Value;
use funky::value::Value::*;
use regex::Regex;
use regex::RegexSet;
use serde::Deserialize;
use validation::validate;
use wasm_parser::{parse, read_wasm};
use funky::config::Configuration;

const USAGE: &str = "
Funky - a WebAssembly Interpreter

Usage:
  ./funky <input> <function> [<args>...] [--stage0 | --stage1] [--spec] [--debugger]
  ./funky (-h | --help)
  ./funky --version

Options:
  -h --help     Show this screen.
  --version     Show version.
  --stage0      Stop at Parser.
  --stage1      Stop at Validation.
  --spec        Format output to be compliant for spec tests
  --debugger    Debugger runs the program.";

#[derive(Debug, Deserialize)]
struct Args {
    flag_stage0: bool,
    flag_stage1: bool,
    flag_spec: bool,
    arg_input: String,
    arg_function: String,
    arg_args: Vec<String>,
    flag_debugger: bool,
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

    let mut config = Configuration::new();

    if args.flag_debugger {
        config.enable_debugger();
    }

    let mi = ModuleInstance::new(&module);
    info!("Constructing engine");
    let mut e = Engine::new(mi, &module, config);
    debug!("engine {:#?}", e);

    debug!("Instantiation engine");

    if let Err(err) = e.instantiation(&module) {
        panic!("{}", err);
    }

    info!("Invoking function {:?}", 0);
    let inv_args = parse_args(args.arg_args);

    if let Err(err) = e.invoke_exported_function_by_name(
        &args.arg_function,
        inv_args
    ) {
        panic!("{}", err);
    }

    if args.flag_spec {
        println!("{:?}", e.store.stack.last())
    } else {
        println!("Last value on stack was: {:?}", e.store.stack.last())
    }
}
