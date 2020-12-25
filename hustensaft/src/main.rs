use std::io;
use tui::Terminal;
use tui::backend::TermionBackend;
use termion::raw::IntoRawMode;
use tui::widgets::{Widget, Block, Borders};
use tui::layout::{Layout, Constraint, Direction};
use funky::cli::parse_args;
use funky::engine::{Engine, ModuleInstance};
use validation::validate;
use wasm_parser::{parse, read_wasm};
use funky::config::Configuration;
use docopt::Docopt;
use log::{debug, info};
use serde::Deserialize;

const USAGE: &str = "
Hustensaft - a debugger for the  WebAssembly Interpreter funky

Usage:
  ./funky <input> <function> [<args>...] 
  ./funky (-h | --help)
  ./funky --version

Options:
  -h --help     Show this screen.
  --version     Show version.";

#[derive(Debug, Deserialize)]
struct Args {
    arg_input: String,
    arg_function: String,
    arg_args: Vec<String>,
}

fn main() -> Result<(), io::Error> {
    env_logger::init();

    let args: Args = Docopt::new(USAGE)
        .and_then(|d| d.deserialize())
        .unwrap_or_else(|e| e.exit());

    let mut config = Configuration::new();
    config.enable_debugger();

    let reader = read_wasm!(args.arg_input);

    info!("Parsing wasm file");

    let module = parse(reader).unwrap();
    let validation = validate(&module);

    let mi = ModuleInstance::new(&module);
    info!("Constructing engine");
    let mut e = Engine::new(mi, &module, config);
    debug!("engine {:#?}", e);

    debug!("Instantiation engine");

    if let Err(err) = e.instantiation(&module) {
        panic!("{}", err);
    }

    info!("Invoking function {:?}", 0);
    let inv_args = parse_args(args.arg_args);

    if let Err(err) = e.invoke_exported_function_by_name(
        &args.arg_function,
        inv_args
    ) {
        panic!("{}", err);
    }

    //let stdout = io::stdout().into_raw_mode()?;
    //let backend = TermionBackend::new(stdout);
    //let mut terminal = Terminal::new(backend)?;

    

    let result = e.store.stack.last();

    Ok(())



    /*
   terminal.draw(|f| {
        let size = f.size();
        let block = Block::default()
            .title("Block")
            .borders(Borders::ALL);
        f.render_widget(block, size);
    })*/
}
