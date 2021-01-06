path="./testsuite/block.0.wasm"
func="break-bare"
num=3
n=0

cargo build --release --bin funky

while [ "$n" -lt $num ]; do
	n=$(( n + 1 ))
	perf stat -e cache-misses,branch-misses,cpu-cycles,instructions,branch-instructions -x \; -o wolkentreiber.csv  target/release/funky $path $func && ./wolkentreiber.py $path $func
done

path="./tests/gcd.wasm"
func="gcd"
n=0

while [ "$n" -lt $num ]; do
	n=$(( n + 1 ))
	perf stat -e cache-misses,branch-misses,cpu-cycles,instructions,branch-instructions -x \; -o wolkentreiber.csv target/release/funky  $path $func "I32(640)" "I32(125483)" && ./wolkentreiber.py $path $func
done
