perf record -g --call-graph=dwarf ./target/release/funky fib.wasm fib 'I32(30)'
