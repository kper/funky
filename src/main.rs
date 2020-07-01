#[macro_use]
extern crate log;
extern crate env_logger;
extern crate funky;
extern crate regex;

use docopt::Docopt;
use funky::engine::Value;
use funky::engine::Value::*;
use funky::engine::{Engine, ModuleInstance};
use regex::Regex;
use regex::RegexSet;
use serde::Deserialize;
use validation::validate;
use wasm_parser::{parse, read_wasm};

const USAGE: &str = "
Funky - a WebAssembly Interpreter

Usage:
  ./funky <input> <function> [<args>...] [--stage0 | --stage1] [--spec]
  ./funky (-h | --help)
  ./funky --version

Options:
  -h --help     Show this screen.
  --version     Show version.
  --stage0      Stop at Parser
  --stage1      Stop at Validation
  --spec        Format output to be compliant for spec tests
";

#[derive(Debug, Deserialize)]
struct Args {
    flag_stage0: bool,
    flag_stage1: bool,
    flag_spec: bool,
    arg_input: String,
    arg_function: String,
    arg_args: Vec<String>,
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

    let mi = ModuleInstance::new(&module);
    info!("Constructing engine");
    let mut e = Engine::new(mi, &module);
    debug!("engine {:#?}", e);

    debug!("Instantiation engine");

    e.instantiation(&module);

    info!("Invoking function {:?}", 0);
    let inv_args = parse_args(args.arg_args);
    e.invoke_exported_function_by_name(&args.arg_function, inv_args);
    if args.flag_spec {
        println!("{:?}", e.store.stack.last())
    } else {
        println!("Last value on stack was: {:?}", e.store.stack.last())
    }
}

fn parse_args(args: Vec<String>) -> Vec<Value> {
    let matchers = &[
        r"I32\((-?[0-9]+)\)",
        r"I64\((-?[0-9]+)\)",
        r"F32\((-?[0-9]+(\.[0-9]+)?)\)",
        r"F64\((-?[0-9]+(\.[0-9]+)?)\)",
        r"F32\(inf\)",
        r"F32\(-inf\)",
        r"F64\(inf\)",
        r"F64\(-inf\)",
        r"F32\(nan\)",
        r"F64\(nan\)",
    ];
    let set = RegexSet::new(matchers).unwrap();
    args.iter()
        .map(|a| {
            let matches = set.matches(a);
            debug!("matches: {:?}", matches);
            if matches.matched(0) {
                let re = Regex::new(matchers[0]).unwrap();
                let caps = re.captures(a).unwrap();
                match caps[1].parse::<i32>() {
                    Ok(x) => I32(x),
                    _ => I32(caps[1].parse::<u32>().unwrap() as i32),
                }
            } else if matches.matched(1) {
                let re = Regex::new(matchers[1]).unwrap();
                let caps = re.captures(a).unwrap();
                match caps[1].parse::<i64>() {
                    Ok(x) => I64(x),
                    _ => I64(caps[1].parse::<u64>().unwrap() as i64),
                }
            } else if matches.matched(2) {
                let re = Regex::new(matchers[2]).unwrap();
                let caps = re.captures(a).unwrap();
                F32(caps[1].parse::<f32>().unwrap())
            } else if matches.matched(3) {
                let re = Regex::new(matchers[3]).unwrap();
                let caps = re.captures(a).unwrap();
                F64(caps[1].parse::<f64>().unwrap())
            } else if matches.matched(4) {
                F32(f32::INFINITY)
            } else if matches.matched(5) {
                F32(-f32::INFINITY)
            } else if matches.matched(6) {
                F64(f64::INFINITY)
            } else if matches.matched(7) {
                F64(-f64::INFINITY)
            } else if matches.matched(8) {
                F32(f32::NAN)
            } else if matches.matched(9) {
                F64(f64::NAN)
            } else {
                panic!("Invalid parameter type specified {}", a);
            }
        })
        .collect()
}
