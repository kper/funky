extern crate wasm_parser;

use wasm_parser::{parse, read_wasm};

use std::env;

fn main() {
    env_logger::init();

    let reader = read_wasm!("./test_files/simple_bg.wasm");
    //let reader = read_wasm!("./test_files/empty.wasm");
    
    parse(reader);
}
