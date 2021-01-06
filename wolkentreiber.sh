perf stat -e cache-misses,branch-misses,cpu-cycles,instructions,branch-instructions -x \; -o wolkentreiber.csv  cargo run --bin funky -- ./testsuite/block.0.wasm "break-bare"
