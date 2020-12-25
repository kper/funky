use docopt::Docopt;
use funky::cli::parse_args;
use funky::config::Configuration;
use funky::engine::{Engine, ModuleInstance};
use log::{debug, info};
use serde::Deserialize;
use std::io::{stdin, stdout, Write};
use termion::event::Key;
use termion::input::TermRead;
use termion::{input::MouseTerminal, raw::IntoRawMode, screen::AlternateScreen};
use tui::backend::TermionBackend;
use tui::Terminal;
use tui::{
    layout::{Alignment, Constraint, Direction, Layout},
    style::{Color, Modifier, Style},
    text::{Span, Spans},
    widgets::{Block, Borders, Paragraph, Wrap},
};
use validation::validate;
use wasm_parser::{parse, read_wasm};

use std::sync::Arc;
use std::sync::Mutex;

use crate::util::{Event, Events};

mod util;

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

fn main() -> Result<(), std::io::Error> {
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
    let e = Arc::new(Mutex::new(Engine::new(mi, &module, config)));
    debug!("engine {:#?}", e);

    debug!("Instantiation engine");

    if let Err(err) = e.lock().unwrap().instantiation(&module) {
        panic!("{}", err);
    }

    info!("Invoking function {:?}", 0);
    let inv_args = parse_args(args.arg_args);

    let args_function_cpy = args.arg_function.clone();

    std::thread::spawn(move || {
        if let Err(err) = e
            .clone()
            .lock()
            .unwrap()
            .invoke_exported_function_by_name(&args_function_cpy, inv_args)
        {
            panic!("{}", err);
        }

        let result = e.lock().unwrap().store.stack.last();
    });

    let stdin = stdin();
    let stdout = stdout().into_raw_mode()?;
    let stdout = MouseTerminal::from(stdout);
    let stdout = AlternateScreen::from(stdout);
    let backend = TermionBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    let events = Events::new();
    let mut scroll = (0, 0);
    loop {
        if let Event::Input(key) = events.next().unwrap() {
            if key == Key::Char('q') {
                break;
            } else if key == Key::Char('l') {
                let (y, mut x) = scroll;
                x += 1;
                scroll.1 = x;
            } else if key == Key::Char('h') {
                let (y, mut x) = scroll;
                if x > 0 {
                    x -= 1;
                    scroll.1 = x;
                }
            } else if key == Key::Char('j') {
                let (mut y, x) = scroll;
                y += 1;
                scroll.0 = y;
            } else if key == Key::Char('k') {
                let (mut y, x) = scroll;
                if y > 0 {
                    y -= 1;
                    scroll.0 = y;
                }
            }
        }

        terminal.draw(|f| {
            let size = f.size();

            let chunks = Layout::default()
                .direction(Direction::Horizontal)
                .margin(1)
                .constraints(
                    [
                        Constraint::Percentage(10),
                        Constraint::Percentage(20),
                    ]
                    .as_ref(),
                )
                .split(f.size());
            let block = Block::default().title("Hustensaft").borders(Borders::ALL);
            f.render_widget(block, size);

            let paragraph = Paragraph::new("Das ist ein TestThere are many variations of passages of Lorem Ipsum available, but the majority have suffered alteration in some form, by injected humour, or randomised words which don't look even slightly believable. If you are going to use a passage of Lorem Ipsum, you need to be sure there isn't anything embarrassing hidden in the middle of text. All the Lorem Ipsum generators on the Internet tend to repeat predefined chunks as necessary, making this the first true generator on the Internet. It uses a dictionary of over 200 Latin words, combined with a handful of model sentence structures, to generate Lorem Ipsum which looks reasonable. The generated Lorem Ipsum is therefore always free from repetition, injected humour, or non-characteristic words etc.")
                .style(Style::default())
                .alignment(Alignment::Left)
                .scroll(scroll)
                .wrap(Wrap { trim: true });

            f.render_widget(paragraph, chunks[0]);
        })?;
    }

    Ok(())
}
