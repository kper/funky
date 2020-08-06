cargo build --release && perf record -g --call-graph=dwarf ./target/release/funky tests/fib.wasm fib 'I32(30)'
