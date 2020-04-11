#[macro_use]
extern crate log;
extern crate env_logger;
mod engine;
use crate::engine::Engine;
use crate::engine::ModuleInstance;
use crate::engine::Value::*;
use wasm_parser::{parse, read_wasm};

fn main() {
    env_logger::init();
    let reader = read_wasm!("./tests/function_call.wasm");

    info!("Parsing wasm file");
    let module = parse(reader);
    let mi = ModuleInstance::new(module.unwrap());
    info!("Constructing engine");
    let mut e = Engine::new(mi);
    info!("Invoking function {:?}", 1);
    e.invoke_function(1, vec![I32(2)]);
    println!("Last value on stack was: {:?}", e.store.stack.last())
}
