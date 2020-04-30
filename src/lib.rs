#[macro_use]
extern crate log;
extern crate wasm_parser;
pub mod engine;
pub mod allocation;
pub mod instantiation;

#[cfg(test)]
mod tests;
