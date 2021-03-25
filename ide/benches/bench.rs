use ide::solver::bfs::*;
use ide::solver::*;

use ide::icfg::convert::Convert;
use ide::icfg::graph2::Graph;

use ide::grammar::*;

use funky::engine::module::ModuleInstance;
use funky::engine::*;
use ide::ir::wasm_ast::IR;
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

fn bench_fib<T: GraphReachability>(solver: &mut IfdsSolver<T>, graph: &mut Graph, req: &Request) {
    solver.all_sinks(graph, req);
}

macro_rules! benchmark {
    ($name:ident) => {
        fn $name(c: &mut Criterion) {
            let engine = wasm!(format!("../tests/{}.wasm", stringify!($name)));
            let mut ir = IR::new();
            ir.visit(&engine).unwrap();

            let mut convert = Convert::new();
            let prog = ProgramParser::new().parse(&ir.buffer()).unwrap();
            let mut graph = convert.visit(&prog).unwrap();
            let mut solver = IfdsSolver::new(BFS);

            let req = Request {
                variable: "%0".to_string(),
                function: "0".to_string(),
                pc: 1,
            };
            c.bench_function(stringify!($name), |b| {
                b.iter(|| bench_fib(&mut solver, &mut graph, &req))
            });
        }
    };
}

benchmark!(fib);
benchmark!(fac);
benchmark!(blocks);

criterion_group!(benches, fib, fac, blocks);
criterion_main!(benches);
