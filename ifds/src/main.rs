use crate::ir::wasm_ast::IR;
use anyhow::{Context, Result};
use funky::engine::module::ModuleInstance;
use funky::engine::*;
use itertools::Itertools;
use log::debug;
use std::path::PathBuf;
use structopt::StructOpt;
use validation::validate;
use wasm_parser::{parse, read_wasm};

use crate::icfg::flowfuncs::sparse_taint::flow::SparseTaintNormalFlowFunction;
use crate::icfg::flowfuncs::sparse_taint::initial::SparseTaintInitialFlowFunction;
use crate::icfg::flowfuncs::taint::flow::TaintNormalFlowFunction;
use crate::icfg::flowfuncs::taint::initial::TaintInitialFlowFunction;
use crate::icfg::tabulation::fast::TabulationFast;
use crate::icfg::tabulation::naive::TabulationNaive;
use crate::icfg::tabulation::orig::TabulationOriginal;
use crate::icfg::tabulation::sparse::TabulationSparse;
use crate::{solver::bfs::*, solver::*};

use meta::*;

use std::fs::File;
use std::io::{Read, Write};

use crate::grammar::*;

use std::io;
use termion::raw::IntoRawMode;
use tui::backend::TermionBackend;
use tui::Terminal;

use termion::event::Key;

use tui::{
    layout::{Constraint, Direction, Layout},
    style::{Color, Modifier, Style},
    text::{Span, Spans},
    widgets::{Block, Borders, List, ListItem, Paragraph},
};

use crate::ir::ast::Instruction;

use std::net::UdpSocket;

#[macro_use]
extern crate lalrpop_util;

lalrpop_mod!(pub grammar);

mod counter;
mod icfg;
mod ir;
mod keyboard;
mod meta;
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
        #[structopt(short)]
        function: String,
        #[structopt(long)]
        pc: usize,
    },
    Ui {
        #[structopt(parse(from_os_str))]
        file: PathBuf,
        #[structopt(long)]
        ir: bool,
        #[structopt(short, parse(from_os_str))]
        export_graph: Option<PathBuf>,
    },
    Repl {
        #[structopt(parse(from_os_str))]
        file: PathBuf,
        #[structopt(long)]
        ir: bool,
        #[structopt(short, parse(from_os_str))]
        export_graph: Option<PathBuf>,
    },
    Fast {
        #[structopt(parse(from_os_str))]
        file: PathBuf,
        #[structopt(long)]
        ir: bool,
        #[structopt(short, parse(from_os_str))]
        export_graph: Option<PathBuf>,
        #[structopt(short)]
        function: String,
        #[structopt(short)]
        pc: usize,
        #[structopt(short)]
        var: String,
    },
    Naive {
        #[structopt(parse(from_os_str))]
        file: PathBuf,
        #[structopt(long)]
        ir: bool,
        #[structopt(short, parse(from_os_str))]
        export_graph: Option<PathBuf>,
        #[structopt(short)]
        function: String,
        #[structopt(short)]
        pc: usize,
        #[structopt(short)]
        var: String,
    },
    Orig {
        #[structopt(parse(from_os_str))]
        file: PathBuf,
        #[structopt(long)]
        ir: bool,
        #[structopt(short, parse(from_os_str))]
        export_graph: Option<PathBuf>,
        #[structopt(short)]
        function: String,
        #[structopt(short)]
        pc: usize,
        #[structopt(short)]
        var: String,
    },
    Sparse {
        #[structopt(parse(from_os_str))]
        file: PathBuf,
        #[structopt(long)]
        ir: bool,
        #[structopt(short, parse(from_os_str))]
        export_graph: Option<PathBuf>,
        #[structopt(short)]
        function: String,
        #[structopt(short)]
        pc: usize,
        #[structopt(short)]
        var: String,
        #[structopt(short, parse(from_os_str))]
        meta: Option<PathBuf>,
    },
    Meta {
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
        Opt::Tikz {
            file,
            ir,
            function,
            pc,
        } => {
            if let Err(err) = tikz(file, ir, function, pc) {
                eprintln!("ERROR: {}", err);
                err.chain()
                    .skip(1)
                    .for_each(|cause| eprintln!("because: {}", cause));
                std::process::exit(1);
            }
        }
        Opt::Ui {
            file,
            ir,
            export_graph,
        } => {
            if let Err(err) = ui(file, ir, export_graph) {
                eprintln!("ERROR: {}", err);
                err.chain()
                    .skip(1)
                    .for_each(|cause| eprintln!("because: {}", cause));
                std::process::exit(1);
            }
        }
        Opt::Repl {
            file,
            ir,
            export_graph,
        } => {
            if let Err(err) = repl(file, ir, export_graph) {
                eprintln!("ERROR: {}", err);
                err.chain()
                    .skip(1)
                    .for_each(|cause| eprintln!("because: {}", cause));
                std::process::exit(1);
            }
        }
        Opt::Fast {
            file,
            ir,
            export_graph,
            function,
            pc,
            var,
        } => {
            if let Err(err) = fast(file, ir, export_graph, function, pc, var) {
                eprintln!("ERROR: {}", err);
                err.chain()
                    .skip(1)
                    .for_each(|cause| eprintln!("because: {}", cause));
                std::process::exit(1);
            }
        }
        Opt::Orig {
            file,
            ir,
            export_graph,
            function,
            pc,
            var,
        } => {
            if let Err(err) = orig(file, ir, export_graph, function, pc, var) {
                eprintln!("ERROR: {}", err);
                err.chain()
                    .skip(1)
                    .for_each(|cause| eprintln!("because: {}", cause));
                std::process::exit(1);
            }
        }
        Opt::Naive {
            file,
            ir,
            export_graph,
            function,
            pc,
            var,
        } => {
            if let Err(err) = naive(file, ir, export_graph, function, pc, var) {
                eprintln!("ERROR: {}", err);
                err.chain()
                    .skip(1)
                    .for_each(|cause| eprintln!("because: {}", cause));
                std::process::exit(1);
            }
        }
        Opt::Sparse {
            file,
            ir,
            export_graph,
            function,
            pc,
            var,
            meta,
        } => {
            if let Err(err) = sparse(file, ir, export_graph, function, pc, var, meta) {
                eprintln!("ERROR: {}", err);
                err.chain()
                    .skip(1)
                    .for_each(|cause| eprintln!("because: {}", cause));
                std::process::exit(1);
            }
        }
        Opt::Meta { file, ir } => {
            if let Err(err) = meta(file, ir) {
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

    ir.visit(&engine).context("Generating IR failed")?;

    Ok(ir)
}

fn tikz(file: PathBuf, is_ir: bool, function: String, pc: usize) -> Result<()> {
    let mut convert = TabulationFast::new(TaintInitialFlowFunction, TaintNormalFlowFunction);

    let buffer = match is_ir {
        false => {
            let ir = ir(file).context("Cannot create intermediate representation of file")?;
            let buffer = ir.buffer();

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

    let req = Request {
        function,
        pc,
        variable: None,
    };
    let (graph, state) = convert
        .visit(&prog, &req)
        .context("Cannot create the graph")?;

    let output = crate::icfg::tikz::render_to(&graph, &state);

    println!("{}", output);

    Ok(())
}

use tui::widgets::ListState;

struct InstructionList<'a> {
    state: ListState,
    items: Vec<(usize, usize, Option<&'a Instruction>, &'a str)>,
    current: usize,
}

fn ui(file: PathBuf, is_ir: bool, export_graph: Option<PathBuf>) -> Result<()> {
    let mut convert = TabulationSparse::new(
        SparseTaintInitialFlowFunction,
        SparseTaintNormalFlowFunction,
    );

    let buffer = match is_ir {
        false => {
            let ir = ir(file).context("Cannot create intermediate representation of file")?;
            let buffer = ir.buffer();

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

    let events = Events::new();

    let mut line_annoted_code = Vec::new();
    let mut pc = 0;
    let mut line_no = 0;

    for function in prog.functions.iter() {
        line_annoted_code.push((pc, line_no, None, function.name.as_str()));
        line_no += 1;

        for instruction in function.instructions.iter() {
            line_annoted_code.push((pc, line_no, Some(instruction), function.name.as_str()));
            pc += 1;
            line_no += 1;
        }

        pc = 0;
    }

    let mut stateful = InstructionList {
        state: ListState::default(),
        items: line_annoted_code.clone(),
        current: 0,
    };

    let mut get_taints = |req: &Request| {
        let res = convert.visit(&prog, &req);
        let (graph, state) = res.expect("Cannot create graph");

        //let taints = solver.all_sinks(&mut graph, &req);
        let sinks = graph
            .edges
            .iter()
            .map(|x| x.to())
            .filter(|x| &x.function == &req.function)
            .unique()
            .map(|x| Taint {
                function: x.function.clone(),
                pc: x.pc - 1,
                variable: x.belongs_to_var.clone(),
            })
            .collect::<Vec<_>>();

        (graph, state, sinks)
    };

    let mut input = String::new();

    let stdout = io::stdout()
        .into_raw_mode()
        .context("Cannot create stdout")?;
    let backend = TermionBackend::new(stdout);
    let mut terminal = Terminal::new(backend).context("Cannot create new terminal")?;

    // clean the screen
    print!("{}[2J", 27 as char);

    let mut taints: Vec<Taint> = Vec::new();

    let mut already_computed = false;
    let mut req = None;

    let udp_socket = setup_logging();

    loop {
        if let Some(ref req) = req {
            if !already_computed {
                let res = get_taints(req);
                taints = res.2;
                already_computed = true;
                let _ = udp_socket.send_to(format!("{:#?}", taints).as_bytes(), "127.0.0.1:4242");
                //.context("Cannot send logging information")?;

                if let Some(ref export_graph) = export_graph {
                    let output = crate::icfg::tikz2::render_to(&res.0, &res.1);

                    let mut fs = File::create(export_graph).context("Cannot write export file")?;
                    fs.write_all(output.as_bytes())
                        .context("Cannot write file")?;
                }
            }
        }

        struct Line<'a> {
            instruction: Option<&'a Instruction>,
            pc: usize,
            is_taint: bool,
            is_function: bool,
            function: &'a str,
        }

        terminal
            .draw(|f| {
                let size = f.size();
                let block = Block::default()
                    .title("Taint analysis")
                    .borders(Borders::ALL);
                f.render_widget(block, size);

                let chunks = Layout::default()
                    .direction(Direction::Horizontal)
                    .margin(1)
                    .constraints([Constraint::Percentage(80), Constraint::Percentage(20)].as_ref())
                    .split(f.size());

                let mut code: Vec<Line> = stateful
                    .items
                    .iter()
                    .enumerate()
                    .map(|(_index, (pc, _line_no, instruction, function))| {
                        if let Some(instruction) = instruction {
                            Line {
                                instruction: Some(instruction),
                                pc: *pc,
                                is_taint: false,
                                is_function: false,
                                function,
                            }
                        } else {
                            Line {
                                instruction: *instruction,
                                pc: *pc,
                                is_taint: false,
                                is_function: true,
                                function,
                            }
                        }
                    })
                    .collect();

                for taint in taints.iter() {
                    let i = code
                        .iter_mut()
                        .find(|x| x.function == taint.function.as_str() && x.pc == taint.pc);

                    if let Some(line) = i {
                        if let Some(instruction) = line.instruction {
                            if match_taint(instruction, &&taint) {
                                line.is_taint = true;
                            }
                        }
                    }
                }

                let items: Vec<ListItem> = code
                    .iter()
                    .map(|x| {
                        if !x.is_function {
                            if x.is_taint {
                                ListItem::new(Spans::from(Span::styled(
                                    format!("{:02}| {:?}", x.pc, x.instruction.unwrap()),
                                    Style::default()
                                        .bg(Color::LightRed)
                                        .add_modifier(Modifier::BOLD),
                                )))
                            } else {
                                ListItem::new(Spans::from(Span::styled(
                                    format!("{:02}| {:?}", x.pc, x.instruction.unwrap()),
                                    Style::default().add_modifier(Modifier::ITALIC),
                                )))
                            }
                        } else {
                            ListItem::new(Spans::from(Span::styled(
                                format!("{:02}| Function {}", x.pc, x.function),
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

                let input = Paragraph::new(input.to_string())
                    .style(Style::default())
                    .block(Block::default().title("Taints"));

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
            //let function = get_function_by_index(&prog.functions, stateful.current);
            let entry = get_variable_by_index(&line_annoted_code, stateful.current);

            if let Some(entry) = entry {
                req = Some(Request {
                    pc: entry.2,
                    function: entry.1.clone(),
                    variable: Some(entry.0.clone()),
                });
                input = format!("{:#?}", req);
                already_computed = false;
            }
        }
    }

    Ok(())
}

fn repl(file: PathBuf, is_ir: bool, export_graph: Option<PathBuf>) -> Result<()> {
    let mut convert = TabulationFast::new(TaintInitialFlowFunction, TaintNormalFlowFunction);

    let buffer = match is_ir {
        false => {
            let ir = ir(file).context("Cannot create intermediate representation of file")?;
            let buffer = ir.buffer();

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

    let mut get_taints = |req: &Request| {
        let mut solver = IfdsSolver;

        let res = convert.visit(&prog, &req);
        let (mut graph, state) = res.expect("Cannot create graph");

        let taints = solver.all_sinks(&mut graph, &req);

        (graph, state, taints)
    };

    loop {
        let mut buffer = String::new();
        io::stdin().read_to_string(&mut buffer)?;

        let splitted = buffer.trim().split(' ').collect::<Vec<_>>();

        let req = Request {
            function: splitted.get(0).unwrap().to_string(),
            pc: splitted.get(1).unwrap().parse::<usize>().unwrap(),
            variable: splitted.get(2).map(|x| x.to_string()),
        };

        println!("{:?}", req);

        let res = get_taints(&req);
        let state = res.1;
        let taints = res.2.context("Cannot get taints for ui")?;

        println!("{:#?}", taints);

        if let Some(ref export_graph) = export_graph {
            let output = crate::icfg::tikz::render_to(&res.0, &state);

            let mut fs = File::create(export_graph).context("Cannot write export file")?;
            fs.write_all(output.as_bytes())
                .context("Cannot write file")?;
        }
    }
}

fn fast(
    file: PathBuf,
    is_ir: bool,
    export_graph: Option<PathBuf>,
    function: String,
    pc: usize,
    var: String,
) -> Result<()> {
    let mut convert = TabulationFast::new(TaintInitialFlowFunction, TaintNormalFlowFunction);

    let buffer = match is_ir {
        false => {
            let ir = ir(file).context("Cannot create intermediate representation of file")?;
            let buffer = ir.buffer();

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

    let mut get_taints = |req: &Request| {
        let mut solver = IfdsSolver;

        let res = convert.visit(&prog, &req);
        let (mut graph, state) = res.expect("Cannot create graph");

        let taints = solver.all_sinks(&mut graph, &req);

        (graph, state, taints)
    };

    let req = Request {
        function,
        pc,
        variable: Some(var),
    };

    let res = get_taints(&req);
    let state = res.1;
    let taints = res.2.context("Cannot get taints for ui")?;

    println!("{:#?}", taints);

    if let Some(ref export_graph) = export_graph {
        let output = crate::icfg::tikz::render_to(&res.0, &state);

        let mut fs = File::create(export_graph).context("Cannot write export file")?;
        fs.write_all(output.as_bytes())
            .context("Cannot write file")?;
    }

    Ok(())
}

fn naive(
    file: PathBuf,
    is_ir: bool,
    export_graph: Option<PathBuf>,
    function: String,
    pc: usize,
    var: String,
) -> Result<()> {
    let mut convert = TabulationNaive::default();

    let buffer = match is_ir {
        false => {
            let ir = ir(file).context("Cannot create intermediate representation of file")?;
            let buffer = ir.buffer();

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

    let mut get_taints = |req: &Request| {
        let mut solver = Bfs;
        let (mut graph, state, _) = convert.visit(&prog).unwrap();

        let taints = solver.all_sinks(&mut graph, &state, &req);

        (graph, state, taints)
    };

    let req = Request {
        function,
        pc,
        variable: Some(var),
    };

    let res = get_taints(&req);
    let state = res.1;
    let taints = res.2;

    println!("{:#?}", taints);

    if let Some(ref export_graph) = export_graph {
        let output = crate::icfg::tikz::render_to(&res.0, &state);

        let mut fs = File::create(export_graph).context("Cannot write export file")?;
        fs.write_all(output.as_bytes())
            .context("Cannot write file")?;
    }

    Ok(())
}

fn orig(
    file: PathBuf,
    is_ir: bool,
    export_graph: Option<PathBuf>,
    function: String,
    pc: usize,
    var: String,
) -> Result<()> {
    let mut convert = TabulationOriginal::default();

    let buffer = match is_ir {
        false => {
            let ir = ir(file).context("Cannot create intermediate representation of file")?;
            let buffer = ir.buffer();

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

    let mut get_taints = |req: &Request| {
        let mut solver = IfdsSolver;

        let (mut graph, state) = convert.visit(&prog, req).unwrap();

        let taints = solver.all_sinks(&mut graph, req);

        (graph, state, taints)
    };

    let req = Request {
        function,
        pc,
        variable: Some(var),
    };

    let res = get_taints(&req);
    let state = res.1;
    let taints = res.2;

    println!("{:#?}", taints);

    if let Some(ref export_graph) = export_graph {
        let output = crate::icfg::tikz::render_to(&res.0, &state);

        let mut fs = File::create(export_graph).context("Cannot write export file")?;
        fs.write_all(output.as_bytes())
            .context("Cannot write file")?;
    }

    Ok(())
}

fn get_variable_by_index(
    instructions: &Vec<(usize, usize, Option<&Instruction>, &str)>,
    index: usize,
) -> Option<(String, String, usize)> {
    let (pc, _line_no, instruction, function) = instructions.get(index).unwrap();
    match instruction {
        Some(Instruction::Unop(dest, _src)) => {
            Some((dest.clone(), function.clone().to_string(), *pc))
        }
        Some(Instruction::BinOp(dest, _src, _)) => {
            Some((dest.clone(), function.clone().to_string(), *pc))
        }
        Some(Instruction::Const(dest, _src)) => {
            Some((dest.clone(), function.clone().to_string(), *pc))
        }
        Some(Instruction::Assign(dest, _src)) => {
            Some((dest.clone(), function.clone().to_string(), *pc))
        }
        Some(Instruction::Conditional(src, _)) => {
            Some((src.clone(), function.clone().to_string(), *pc))
        }
        Some(Instruction::Unknown(dest)) => Some((dest.clone(), function.clone().to_string(), *pc)),
        Some(Instruction::Phi(dest, _, _)) => {
            Some((dest.clone(), function.clone().to_string(), *pc))
        }
        _ => None,
    }
}

fn sparse(
    file: PathBuf,
    is_ir: bool,
    export_graph: Option<PathBuf>,
    function: String,
    pc: usize,
    var: String,
    meta: Option<PathBuf>,
) -> Result<()> {
    let mut convert = TabulationSparse::new(
        SparseTaintInitialFlowFunction,
        SparseTaintNormalFlowFunction,
    );

    let buffer = match is_ir {
        false => {
            let ir = ir(file).context("Cannot create intermediate representation of file")?;
            let buffer = ir.buffer();

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

    let mut get_taints = |req: &Request| {
        let res = convert.visit(&prog, &req);
        let (graph, state) = res.expect("Cannot create graph");

        let taints = graph
            .edges
            .iter()
            .map(|x| x.to())
            .filter(|x| &x.function == &req.function)
            .map(|x| Taint {
                variable: x.belongs_to_var.clone(),
                pc: x.pc,
                function: x.function.clone(),
            })
            .collect::<Vec<_>>();

        (graph, state, taints)
    };

    let req = Request {
        function,
        pc,
        variable: Some(var),
    };

    let res = get_taints(&req);
    let state = res.1;
    let taints = res.2;

    println!("{:#?}", taints);

    if let Some(ref export_graph) = export_graph {
        let output = crate::icfg::tikz2::render_to(&res.0, &state);

        let mut fs = File::create(export_graph).context("Cannot write export file")?;
        fs.write_all(output.as_bytes())
            .context("Cannot write file")?;
    }

    if let Some(ref meta) = meta {
        let m = meta_sparse(&prog, &res.0, &state, convert.get_defuse());

        let mut fs = File::create(meta).context("Cannot write meta file")?;
        let output = serde_json::to_string_pretty(&m)?;
        fs.write_all(output.as_bytes())
            .context("Cannot write file")?;
    }

    Ok(())
}

fn meta(file: PathBuf, is_ir: bool) -> Result<()> {
    let buffer = match is_ir {
        false => {
            let ir = ir(file).context("Cannot create intermediate representation of file")?;
            let buffer = ir.buffer();

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
    let meta = meta_naive(&prog);

    println!(
        "{}",
        serde_json::to_string_pretty(&meta).context("Cannot construct json")?
    );

    Ok(())
}

// check if the taint touches the instruction, then returns `true`.
fn match_taint(instruction: &Instruction, taint: &&Taint) -> bool {
    match instruction {
        Instruction::Unop(dest, src) => &taint.variable == dest || &taint.variable == src,
        Instruction::BinOp(dest, src1, src2) => {
            &taint.variable == dest || &taint.variable == src1 || &taint.variable == src2
        }
        Instruction::Const(dest, _) => &taint.variable == dest,
        Instruction::Assign(dest, src) => &taint.variable == dest || &taint.variable == src,
        Instruction::Call(_callee, params, dest) => {
            params.contains(&taint.variable) || dest.contains(&taint.variable)
        }
        Instruction::CallIndirect(_callee, params, dest) => {
            params.contains(&taint.variable) || dest.contains(&taint.variable)
        }
        Instruction::Kill(dest) => &taint.variable == dest,
        Instruction::Conditional(dest, _) => &taint.variable == dest,
        Instruction::Return(dest) => dest.contains(&taint.variable),
        Instruction::Phi(dest, src1, src2) => {
            &taint.variable == dest || &taint.variable == src1 || &taint.variable == src2
        }
        Instruction::Store(src1, _, src2) => &taint.variable == src1 || &taint.variable == src2,
        Instruction::Load(dest, _, src) => &taint.variable == dest || &taint.variable == src,
        _ => false,
    }
}

fn setup_logging() -> UdpSocket {
    let socket = UdpSocket::bind("127.0.0.1:34254").expect("couldn't bind to address");
    socket
}
