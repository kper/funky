extern crate wasm_parser;

use wasm_parser::{parse, read_wasm};

//use std::env;

fn main() {
    env_logger::init();

    //let reader = read_wasm!("./test_files/if_loop.wasm");
    let reader = read_wasm!("./test_files/arithmetic_i32.wasm");
    //let reader = read_wasm!("./test_files/empty.wasm");

    let _module = parse(reader);
    //println!("{:#?}", module);
}
