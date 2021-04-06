use ifds::{ir::ast::Program, solver::*};
use ifds::icfg::convert::ConvertSummary;
use ifds::grammar::*;

use funky::engine::module::ModuleInstance;
use funky::engine::*;
use ifds::ir::wasm_ast::IR;
use validation::validate;
use wasm_parser::{parse, read_wasm};

use criterion::{criterion_group, criterion_main, Criterion};
macro_rules! wasm {
    ($input:expr) => {{
        let file = read_wasm!($input);
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

        engine
    }};
}

fn bench(convert: &mut ConvertSummary, prog: &Program, req: &Request) {
    let mut solver = IfdsSolver;
    let mut graph = convert.visit(&prog, req).unwrap();
    solver.all_sinks(&mut graph, req).unwrap();
}

macro_rules! benchmark {
    ($name:ident) => {
        fn $name(c: &mut Criterion) {
            let engine = wasm!(format!("../tests/{}.wasm", stringify!($name)));
            let mut ir = IR::new();
            ir.visit(&engine).unwrap();

            let mut convert = ConvertSummary::new();
            let prog = ProgramParser::new().parse(&ir.buffer()).unwrap();

            for function in prog.functions.iter() {
                let req = Request {
                    variable: Some("%0".to_string()),
                    function: function.name.clone(),
                    pc: 0,
                };
                c.bench_function(&format!("{}_func_{}", stringify!($name), &function.name), |b| {
                    b.iter(|| bench(&mut convert, &prog, &req))
                });
            }
        }
    };
}

benchmark!(fib);
benchmark!(fac);
benchmark!(logic);
benchmark!(gcd);

criterion_group!(benches, fib, fac, logic, gcd);
criterion_main!(benches);
