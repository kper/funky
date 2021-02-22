use crate::json::{ActionType, AssertReturn, Command, FailedCommand};
use anyhow::{Context, Result};
use funky::debugger::RelativeProgramCounter;
use funky::engine::import_resolver::{Import, Imports};
use funky::engine::module::ModuleInstance;
use funky::engine::stack::StackContent;
use funky::engine::store::GlobalInstance;
use funky::engine::Engine;
use funky::engine::TableInstance;
use funky::value::Value;
use funky::{parse, validate};
use log::debug;
use serde::Deserialize;
use std::collections::HashMap;
use std::rc::Rc;
use std::{
    borrow::BorrowMut,
    cell::{RefCell, RefMut},
};

#[derive(Debug, Clone, Deserialize, PartialEq)]
pub(crate) struct TestFile {
    pub source_filename: String,
    commands: Vec<Command>,
}

pub(crate) type SpecStatistic = Vec<Statistic>;

#[derive(Debug, Default, Clone, PartialEq)]
pub(crate) struct Statistic {
    file: String,
    failed: usize,
    succeeded: usize,
    skipped: usize,
    total: usize,
    failed_case: Option<FailedCommand>,
}

impl Statistic {
    pub fn new(name: String) -> Self {
        Self {
            file: name,
            ..Default::default()
        }
    }

    pub fn register(&mut self, count: usize) {
        self.total = count;
    }

    pub fn success(&mut self) {
        self.succeeded += 1;
    }

    pub fn failed(&mut self, actuals: Vec<Value>, case: Command) {
        self.failed += 1;
        self.skipped = self.total - self.succeeded;
        self.failed_case = Some(FailedCommand::new(actuals, case));
    }

    pub fn get_successes(&self) -> usize {
        self.succeeded
    }

    pub fn get_total(&self) -> usize {
        self.total
    }
}

impl TestFile {
    pub fn get_len_cases(&self) -> usize {
        self.commands
            .iter()
            .filter_map(|x| match x {
                Command::AssertReturn(w) => Some(w),
                _ => None,
            })
            .count()
    }

    pub fn get_cases(&self) -> impl Iterator<Item = &Command> {
        self.commands.iter().filter(
            |x| matches!(x, Command::Module(_) | Command::AssertReturn(_) | Command::Action(_)),
        )
    }

    pub fn get_fs_names(&self) -> Vec<&String> {
        self.commands
            .iter()
            .filter_map(|x| match x {
                Command::Module(w) => Some(&w.filename),
                _ => None,
            })
            .collect::<Vec<_>>()
    }

    pub fn setup(&mut self, name: &str) -> Result<Engine> {
        let reader = self.read_wasm(&format!("testsuite/{}", name))?;
        let module = parse(reader).context(format!("Parsing failed for {}", name))?;
        let validation = validate(&module); //TODO check validation
        let mi = ModuleInstance::new(&module);

        let spectest_import = self.get_spectest_import();

        Engine::new(
            mi,
            &module,
            Box::new(RelativeProgramCounter::default()),
            &spectest_import,
        )
    }

    fn read_wasm(&self, fs: &str) -> Result<Vec<u8>> {
        use std::fs::File;
        use std::io::prelude::*;

        let mut file = File::open(fs).context(format!("Cannot open {}", fs))?;
        let mut reader = Vec::new();

        file.read_to_end(&mut reader)
            .context(format!("Cannot read to end {}", fs))?;

        Ok(reader)
    }

    pub fn run_cases(&mut self, statistic: &mut Statistic) -> Result<()> {
        let mut named_modules = HashMap::new();

        let cases = self.commands.clone();

        statistic.register(cases.len());

        let mut counter = 0;

        for case in cases {
            match case {
                Command::Module(m) => {
                    if let Some(ref name) = m.name {
                        let engine = self.setup(name)?;
                        named_modules.insert(name.clone(), engine);
                    } else {
                        // Save unnamed modules as "last"
                        let mut engine = self.setup(&m.filename)?;
                        named_modules.remove("last"); // Delete old value
                        named_modules.insert("last".to_string(), engine);
                    }

                    statistic.success();
                }
                Command::AssertReturn(ref x) => {
                    let mut engine;

                    if let Some(ref name) = x.action.module {
                        engine = named_modules.get_mut(name).unwrap();
                    } else {
                        engine = named_modules.get_mut("last").unwrap();
                    }

                    let mut actuals = Vec::new();
                    let result = self.run_assert_return(&mut engine, x, &mut actuals);

                    match result {
                        true => statistic.success(),
                        false => statistic.failed(actuals, case),
                    }
                }
                _ => {}
            }
        }

        Ok(())
    }

    fn run_assert_return(
        &self,
        engine: &mut Engine,
        case: &AssertReturn,
        actuals: &mut Vec<Value>,
    ) -> bool {
        let expected: Vec<_> = case.get_expected().iter().copied().collect();
        let args = case.get_args();

        let mut extracted_actuals = match case.action.ty {
            ActionType::Invoke => {
                debug!(
                    "Invoking with {} and {:?}",
                    &case.action.field,
                    args
                );

                if let Err(err) =
                    engine.invoke_exported_function_by_name(&case.action.field, args)
                {
                    debug!("failed for lineno {}", case.line);
                    return false;
                }

                let mut actuals = Vec::new();

                while let Some(StackContent::Value(val)) = engine.store.stack.pop() {
                    actuals.push(val);
                }

                actuals.into_iter().rev().collect()
            }
            ActionType::Get => {
                let res = engine.get(&case.action.field);
                if let Err(ref err) = res {
                    debug!("failed for lineno {}", case.line);
                    return false;
                } else {
                    vec![res.unwrap()]
                }
            }
        };

        //actuals.append(&mut extracted_actuals.into_iter().rev().collect());

        let mut total_do_match = true;
        for i in 0..actuals.len() {
            let do_match = match (actuals.get(i), expected.get(i)) {
                (Some(Value::F32(f1)), Some(Value::F32(f2))) if f1.is_nan() && f2.is_nan() => true,
                (Some(Value::F64(f1)), Some(Value::F64(f2))) if f1.is_nan() && f2.is_nan() => true,
                (Some(f1), Some(f2)) => f1 == f2,
                (None, None) => true,
                _ => false,
            };

            total_do_match &= do_match;
        }

        total_do_match
    }

    /// The default imports
    fn get_spectest_import(&self) -> Imports {
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
        imports.push(Import::Table(
            module,
            "table".to_string(),
            TableInstance::new(10, Some(20)),
        ));

        imports
    }
}
