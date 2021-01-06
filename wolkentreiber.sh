path="./testsuite/block.0.wasm"

perf stat -e cache-misses,branch-misses,cpu-cycles,instructions,branch-instructions -x \; -o wolkentreiber.csv  cargo run --bin funky -- $path "break-bare" && ./wolkentreiber.py $path

path="./tests/gcd.wasm"

perf stat -e cache-misses,branch-misses,cpu-cycles,instructions,branch-instructions -x \; -o wolkentreiber.csv  cargo run --bin funky -- $path "gcd" "I32(640)" "I32(125483)"  && ./wolkentreiber.py $path

