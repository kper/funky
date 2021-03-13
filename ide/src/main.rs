use crate::io::Cursor;
use crate::ssa::wasm_ast::IR;
use funky::engine::module::ModuleInstance;
use funky::engine::*;
use std::fs::File;
use std::io::{self, Write};
use std::{io::Read, path::PathBuf};
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

/*
#[derive(Debug, StructOpt)]
struct Opt {
    /// Input file
    #[structopt(parse(from_os_str))]
    input: PathBuf,

    /// Output file, stdout if not present
    #[structopt(parse(from_os_str))]
    output: Option<PathBuf>,
}*/

#[derive(Debug, StructOpt)]
#[structopt(name = "taint", about = "Taint analysis for wasm")]
enum Opt {
    Ssa {
        #[structopt(parse(from_os_str))]
        file: PathBuf,
    },
    Graph {
        #[structopt(parse(from_os_str))]
        file: PathBuf,
    },
}

fn main() {
    let opt = Opt::from_args();
    println!("{:?}", opt);

    match opt {
        Opt::Ssa { file } => {
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

            let mut ir = IR::new(&engine);

            ir.visit().unwrap();

            println!("{}", ir.buffer());
        }
        Opt::Graph { file } => {
            let mut convert = Convert::new();

            let mut fs = File::open(file).unwrap();
            let mut buffer = String::new();
            fs.read_to_string(&mut buffer).unwrap();

            let prog = ProgramParser::new()
                .parse(&buffer)
                .unwrap();

            let res = convert.visit(prog).unwrap();

            let mut dot = Cursor::new(Vec::new());
            render_to(&res, &mut dot);

            println!("{}", std::str::from_utf8(dot.get_ref()).unwrap());
        }
    }
}
