use docopt::Docopt;
use funky::cli::parse_args;
use funky::debugger::DebuggerProgramCounter;
use funky::engine::import_resolver::Imports;
use funky::engine::module::ModuleInstance;
use funky::engine::Engine;
use log::{debug, info};
use serde::Deserialize;
use std::io::{stdin, stdout};
use termion::event::Key;
use termion::{input::MouseTerminal, raw::IntoRawMode, screen::AlternateScreen};
use tui::backend::TermionBackend;
use tui::Terminal;
use tui::{
    layout::{Alignment, Constraint, Direction, Layout},
    style::{Color, Modifier, Style},
    text::{Span, Spans},
    widgets::{Block, Borders, List, ListItem, Paragraph, Wrap},
};
use validation::validate;
use wasm_parser::core::{Instruction, InstructionWrapper};
use wasm_parser::{parse, read_wasm};

use std::sync::atomic::AtomicBool;
use std::sync::mpsc::channel;

use crate::util::{Events, StatefulList};
use anyhow::{Context, Result};
use std::sync::{Arc, Mutex};

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

fn main() -> Result<()> {
    env_logger::init();

    let args: Args = Docopt::new(USAGE)
        .and_then(|d| d.deserialize())
        .unwrap_or_else(|e| e.exit());

    let reader = read_wasm!(args.arg_input);

    info!("Parsing wasm file");

    let module = parse(reader)?;
    let _validation = validate(&module).context("Validation failed")?;

    let mi = ModuleInstance::new(&module);
    info!("Constructing engine");

    let (instruction_watcher_tx, instruction_watcher_rx) = channel();
    let (instruction_advancer_tx, instruction_advancer_rx) = channel();
    let debugger =
        DebuggerProgramCounter::new(instruction_watcher_tx, instruction_advancer_rx).unwrap();

    let e = Arc::new(Mutex::new(
        Engine::new(mi, &module, Box::new(debugger), &Imports::new()).expect("Cannot create engine"),
    ));
    debug!("engine {:#?}", e);

    info!("Invoking function {:?}", 0);
    let inv_args = parse_args(args.arg_args);

    let args_function_cpy = args.arg_function;

    let lock = e.lock().unwrap();
    let copy = lock.module.get_code().clone();

    let engine = e;

    let terminated = Arc::new(AtomicBool::new(false));
    let cpy = terminated.clone();

    std::thread::spawn(move || {
        if let Err(err) = engine
            .lock()
            .unwrap()
            .invoke_exported_function_by_name(&args_function_cpy, inv_args)
        {
            panic!("{}", err);
        }

        println!("{:?}", engine.lock().unwrap().store.stack.last());

        cpy.store(true, std::sync::atomic::Ordering::Relaxed);

        //std::process::exit(0);
    });

    let _stdin = stdin();
    let stdout = stdout().into_raw_mode()?;
    let stdout = MouseTerminal::from(stdout);
    let stdout = AlternateScreen::from(stdout);
    let backend = TermionBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    let events = Events::new();
    let mut scroll = (0, 0);
    let mut scroll2 = (0, 0);

    let mut state = None;

    let functions: Vec<_> = copy.into_iter().map(|w| w.code.clone()).flatten().collect();
    let instructions = get_instructions(&functions); //expands the blocks

    let mut stateful = StatefulList::with_items(instructions);

    loop {
        if terminated.load(std::sync::atomic::Ordering::Relaxed) {
            break;
        }

        let key = events.next().unwrap();

        if key == Key::Char('q') {
            break;
        } else if key == Key::Char('l') {
            let (_y, mut x) = scroll;
            x += 1;
            scroll.1 = x;
        } else if key == Key::Char('h') {
            let (_y, mut x) = scroll;
            if x > 0 {
                x -= 1;
                scroll.1 = x;
            }
        } else if key == Key::Char('j') {
            let (mut y, _x) = scroll;
            y += 1;
            scroll.0 = y;
        } else if key == Key::Char('k') {
            let (mut y, _x) = scroll;
            if y > 0 {
                y -= 1;
                scroll.0 = y;
            }
        } else if key == Key::Down {
            let (mut y, _x) = scroll2;
            y += 1;
            scroll2.0 = y;
        } else if key == Key::Up {
            let (mut y, _x) = scroll2;
            if y > 0 {
                y -= 1;
                scroll2.0 = y;
            }
        } else if key == Key::Backspace {
            instruction_advancer_tx.send(()).unwrap();
            state = Some(instruction_watcher_rx.recv().unwrap()); // Blocking
            if let Some(ref state) = state {
                stateful.find_by_id(state.get_pc());
            }
        }

        terminal.draw(|f| {
            let size = f.size();

            let chunks = Layout::default()
                .direction(Direction::Horizontal)
                .margin(1)
                .constraints([Constraint::Percentage(70), Constraint::Percentage(30)].as_ref())
                .split(f.size());
            let block = Block::default().title("Hustensaft").borders(Borders::ALL);
            f.render_widget(block, size);

            let items: Vec<ListItem> = stateful
                .items
                .iter()
                .map(|w| {
                    ListItem::new(Spans::from(Span::styled(
                        format!("{}", w.get_instruction()),
                        Style::default().add_modifier(Modifier::ITALIC),
                    )))
                    .style(Style::default())
                })
                .collect();

            let list = List::new(items)
                .highlight_style(
                    Style::default()
                        .bg(Color::LightYellow)
                        .add_modifier(Modifier::BOLD),
                )
                .highlight_symbol(">> ");

            f.render_stateful_widget(list, chunks[0], &mut stateful.state);

            if let Some(state) = state.clone() {
                let pc = Paragraph::new(format!("State {}", state))
                    .style(Style::default())
                    .alignment(Alignment::Left)
                    .scroll(scroll2)
                    .wrap(Wrap { trim: false });

                f.render_widget(pc, chunks[1]);
            } else {
                let no_state = Paragraph::new("No state")
                    .style(Style::default())
                    .alignment(Alignment::Left)
                    .wrap(Wrap { trim: false });

                f.render_widget(no_state, chunks[1]);
            }
        })?;
    }

    Ok(())
}

/// Block structures save instructions in enum variants.
/// This function expands it and flattens to linearize a list.
fn get_instructions(instructions: &[InstructionWrapper]) -> Vec<&InstructionWrapper> {
    let mut result = Vec::new();

    for i in instructions {
        match i.get_instruction() {
            Instruction::OP_BLOCK(_, block) => {
                result.push(i);
                result.extend(&get_instructions(block.get_instructions()));
            }
            Instruction::OP_LOOP(_, block) => {
                result.push(i);
                result.extend(&get_instructions(block.get_instructions()));
            }
            Instruction::OP_IF(_, block) => {
                result.push(i);
                result.extend(&get_instructions(block.get_instructions()));
            }
            Instruction::OP_IF_AND_ELSE(_, block1, block2) => {
                result.push(i);
                result.extend(&get_instructions(block1.get_instructions()));
                result.extend(&get_instructions(block2.get_instructions()));
            }
            _ => {
                result.push(i);
            }
        }
    }

    result
}
