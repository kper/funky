use crate::io::Cursor;
use crate::ssa::wasm_ast::IR;
use anyhow::{Context, Result};
use funky::engine::module::ModuleInstance;
use funky::engine::*;
use log::debug;
use std::io::{self};
use std::path::PathBuf;
use structopt::StructOpt;
use validation::validate;
use wasm_parser::{parse, read_wasm};

use crate::icfg::convert::Convert;
use crate::icfg::graphviz::render_to;

use crate::grammar::*;

#[macro_use]
extern crate lalrpop_util;

lalrpop_mod!(pub grammar);

mod counter;
mod icfg;
mod ssa;
mod symbol_table;

#[cfg(test)]
mod tests;

#[derive(Debug, StructOpt)]
#[structopt(name = "taint", about = "Taint analysis for wasm")]
enum Opt {
    Ir {
        #[structopt(parse(from_os_str))]
        file: PathBuf,
    },
    Graph {
        #[structopt(parse(from_os_str))]
        file: PathBuf,
    },
}

fn main() {
    env_logger::init();
    let opt = Opt::from_args();
    debug!("{:?}", opt);

    match opt {
        Opt::Ir { file } => {
            match ir(file) {
                Ok(ir) => {
                    println!("{}", ir.buffer());
                }
                Err(err) => {
                    eprintln!("ERROR: {}", err);
                    err.chain()
                        .skip(1)
                        .for_each(|cause| eprintln!("because: {}", cause));
                    std::process::exit(1);
                }
            };
        }
        Opt::Graph { file } => {
            if let Err(err) = graph(file) {
                eprintln!("ERROR: {}", err);
                err.chain()
                    .skip(1)
                    .for_each(|cause| eprintln!("because: {}", cause));
                std::process::exit(1);
            }
        }
    }
}

fn ir(file: PathBuf) -> Result<IR> {
    let file = read_wasm!(file);
    let module = parse(file).expect("Parsing failed");
    assert!(validate(&module).is_ok());

    let imports = Vec::new();

    let instance = ModuleInstance::new(&module);
    let engine = Engine::new(
        instance,
        &module,
        Box::new(funky::debugger::RelativeProgramCounter::default()),
        &imports,
    )
    .unwrap();

    let mut ir = IR::new();

    ir.visit(&engine).unwrap();

    Ok(ir)
}

fn graph(file: PathBuf) -> Result<()> {
    let mut convert = Convert::new();

    //let mut fs = File::open(file).context("Cannot open file")?;

    let ir = ir(file).context("Cannot read intermediate representation of file")?;

    let buffer = ir.buffer().clone();

    let prog = ProgramParser::new().parse(&buffer).unwrap();

    let res = convert.visit(prog).context("Cannot create the graph")?;

    let mut dot = Cursor::new(Vec::new());
    render_to(&res, &mut dot);

    println!("{}", std::str::from_utf8(dot.get_ref()).unwrap());

    Ok(())
}
