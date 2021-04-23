use ifds::grammar::*;
use ifds::icfg::convert::FastConvert;
use ifds::icfg::naive::convert::Convert;
use ifds::icfg::orig::convert::OriginalConvert;
use ifds::{ir::ast::Program, solver::bfs::*, solver::*};

use funky::engine::module::ModuleInstance;
use funky::engine::*;
use ifds::ir::wasm_ast::IR;
use validation::validate;
use wasm_parser::{parse, read_wasm};

use ifds::icfg::flowfuncs::taint::flow::TaintNormalFlowFunction;
use ifds::icfg::flowfuncs::taint::initial::TaintInitialFlowFunction;

use criterion::*;

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

fn bench(
    convert: &mut FastConvert<TaintInitialFlowFunction, TaintNormalFlowFunction>,
    prog: &Program,
    req: &Request,
) {
    let mut solver = IfdsSolver;
    let (mut graph, _state) = convert.visit(&prog, req).unwrap();
    solver.all_sinks(&mut graph, req).unwrap();
}

fn bench_naive(convert: &mut Convert, prog: &Program, req: &Request) {
    let mut solver = Bfs;
    let (mut graph, state) = convert.visit(&prog).unwrap();

    solver.all_sinks(&mut graph, &state, &req);
}

fn bench_orig(
    convert: &mut OriginalConvert,
    prog: &Program,
    req: &Request,
) {
    let mut solver = IfdsSolver;
    let (mut graph, _state) = convert.visit(&prog, req).unwrap();
    solver.all_sinks(&mut graph, req).unwrap();
}

fn bench_fib(c: &mut Criterion) {
    let name = "fib";
    let mut group = c.benchmark_group("Fibonacci");
    group.sampling_mode(SamplingMode::Flat);

    let engine = wasm!(format!("../tests/{}.wasm", name));
    let mut ir = IR::new();
    ir.visit(&engine).unwrap();

    let prog = ProgramParser::new().parse(&ir.buffer()).unwrap();

    let mut convert = Convert::default();

    // Naive
    for function in prog.functions.iter() {
        let req = Request {
            variable: Some("%0".to_string()),
            function: function.name.clone(),
            pc: 0,
        };
        group.bench_function(
            &format!("{}_func_{}", name, &function.name),
            |b| b.iter(|| bench_naive(&mut convert, &prog, &req)),
        );
    }

    let mut convert_fast = FastConvert::new(TaintInitialFlowFunction, TaintNormalFlowFunction);

    for function in prog.functions.iter() {
        let req = Request {
            variable: Some("%0".to_string()),
            function: function.name.clone(),
            pc: 0,
        };

        group.bench_function(&format!("{}_func_{}_fast", name, &function.name), |b| {
            b.iter(|| bench(&mut convert_fast, &prog, &req))
        });
    }

    let mut orig_convert = OriginalConvert::default();

    for function in prog.functions.iter() {
        let req = Request {
            variable: Some("%0".to_string()),
            function: function.name.clone(),
            pc: 0,
        };

        group.bench_function(&format!("{}_func_{}_orig", name, &function.name), |b| {
            b.iter(|| bench_orig(&mut orig_convert, &prog, &req))
        });
    }

    group.finish();
}


fn bench_fac(c: &mut Criterion) {
    let name = "fac";
    let mut group = c.benchmark_group("Fac");
    group.sampling_mode(SamplingMode::Flat);

    let engine = wasm!(format!("../tests/{}.wasm", name));
    let mut ir = IR::new();
    ir.visit(&engine).unwrap();

    let prog = ProgramParser::new().parse(&ir.buffer()).unwrap();

    let mut convert = Convert::default();

    // Naive
    for function in prog.functions.iter() {
        let req = Request {
            variable: Some("%0".to_string()),
            function: function.name.clone(),
            pc: 0,
        };
        group.bench_function(
            &format!("{}_func_{}", name, &function.name),
            |b| b.iter(|| bench_naive(&mut convert, &prog, &req)),
        );
    }

    let mut convert_fast = FastConvert::new(TaintInitialFlowFunction, TaintNormalFlowFunction);

    for function in prog.functions.iter() {
        let req = Request {
            variable: Some("%0".to_string()),
            function: function.name.clone(),
            pc: 0,
        };

        group.bench_function(&format!("{}_func_{}_fast", name, &function.name), |b| {
            b.iter(|| bench(&mut convert_fast, &prog, &req))
        });
    }

    let mut orig_convert = OriginalConvert::default();

    for function in prog.functions.iter() {
        let req = Request {
            variable: Some("%0".to_string()),
            function: function.name.clone(),
            pc: 0,
        };

        group.bench_function(&format!("{}_func_{}_orig", name, &function.name), |b| {
            b.iter(|| bench_orig(&mut orig_convert, &prog, &req))
        });
    }

    group.finish();
}

fn bench_logic(c: &mut Criterion) {
    let name = "logic";
    let mut group = c.benchmark_group("Logic");
    group.sampling_mode(SamplingMode::Flat);

    let engine = wasm!(format!("../tests/{}.wasm", name));
    let mut ir = IR::new();
    ir.visit(&engine).unwrap();

    let prog = ProgramParser::new().parse(&ir.buffer()).unwrap();

    let mut convert = Convert::default();

    // Naive
    for function in prog.functions.iter() {
        let req = Request {
            variable: Some("%0".to_string()),
            function: function.name.clone(),
            pc: 0,
        };
        group.bench_function(
            &format!("{}_func_{}", name, &function.name),
            |b| b.iter(|| bench_naive(&mut convert, &prog, &req)),
        );
    }

    let mut convert_fast = FastConvert::new(TaintInitialFlowFunction, TaintNormalFlowFunction);

    for function in prog.functions.iter() {
        let req = Request {
            variable: Some("%0".to_string()),
            function: function.name.clone(),
            pc: 0,
        };

        group.bench_function(&format!("{}_func_{}_fast", name, &function.name), |b| {
            b.iter(|| bench(&mut convert_fast, &prog, &req))
        });
    }

    let mut orig_convert = OriginalConvert::default();

    for function in prog.functions.iter() {
        let req = Request {
            variable: Some("%0".to_string()),
            function: function.name.clone(),
            pc: 0,
        };

        group.bench_function(&format!("{}_func_{}_orig", name, &function.name), |b| {
            b.iter(|| bench_orig(&mut orig_convert, &prog, &req))
        });
    }

    group.finish();
}

fn bench_gcd(c: &mut Criterion) {
    let name = "gcd";
    let mut group = c.benchmark_group("Greatest Common Divisor");
    group.sampling_mode(SamplingMode::Flat);

    let engine = wasm!(format!("../tests/{}.wasm", name));
    let mut ir = IR::new();
    ir.visit(&engine).unwrap();

    let prog = ProgramParser::new().parse(&ir.buffer()).unwrap();

    let mut convert = Convert::default();

    // Naive
    for function in prog.functions.iter() {
        let req = Request {
            variable: Some("%0".to_string()),
            function: function.name.clone(),
            pc: 0,
        };
        group.bench_function(
            &format!("{}_func_{}", name, &function.name),
            |b| b.iter(|| bench_naive(&mut convert, &prog, &req)),
        );
    }

    let mut convert_fast = FastConvert::new(TaintInitialFlowFunction, TaintNormalFlowFunction);

    for function in prog.functions.iter() {
        let req = Request {
            variable: Some("%0".to_string()),
            function: function.name.clone(),
            pc: 0,
        };

        group.bench_function(&format!("{}_func_{}_fast", name, &function.name), |b| {
            b.iter(|| bench(&mut convert_fast, &prog, &req))
        });
    }

    let mut orig_convert = OriginalConvert::default();

    for function in prog.functions.iter() {
        let req = Request {
            variable: Some("%0".to_string()),
            function: function.name.clone(),
            pc: 0,
        };

        group.bench_function(&format!("{}_func_{}_orig", name, &function.name), |b| {
            b.iter(|| bench_orig(&mut orig_convert, &prog, &req))
        });
    }

    group.finish();
}

criterion_group!(benches, bench_fib, bench_fac, bench_logic, bench_gcd);
criterion_main!(benches);
