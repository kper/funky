import os
import time

b = "./target/release/ifds"
run_times = 3

result = open("benchmark_results.csv", "w")

def report(name, times):
    result.write("{},{}\n".format(name, ",".join([str(i) for i in times])))

def run(name, fs, func, pc, var, ty):
    times = []

    cmd = "run"

    if ty == 0:
        cmd = "naive"
    elif ty == 1:
        cmd = "orig"

    # warmup
    os.system("{} {} {} -f {} -p {} -v {} > /dev/null".format(b, cmd, fs, func, pc, var))
    for _ in range(run_times):
        start = time.time()
        os.system("{} {} {} -f {} -p {} -v {} > /dev/null".format(b, cmd, fs, func, pc, var))
        end = time.time()
        delta = end - start
        print("Time {}".format(delta))
        times.append(delta)
    report(name, times)

run("fast blocks.wasm", "../tests/blocks.wasm", "0", 8, "%10", -1)
run("fast sha256.wasm", "../tests/sha256.wasm", "3", 1, "%65", -1)

run("orig sha256.wasm", "../tests/sha256.wasm", "3", 1, "%65", 1)

#run("bfs blocks.wasm", "../tests/blocks.wasm", "0", 8, "%10", True)
run("bfs sha256.wasm", "../tests/sha256.wasm", "3", 1, "%65", 0)


result.close()