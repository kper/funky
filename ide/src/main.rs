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

use std::fs::File;
use std::io::Read;

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
    Tikz {
        #[structopt(parse(from_os_str))]
        file: PathBuf,
        #[structopt(long)]
        ir: bool,
    }
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
        Opt::Tikz { file, ir} => {
            if let Err(err) = tikz(file, ir) {
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

fn tikz(file: PathBuf, is_ir: bool) -> Result<()> {
    let mut convert = Convert::new();

    let buffer = match is_ir {
        false => {
            let ir = ir(file).context("Cannot create intermediate representation of file")?;
            let buffer = ir.buffer().clone();

            buffer
        }
        true => {
            let mut fs = File::open(file).context("Cannot open ir file")?;
            let mut buffer = String::new();

            fs.read_to_string(&mut buffer)
                .context("Cannot read file to string")?;

            buffer
        }
    };

    let prog = ProgramParser::new().parse(&buffer).unwrap();

    let res = convert.visit(prog).context("Cannot create the graph")?;

    let output = crate::icfg::tikz::render_to(&res);

    println!("{}", output);

    Ok(())
}

