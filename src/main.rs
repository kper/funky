use wasm_parser::{parse, read_wasm};

mod engine;
use crate::engine::Engine;
use crate::engine::ModuleInstance;
use crate::engine::Value::*;

fn main() {
    let reader = read_wasm!("./wasm_parser/test_files/simple_bg.wasm");

    let module = parse(reader);
    println!("{:#?}", module);
    let mi = ModuleInstance::new(module.unwrap());
    let mut e = Engine::new(mi);
    e.invoke_function(0, vec![I32(4), I32(4)]);
    println!("Last value on stack was: {:?}", e.store.stack.last())
}
