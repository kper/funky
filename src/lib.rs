#[macro_use]
extern crate log;
extern crate wasm_parser;
pub mod engine;
pub mod allocation;
pub mod instantiation;

pub use wasm_parser::read_wasm;
pub use wasm_parser::parse;
pub use validation::validate;

#[cfg(test)]
mod tests;
