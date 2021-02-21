#![allow(unused)]

#[macro_use]
extern crate funky;

use std::cell::RefCell;
use std::collections::HashMap;
use std::fs::{create_dir, read_dir, read_to_string, remove_file, DirEntry, File, OpenOptions};
use std::io::{Read, Write};
use std::path::Path;
use std::rc::Rc;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::{Arc, Mutex};

use funky::debugger::RelativeProgramCounter;
use funky::engine::import_resolver::{Import, Imports};
use funky::engine::module::ModuleInstance;
use funky::engine::store::GlobalInstance;
use funky::engine::Engine;
use funky::engine::TableInstance;
use funky::value::Value;
use funky::{parse, read_wasm, validate};

use crate::core::*;
use crate::json::*;
use log::{debug, error};
use std::path::PathBuf;
use structopt::StructOpt;

use anyhow::{Context, Result};

mod core;
mod json;

#[derive(Debug, StructOpt)]
#[structopt(name = "testrunner", about = "Runs the official webassembly spectests")]
struct Opt {
    #[structopt(parse(from_os_str))]
    inputs: Vec<PathBuf>,
}

fn main() {
    env_logger::init();
    let opt = Opt::from_args();

    if let Err(err) = work(&opt) {
        eprintln!("ERROR: {}", err);
        err.chain()
            .skip(1)
            .for_each(|cause| eprintln!("because: {}", cause));
        std::process::exit(1);
    }
}

fn work(opt: &Opt) -> Result<()> {
    //TODO implement filtering
    let mut files = get_testfiles().context("Trying to fetch the testfiles")?;

    debug!("=> Detected {} testsuite files", files.len());

    let mut spectests = Vec::new();

    for file in files.iter_mut() {
        debug!("Running testfile {}", file.source_filename);
        let mut statistic = Statistic::new(file.source_filename.clone());

        file.run_cases(&mut statistic)
            .context(format!("Running cases of {} failed", file.source_filename));

        report_spectest(&statistic);

        spectests.push(statistic);
    }

    let total = spectests.iter().map(|x| x.get_successes()).sum::<usize>() as f64
        / spectests.iter().map(|x| x.get_total()).sum::<usize>() as f64;

    println!("{}% total", total);

    // Handle total
    {
        let mut total_fs = File::open(".testrunner");

        if let Ok(mut file) = total_fs {
            let mut content = String::new();
            file.read_to_string(&mut content);

            let old_total = content
                .parse::<f64>()
                .context("Total in .testrunner is not a float")?;

            if total >= old_total {
                debug!("=> Results are better. Replacing old total");
                report_total(total);
            } else {
                eprintln!(
                    "Current testrun has {} success, while the best run had {}",
                    total, old_total
                );
                std::process::exit(-1);
            }
        } else {
            report_total(total)?;
        }
    }

    Ok(())
}

fn report_total(total: f64) -> Result<()> {
    use std::fs::OpenOptions;

    let mut file = OpenOptions::new()
        .read(false)
        .write(true)
        .create(true)
        .open(".testrunner")
        .context("Cannot create .testrunner")?;

    file.write_all(format!("{}\n", total).as_bytes());

    Ok(())
}

fn report_spectest(stat: &Statistic) {
    println!("{:#?}", stat);
}

fn get_testfiles() -> Result<Vec<TestFile>> {
    let files = read_dir("./testsuite")
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
        .collect::<Vec<_>>();

    let testfiles = files
        .into_iter()
        .map(|x| {
            let path = x.path();
            let p = path.to_str().unwrap();
            let mut buffer = read_to_string(p).unwrap();

            let fs: TestFile = serde_json::from_str(&buffer).unwrap();

            fs
        })
        .collect::<Vec<_>>();

    Ok(testfiles)
}
