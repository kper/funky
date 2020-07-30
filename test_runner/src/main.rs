#![allow(unused)]

//extern crate serde_json;

#[macro_use]
extern crate funky;

use std::fs::{create_dir, read_dir, read_to_string, remove_file, DirEntry, File, OpenOptions};
use std::io::Write;
use std::path::Path;
use std::process::Command;
use std::sync::atomic::AtomicUsize;
use std::sync::{Arc, Mutex};

use funky::engine::{Engine, ModuleInstance, StackContent, Value};
use funky::{parse, read_wasm, validate};

use env_logger;
use log::debug;

use json::*;

mod json;

macro_rules! remove_test_results_with_ending {
    ($ending:expr) => {
        read_dir("./test_results")
            .unwrap()
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

    assert!(args.len() <= 2); //only two arguments allowed

    let paths = match args.get(1) {
        // Get all files with .json
        Some(test_file) => read_dir("./testsuite")
            .expect("Cannot read ./testsuite")
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
                    .clone()
                    .unwrap()
                    .split(".")
                    .last()
                    .unwrap()
                    == "json"
            })
            .map(|w| w.unwrap())
            .collect::<Vec<_>>(),
    };

    let mut handlers = Vec::new();
    let stdouts = Arc::new(Mutex::new(Vec::new()));
    let length = paths.len();
    let counter = Arc::new(AtomicUsize::new(0));

    for path in paths {
        let st = stdouts.clone();
        let counter = counter.clone();
        let handler = std::thread::spawn(move || {
            println!("--- Running {} ---", path.file_name().to_str().unwrap());

            let stdout = run_spec_test(&path);

            st.lock().unwrap().push(stdout);

            counter.fetch_add(1, std::sync::atomic::Ordering::Relaxed);

            let c = counter.load(std::sync::atomic::Ordering::Relaxed);
            println!("Finished {:.2}%", c as f32 / length as f32 * 100.0);
        });
        handlers.push(handler);
    }

    for h in handlers {
        if let Err(_) = h.join() {
            eprintln!("Error appeared");
            //println!("{}", stdouts.clone().lock().unwrap().join("\n"));
        }
    }

    println!("{}", stdouts.clone().lock().unwrap().join("\n"));
}

fn run_spec_test(path: &DirEntry) -> String {
    //let fs = File::open(path.path().to_str().unwrap()).unwrap();
    let h = path.path();
    let p = h.to_str().unwrap();
    let mut buffer = read_to_string(p).unwrap();

    let fs: TestFile = serde_json::from_str(&buffer).unwrap();
    let count = fs.get_len_cases();

    let fs_name = fs.get_fs_name();

    let reader = read_wasm!(&format!("testsuite/{}", fs_name));
    let module = parse(reader).unwrap();
    let validation = validate(&module);
    let mi = ModuleInstance::new(&module);

    let mut e = Engine::new(mi, &module);
    e.instantiation(&module);

    let mut report_file = OpenOptions::new()
        .create(true)
        .write(true)
        .open(&format!(
            "./test_results/{}.csv",
            h.file_name().unwrap().to_str().unwrap()
        ))
        .expect(&format!(
            "Cannot create ./test_results/{}.csv",
            h.file_name().unwrap().to_str().unwrap()
        ));

    let mut case_file = OpenOptions::new()
        .create(true)
        .write(true)
        .open(&format!(
            "./test_results/{}_cases.output",
            h.file_name().unwrap().to_str().unwrap()
        ))
        .expect(&format!(
            "Cannot create ./test_results/{}_cases.output",
            h.file_name().unwrap().to_str().unwrap()
        ));

    for case in fs.get_cases() {
        let args = case.get_args();
        e.invoke_exported_function_by_name(&case.action.field, args);

        let result = e.store.stack.last();
        let expected = case.get_expected();

        let r2 = match result {
            Some(StackContent::Value(v)) => v,
            _ => panic!("Did not get a value"),
        };

        let do_match = match expected.get(0) {
            Some(r1) => r1 == r2,
            None => result.is_none(),
        };

        if do_match {
            report_ok(&mut report_file, &mut case_file, case, p, expected);
        } else {
            report_fail(&mut report_file, &mut case_file, case, p, expected, r2);
        }
    }

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
        .write_all(format!("{},OK,{},{}", p, case.action.field, buffer).as_bytes())
        .unwrap();
}

fn draw_args(v: Vec<Value>) -> String {
    let mut buffer = String::new();

    for i in v.iter() {
        buffer.push_str(&format!("{:?}", i));
    }

    buffer
}

fn report_fail(
    report_file: &mut File,
    case_file: &mut File,
    case: &AssertReturn,
    p: &str,
    expected: Vec<Value>,
    result: &Value,
) {
    let args = draw_args(case.get_args());
    let expected = draw_args(case.get_expected());

    report_file
        .write_all(format!("{},FAIL,{},{}", p, case.action.field, expected).as_bytes())
        .unwrap();

    case_file.write_all(format!("[FAILED]: {}({}) @ {}\n[FAILED]: Assertion failed!\n[FAILED]: Expected: \t{}\n[FAILED]: Actual:\t{:?}", case.action.field, args, case.line, expected, result ).as_bytes()).unwrap();
}

/*
fn fold_start() {
    println!("travis_fold:start:$1\033[33;1m$2\033[0m");
}

fn fold_end() {
    println!("\ntravis_fold:end:$1\r");
}
*/
