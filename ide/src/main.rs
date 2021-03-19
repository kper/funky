use crate::ir::ast::{Function, Program};
use crate::ir::wasm_ast::IR;
use anyhow::{Context, Result};
use funky::engine::module::ModuleInstance;
use funky::engine::*;
use log::debug;
use regex::Regex;
use std::path::PathBuf;
use structopt::StructOpt;
use validation::validate;
use wasm_parser::{parse, read_wasm};

use crate::icfg::convert::Convert;

use std::fs::File;
use std::io::Read;

use crate::solver::bfs::*;
use crate::solver::*;

use crate::grammar::*;

use std::io;
use termion::raw::IntoRawMode;
use tui::backend::TermionBackend;
use tui::Terminal;

use termion::event::Key;

use tui::{
    layout::{Alignment, Constraint, Direction, Layout},
    style::{Color, Modifier, Style},
    text::{Span, Spans},
    widgets::{Block, Borders, List, ListItem, Paragraph, Wrap},
};

use crate::ir::ast::Instruction;

#[macro_use]
extern crate lalrpop_util;

lalrpop_mod!(pub grammar);

mod counter;
mod icfg;
mod ir;
mod keyboard;
mod solver;
mod symbol_table;

use crate::keyboard::Events;

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
    },
    Ui {
        #[structopt(parse(from_os_str))]
        file: PathBuf,
        #[structopt(long)]
        ir: bool,
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
        Opt::Tikz { file, ir } => {
            if let Err(err) = tikz(file, ir) {
                eprintln!("ERROR: {}", err);
                err.chain()
                    .skip(1)
                    .for_each(|cause| eprintln!("because: {}", cause));
                std::process::exit(1);
            }
        }
        Opt::Ui { file, ir } => {
            if let Err(err) = ui(file, ir) {
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

    let res = convert.visit(&prog).context("Cannot create the graph")?;

    let output = crate::icfg::tikz::render_to(&res);

    println!("{}", output);

    Ok(())
}

use tui::widgets::ListState;

struct InstructionList<'a> {
    state: ListState,
    items: Vec<(usize, Option<&'a Instruction>, &'a str)>,
    //items: Vec<(usize, &'a str)>,
    current: usize,
}

fn ui(file: PathBuf, is_ir: bool) -> Result<()> {
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

    let mut graph = convert.visit(&prog).context("Cannot create the graph")?;

    let events = Events::new();

    let mut line_annoted_code = Vec::new();
    let mut line_no = 0;

    //let lines: Vec<_> = buffer.split("\n").collect();
    /*
    for line in buffer.split("\n") {
        if line.starts_with("define") {
            line_no = 0;
        }

        line_annoted_code.push((line_no, line));
        line_no += 1;
    }*/

    for function in prog.functions.iter() {
        line_annoted_code.push((line_no, None, function.name.as_str()));
        line_no += 1;

        for instruction in function.instructions.iter() {
            line_annoted_code.push((line_no, Some(instruction), function.name.as_str()));
            line_no += 1;
        }

        line_no = 0;
    }

    let mut stateful = InstructionList {
        state: ListState::default(),
        items: line_annoted_code,
        current: 0,
    };

    let stdout = io::stdout()
        .into_raw_mode()
        .context("Cannot create stdout")?;
    let backend = TermionBackend::new(stdout);
    let mut terminal = Terminal::new(backend).context("Cannot create new terminal")?;

    // clean the screen
    print!("{}[2J", 27 as char);

    let mut get_taints = |req: &Request| {
        let mut solver = IfdsSolver::new(BFS);
        solver.all_sinks(&mut graph, &req)
    };

    let mut input = String::new();

    let re = Regex::new(r"%[0-9]+ at [0-9a-zA-Z] [0-9]+").unwrap();

    let mut taints: Vec<Taint> = Vec::new();
    let instructions = prog
        .functions
        .iter()
        .map(|x| &x.instructions)
        .flatten()
        .collect::<Vec<_>>();

    let mut req = None;

    loop {
        /*
        if re.is_match(&input) {
            let splitted = input.split(" ").collect::<Vec<_>>();
            taints = get_taints(Request {
                variable: splitted.get(0).unwrap().to_string(),
                function: splitted.get(1).unwrap().to_string(),
                pc: splitted.get(2).unwrap().parse().unwrap(),
            });
        }*/

        if let Some(ref req) = req {
            taints = get_taints(req);
        }

        terminal
            .draw(|f| {
                let size = f.size();
                let block = Block::default()
                    .title("Taint analysis")
                    .borders(Borders::ALL);
                f.render_widget(block, size);

                let chunks = Layout::default()
                    .direction(Direction::Vertical)
                    .margin(1)
                    .constraints([Constraint::Percentage(90), Constraint::Percentage(10)].as_ref())
                    .split(f.size());

                let items: Vec<ListItem> = stateful
                    .items
                    .iter()
                    .enumerate()
                    //.skip(stateful.current)
                    //.take(50)
                    .map(|(_index, (line_no, instruction, function))| {
                        if let Some(instruction) = instruction {
                            if taints
                                .iter()
                                .find(|x| {
                                    let mut is_ok = false;

                                    if &x.from_function == function && &x.to_function == function {
                                        is_ok = true;
                                    }

                                    is_ok && match_taint(instruction, x)
                                })
                                .is_some()
                            {
                                ListItem::new(Spans::from(Span::styled(
                                    format!("{:02}| {:?}", line_no, instruction),
                                    Style::default()
                                        .bg(Color::LightRed)
                                        .add_modifier(Modifier::BOLD),
                                )))
                            } else {
                                ListItem::new(Spans::from(Span::styled(
                                    format!("{:02}| {:?}", line_no, instruction),
                                    Style::default().add_modifier(Modifier::ITALIC),
                                )))
                            }
                        } else {
                            // Display function
                            ListItem::new(Spans::from(Span::styled(
                                format!("{:02}| Function {}", line_no, function),
                                Style::default()
                                    .bg(Color::LightGreen)
                                    .add_modifier(Modifier::BOLD),
                            )))
                        }
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

                let input = Paragraph::new(input.clone())
                    .style(Style::default().fg(Color::Yellow))
                    .block(Block::default().title("Taint source"));

                f.render_widget(input, chunks[1]);
            })
            .context("Cannot draw on the screen")?;

        let key = events.next().context("Cannot get input")?;

        if key == Key::Char('q') {
            break;
        } else if key == Key::Down {
            if stateful.current < stateful.items.len() {
                stateful.current += 1;
                stateful.state.select(Some(stateful.current));
            }
        } else if key == Key::Up {
            if stateful.current > 0 {
                stateful.current -= 1;
                stateful.state.select(Some(stateful.current));
            }
        } else if key == Key::Esc {
            input = String::new();
            taints.clear();
        } else if key == Key::Backspace {
            input.pop();
            taints.clear();
        } else if key == Key::Right {
            let function = get_function_by_index(&prog.functions, stateful.current);
            let var = get_variable_by_index(&instructions, stateful.current);

            if function.is_some() && var.is_some() {
                req = Some(Request {
                    pc: stateful.current,
                    function: function.unwrap(),
                    variable: var.unwrap(),
                });
            }
        }

        /*else {
            match key {
                Key::Char(c) => {
                    input.push(c);
                }
                _ => {}
            }
        }*/
    }

    Ok(())
}

fn get_function_by_index(functions: &Vec<Function>, index: usize) -> Option<String> {
    let mut count = index as isize;
    for function in functions.iter() {
        count -= function.instructions.len() as isize - 1;

        if count <= 0 {
            return Some(function.name.clone());
        }
    }

    None
}

fn get_variable_by_index(instructions: &Vec<&Instruction>, index: usize) -> Option<String> {
    match instructions.get(index).unwrap() {
        Instruction::Unop(dest, _src) => Some(dest.clone()),
        Instruction::BinOp(dest, _src, _) => Some(dest.clone()),
        Instruction::Const(dest, _src) => Some(dest.clone()),
        Instruction::Assign(dest, _src) => Some(dest.clone()),
        Instruction::Conditional(src, _) => Some(src.clone()),
        _ => None,
    }
}

// check if the taint touches the instruction, then returns `true`.
fn match_taint(instruction: &Instruction, taint: &&Taint) -> bool {
    match instruction {
        Instruction::Unop(dest, src) => {
            &taint.to == dest || &taint.to == src || &taint.from == dest || &taint.from == src
        }
        Instruction::BinOp(dest, src1, src2) => {
            &taint.to == dest
                || &taint.to == src1
                || &taint.to == src2
                || &taint.from == dest
                || &taint.from == src1
                || &taint.from == src2
        }
        Instruction::Const(dest, _) => &taint.from == dest || &taint.to == dest,
        Instruction::Assign(dest, _) => &taint.from == dest || &taint.to == dest,
        Instruction::Call(callee, params, dest) => {
            &taint.to_function == callee
                && (params.contains(&taint.from)
                    || params.contains(&taint.to)
                    || dest.contains(&taint.from)
                    || dest.contains(&taint.to))
        }
        Instruction::Kill(dest) => &taint.from == dest || &taint.to == dest,
        Instruction::Conditional(dest, _) => &taint.from == dest || &taint.to == dest,
        Instruction::Return(dest) => dest.contains(&taint.from) || dest.contains(&taint.to),
        _ => false,
    }
}
