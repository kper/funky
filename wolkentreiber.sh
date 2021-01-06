path="./testsuite/block.0.wasm"
num=3
n=0

cargo build --release --bin funky


while [ "$n" -lt $num ]; do
	n=$(( n + 1 ))
	perf stat -e cache-misses,branch-misses,cpu-cycles,instructions,branch-instructions -x \; -o wolkentreiber.csv  target/release/funky $path "break-bare" && ./wolkentreiber.py $path
done

path="./tests/gcd.wasm"
n=0

while [ "$n" -lt $num ]; do
	n=$(( n + 1 ))
	perf stat -e cache-misses,branch-misses,cpu-cycles,instructions,branch-instructions -x \; -o wolkentreiber.csv target/release/funky $path "gcd" "I32(640)" "I32(125483)"  && ./wolkentreiber.py $path
done
