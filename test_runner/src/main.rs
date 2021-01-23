#![allow(unused)]

#[macro_use]
extern crate funky;

use std::cell::RefCell;
use std::collections::HashMap;
use std::fs::{create_dir, read_dir, read_to_string, remove_file, DirEntry, File, OpenOptions};
use std::io::Write;
use std::path::Path;
use std::rc::Rc;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::{Arc, Mutex};

use funky::debugger::RelativeProgramCounter;
use funky::engine::import_resolver::{Import, Imports};
use funky::engine::module::ModuleInstance;
use funky::engine::stack::StackContent;
use funky::engine::store::GlobalInstance;
use funky::engine::Engine;
use funky::value::Value;
use funky::{parse, read_wasm, validate};

use log::{debug, error};

use json::*;

mod json;

#[derive(Default)]
struct Stats {
    reported_ok: AtomicUsize,
    total_count: AtomicUsize,
}

macro_rules! remove_test_results_with_ending {
    ($ending:expr) => {
        read_dir("./test_results")
            .expect("No ./test_results directory found. Please create one")
            .filter(|w| {
                w.as_ref()
                    .unwrap()
                    .file_name()
                    .into_string()
                    .clone()
                    .unwrap()
                    .split(".")
                    .last()
                    .unwrap()
                    == $ending
            })
            .map(|w| w.unwrap())
            .for_each(|w| {
                remove_file(w.path()).expect("Cannot delete file");
            });
    };
}

fn main() {
    env_logger::init();

    let cmd_arguments: Vec<_> = std::env::args().collect();

    debug!("Cmd arguments {:?}", cmd_arguments);

    remove_file("./report.csv");
    //remove_file("./test_results/*.csv");

    remove_test_results_with_ending!("csv");
    remove_test_results_with_ending!("output");

    create_dir("./test_results");

    let mut file = OpenOptions::new()
        .create(true)
        .write(true)
        .truncate(true)
        .open("./report.csv")
        .expect("Cannot create report");

    file.write_all(b"Path,Status,Case,Args").unwrap();

    let args = ::std::env::args().collect::<Vec<_>>();

    debug!("args {:?}", args);

    let paths = match args.get(1) {
        // Get all files with .json
        Some(test_file) => read_dir("./testsuite")
            .expect("Cannot read ./testsuite")
            .filter(|w| {
                w.as_ref()
                    .unwrap()
                    .file_name()
                    .into_string()
                    .unwrap()
                    .split('.')
                    .last()
                    .unwrap()
                    == "json"
            })
            .map(|w| w.expect("Error with DirEntry"))
            .filter(|w| w.path().file_name() == Path::new(test_file).file_name())
            .collect::<Vec<_>>(),
        None => read_dir("./testsuite")
            .expect("Cannot read ./testsuite")
            .filter(|w| {
                w.as_ref()
                    .unwrap()
                    .file_name()
                    .into_string()
                    .unwrap()
                    .split('.')
                    .last()
                    .unwrap()
                    == "json"
            })
            .map(|w| w.unwrap())
            .collect::<Vec<_>>(),
    };

    debug!("paths {:?}", paths);

    let mut handlers = Vec::new();
    let stdouts = Arc::new(Mutex::new(Vec::new()));
    let length = paths.len();

    // Percentage
    let total_stat = Arc::new(Stats::default());

    for path in paths {
        let st = stdouts.clone();
        //let counter = counter.clone();
        let total_stat = total_stat.clone();

        let fancy_path = path.file_name().to_str().unwrap().to_string();
        let copy_path = fancy_path.clone();

        let cmd_arguments = cmd_arguments.clone();

        let handler = std::thread::Builder::new()
            .stack_size(32 * 1024 * 1024 * 64) // some tests require large stack size
            .spawn(move || {
                println!("--- Running {} ---", fancy_path);

                // Running the spec test
                let stdout = run_spec_test(&path, total_stat, &cmd_arguments);

                st.lock().unwrap().push(stdout);
            })
            .expect("Cannot spawn thread");

        handlers.push((handler, copy_path));
    }

    for (h, p) in handlers {
        if h.join().is_err() {
            eprintln!("Exit status reported error for {}", p);
            //println!("{}", stdouts.clone().lock().unwrap().join("\n"));
        }
    }

    let stdout_guard = stdouts.lock().unwrap();

    println!(
        "{}",
        stdout_guard
            .iter()
            .filter(|x| x != &"")
            .cloned()
            .collect::<Vec<_>>()
            .join("\n")
    );

    println!("Reporting total:");

    let total = total_stat.total_count.load(Ordering::Relaxed);
    let reported_ok = total_stat.reported_ok.load(Ordering::Relaxed);
    println!("Total: {}", total);
    println!("Ok: {}", reported_ok);
    println!("Failed: {}", total - reported_ok);
    println!("Ok %: {:#.4}", reported_ok as f64 / total as f64);
    println!(
        "Failed %: {:#.4}",
        (total - reported_ok) as f64 / total as f64
    );
}

/// `cmd_arguments` are function name which we filter (just for `assert_return`)
fn run_spec_test(path: &DirEntry, total_stats: Arc<Stats>, cmd_arguments: &[String]) -> String {
    let case_stats = Stats::default();

    let h = path.path();
    let p = h.to_str().unwrap();
    let mut buffer = read_to_string(p).unwrap();

    let fs: TestFile = serde_json::from_str(&buffer).unwrap();
    let count = fs.get_len_cases();

    // WASM modules are splitted across multiple files
    let fs_names = fs.get_fs_names();

    // Index the file handlers by name
    let mut fs_handler = HashMap::new();

    let mut report_file = OpenOptions::new()
        .create(true)
        .write(true)
        .open(&format!(
            "./test_results/{}.csv",
            h.file_name().unwrap().to_str().unwrap()
        ))
        .unwrap_or_else(|_| {
            panic!(
                "Cannot create ./test_results/{}.csv",
                h.file_name().unwrap().to_str().unwrap()
            )
        });

    let mut case_file = OpenOptions::new()
        .create(true)
        .write(true)
        .open(&format!(
            "./test_results/{}_cases.output",
            h.file_name().unwrap().to_str().unwrap()
        ))
        .unwrap_or_else(|_| {
            panic!(
                "Cannot create ./test_results/{}_cases.output",
                h.file_name().unwrap().to_str().unwrap()
            )
        });

    // Saves the handler of the engine TODO better explaination
    for fs_name in fs_names {
        let reader = read_wasm!(&format!("testsuite/{}", fs_name));
        let module = parse(reader).unwrap();
        let validation = validate(&module);
        let mi = ModuleInstance::new(&module);

        let spectest_import = get_spectest_import();
        let mut e = Engine::new(
            mi,
            &module,
            Box::new(RelativeProgramCounter::default()),
            &spectest_import,
        );

        if let Err(err) = e {
            eprintln!("ERROR: {}", err);
            err.chain()
                .skip(1)
                .for_each(|cause| eprintln!("because: {}", cause));
            std::process::exit(1);
        }

        fs_handler.insert(fs_name, Rc::new(RefCell::new(e.unwrap())));
    }

    let mut current_engine = None;
    let mut named_modules = HashMap::new();

    for case in fs.get_cases() {
        match case {
            // Replace `current_engine` with next WASM module
            Command::Module(m) => {
                // If the module is named,
                // then save it
                if let Some(ref name) = m.name {
                    named_modules.insert(name, fs_handler.get(&m.filename));
                    current_engine = fs_handler.get(&m.filename);
                } else {
                    current_engine = fs_handler.get(&m.filename);
                }
            }
            Command::Action(case) => {
                let mut engine = current_engine
                    .expect("No WASM module was initialized")
                    .borrow_mut();

                let args = case.get_args();

                if let Err(err) =
                    engine.invoke_exported_function_by_name(&case.action.field, args.clone())
                {
                    report_fail(
                        &mut report_file,
                        &mut case_file,
                        &case,
                        p,
                        vec![],
                        ExecutionResult::NotComparable,
                    );
                }

                let expected = case.get_expected();

                debug!("expected {:?}", expected);
                debug!("arg into the engine {:?}", args);
                debug!("store {:?}", engine.store.stack);

                if expected.is_empty() {
                    // Do not report ok, because this
                    // is an action, not a test
                    continue;
                }

                // Get the actual results based on the count how many results we expect
                let actuals: Vec<_> = engine
                    .store
                    .stack
                    .iter()
                    .rev()
                    .take(expected.len())
                    .collect();

                debug!("Returned actuals {:?}", actuals);

                if !actuals.iter().all(|x| x.is_value()) {
                    report_fail(
                        &mut report_file,
                        &mut case_file,
                        &case,
                        p,
                        expected,
                        ExecutionResult::NotComparable,
                    );

                    error!("Executed function did not return a value");

                    continue;
                }
            }
            Command::AssertReturn(case) => {
                total_stats.total_count.fetch_add(1, Ordering::Relaxed);
                case_stats.total_count.fetch_add(1, Ordering::Relaxed);

                let mut engine = match &case.action.module {
                    None => {
                        debug!("Using current engine");
                        current_engine
                            .expect("No WASM module was initialized")
                            .borrow_mut()
                    }
                    Some(name) => {
                        // do not load current_engine, but defined module
                        debug!("Using named module {}", name);
                        named_modules.get(&name).unwrap().unwrap().borrow_mut()
                    }
                };

                let args = case.get_args();

                let expected: Vec<_> = case.get_expected().iter().copied().collect();
                let actuals = match case.action.ty {
                    ActionType::Invoke => {
                        debug!(
                            "Invoking with {} and {:?}",
                            &case.action.field,
                            args.clone()
                        );

                        if let Err(err) = engine
                            .invoke_exported_function_by_name(&case.action.field, args.clone())
                        {
                            debug!("failed for lineno {}", case.line);
                            report_fail(
                                &mut report_file,
                                &mut case_file,
                                &case,
                                p,
                                vec![],
                                ExecutionResult::NotComparable,
                            );
                        }

                        let actuals: Vec<_> = engine
                            .store
                            .stack
                            .iter()
                            .rev()
                            .take(expected.len())
                            //.rev()
                            .collect();

                        actuals.into_iter().cloned().collect()
                    }
                    ActionType::Get => {
                        let res = engine.get(&case.action.field);
                        if let Err(ref err) = res {
                            debug!("failed for lineno {}", case.line);
                            report_fail(
                                &mut report_file,
                                &mut case_file,
                                &case,
                                p,
                                vec![],
                                ExecutionResult::NotComparable,
                            );

                            vec![]
                        } else {
                            vec![StackContent::Value(res.unwrap())]
                        }
                    }
                };

                debug!("expected {:?}", expected);
                debug!("arg into the engine {:?}", args);
                debug!("store {:?}", actuals);

                // If nothing is expected and no error occurred then ok
                if expected.is_empty() {
                    total_stats.reported_ok.fetch_add(1, Ordering::Relaxed);
                    case_stats.reported_ok.fetch_add(1, Ordering::Relaxed);
                    report_ok(&mut report_file, &mut case_file, &case, p, expected);
                    continue;
                }

                debug!("Returned actuals {:?}", actuals);

                // Get the actual results based on the count how many results we expect

                if !actuals.iter().all(|x| x.is_value()) {
                    report_fail(
                        &mut report_file,
                        &mut case_file,
                        &case,
                        p,
                        expected,
                        ExecutionResult::NotComparable,
                    );

                    error!("Executed function did not return a value");

                    continue;
                }

                let actuals: Vec<_> = actuals
                    .into_iter()
                    .filter_map(|w| match w {
                        StackContent::Value(v) => Some(v),
                        _ => None,
                    })
                    .rev()
                    .collect();

                debug!("Actual {:?}", actuals);

                assert_eq!(expected, actuals);

                let mut total_do_match = true;
                for i in 0..actuals.len() {
                    let do_match = match (actuals.get(i), expected.get(i)) {
                        (Some(Value::F32(f1)), Some(Value::F32(f2)))
                            if f1.is_nan() && f2.is_nan() =>
                        {
                            true
                        }
                        (Some(Value::F64(f1)), Some(Value::F64(f2)))
                            if f1.is_nan() && f2.is_nan() =>
                        {
                            true
                        }
                        (Some(f1), Some(f2)) => f1 == f2,
                        (None, None) => true,
                        _ => false,
                    };

                    total_do_match &= do_match;
                }

                if total_do_match {
                    total_stats.reported_ok.fetch_add(1, Ordering::Relaxed);
                    case_stats.reported_ok.fetch_add(1, Ordering::Relaxed);
                    report_ok(&mut report_file, &mut case_file, &case, p, expected);
                } else {
                    report_fail(
                        &mut report_file,
                        &mut case_file,
                        &case,
                        p,
                        expected,
                        ExecutionResult::Values(actuals),
                    );
                }
            }
            _ => {} // skip Rest
        }
    }

    println!(
        "Summary {} total {} where {} ok and {} failed",
        p,
        case_stats.total_count.load(Ordering::Relaxed),
        case_stats.reported_ok.load(Ordering::Relaxed),
        case_stats.total_count.load(Ordering::Relaxed)
            - case_stats.reported_ok.load(Ordering::Relaxed)
    );

    "".to_string()
}

fn report_ok(
    report_file: &mut File,
    case_file: &mut File,
    case: &AssertReturn,
    p: &str,
    expected: Vec<Value>,
) {
    let mut buffer = String::new();

    for i in expected.iter() {
        buffer.push_str(&format!("{:?}", i));
    }

    report_file
        .write_all(format!("{},OK,{}(),{}\n", p, case.action.field, buffer).as_bytes())
        .unwrap();
}

fn draw_args(v: Vec<Value>) -> String {
    let mut buffer = String::new();

    for i in v.iter() {
        buffer.push_str(&format!("{:?}", i));
    }

    buffer
}

enum ExecutionResult {
    Values(Vec<Value>),
    NotComparable,
}

fn report_fail(
    report_file: &mut File,
    case_file: &mut File,
    case: &AssertReturn,
    p: &str,
    expected: Vec<Value>,
    result: ExecutionResult,
) {
    let args = draw_args(case.get_args());
    let expected = draw_args(case.get_expected());

    report_file
        .write_all(format!("{},FAIL,{},{}\n", p, case.action.field, expected).as_bytes())
        .unwrap();

    match result {
        ExecutionResult::Values(result) => {
            case_file.write_all(format!("[FAILED]: {}({}) @ {}\n[FAILED]: Assertion failed!\n[FAILED]: Expected: \t{}\n[FAILED]: Actual:\t{:?}\n\n", case.action.field, args, case.line, expected, result ).as_bytes()).unwrap();
        }

        ExecutionResult::NotComparable => {
            case_file.write_all(format!("[FAILED]: {}({}) @ {}\n[FAILED]: Assertion failed!\n[FAILED]: Expected: \t{}\n[FAILED]: Actual:\t{:?}\n", case.action.field, args, case.line, expected, "not comparable" ).as_bytes()).unwrap();
        }
    }
}

/// The default imports
fn get_spectest_import() -> Imports {
    let mut imports = Imports::new();

    let module = "spectest".to_string();

    imports.push(Import::Global(
        module.clone(),
        "global_i32".to_string(),
        GlobalInstance::immutable(funky::value::Value::I32(666)),
    ));
    imports.push(Import::Global(
        module.clone(),
        "global_i64".to_string(),
        GlobalInstance::immutable(funky::value::Value::I64(666)),
    ));
    imports.push(Import::Global(
        module.clone(),
        "global_f32".to_string(),
        GlobalInstance::immutable(funky::value::Value::F32(666.6)),
    ));
    imports.push(Import::Global(
        module.clone(),
        "global_f64".to_string(),
        GlobalInstance::immutable(funky::value::Value::F64(666.6)),
    ));

    imports
}

/*
fn fold_start() {
    println!("travis_fold:start:$1\033[33;1m$2\033[0m");
}

fn fold_end() {
    println!("\ntravis_fold:end:$1\r");
}
*/
