import os
import time
import json

b = "./target/release/ifds"
run_times = 1
timeout = 60 * 60

result = open("benchmark_results.csv", "w")
result_size = open("benchmark_size1.csv", "w")
result_meta_sparse = open("meta_sparse.csv", "w")

def report(alg,name, times, mem):
    result.write("{},{},{},{}\n".format(alg, name, ",".join([str(i) for i in times]), mem))

def report_size(name, size):
    result_size.write("{},{}\n".format(name, size))

def report_meta_sparse(name, graph, path, rel_instructions):
    result_meta_sparse.write("{},{},{},{}\n".format(name, graph, path, rel_instructions))


def run(alg, name, fs, func, pc, var, ty):
    print("Running {} ({})".format(name, alg))
    times = []

    cmd = "fast"

    if ty == 0:
        cmd = "naive"
    elif ty == 1:
        cmd = "orig"
    elif ty == 2:
        cmd = "sparse"

    # warmup
    #os.system("timeout {} {} {} {} -f {} -p {} -v {} > /dev/null".format(timeout, b, cmd, fs, func, pc, var))
    for _ in range(run_times):
        start = time.time()
        return_code = os.system("/usr/bin/time -q -o /tmp/mem -f \"%M\" timeout --preserve-status {} {} {} {} -f {} -p {} -v {} > /dev/null 2> /dev/null".format(timeout, b, cmd, fs, func, pc, var))
        end = time.time()
        delta = end - start

        memFile = open("/tmp/mem", "r")
        memUsage = memFile.read()

        #print("Mem usage {}".format(memUsage))
        
        if return_code == 0:
            print("Time {}".format(delta))
            times.append(delta)
        else:
            #print("NA")
            times.append("NA")

        print("=> Run complete")

    report(alg, name, times, memUsage)

def meta_sparse(name, fs, func, pc, var):
    cmd = "sparse"
    return_code = os.system("{} {} {} -f {} -p {} -v {} -m /tmp/meta.out > /dev/null".format(b, cmd, fs, func, pc, var))
    metaFile = open("/tmp/meta.out", "r")
    meta = json.load(metaFile)
    report_meta_sparse(name, meta["estimated_exploded_graph_size"], meta["number_path_edges"], meta["sparse_relevant_instructions"])

def sizes(name, fs):
    cmd = "meta" 
    return_code = os.system("{} {} {} > /tmp/size".format(b, cmd, fs))

    sizeFile = open("/tmp/size", "r")
    size = json.load(sizeFile)
    report_size(name, size["estimated_exploded_graph_size"])

wasm = [{
    "name": "rg.wasm",
    "function": "641",
    "pc": 1,
    "var": "%12"
    },
    {
        "name": "sqlite.wasm",
        "function": "87",
        "pc": 1,
        "var": "%7"
    },
    {
        "name": "wasm3-wasi.wasm",
        "function": "49",
        "pc": 1,
        "var": "%13"
    },
    {
        "name": "d3wasm.wasm",
        "function": "3151",
        "pc": 1,
        "var": "%3"
    },
    {
        "name": "fd.wasm",
        "function": "427",
        "pc": 1,
        "var": "%5"
    },
    {
        "name": "sha256.wasm",
        "function": "3",
        "pc": 1,
        "var": "%65"
    },
    {
        "name": "fib.wasm",
        "function": "0",
        "pc": 3,
        "var": "%1"
    },
    {
        "name": "gcd.wasm",
        "function": "0",
        "pc": 2,
        "var": "%4"
    },
    {
        "name": "fac.wasm",
        "function": "0",
        "pc": 1,
        "var": "%1"
    },
    {
        "name": "blocks.wasm",
        "function": "0",
        "pc": 6,
        "var": "%10" 
    }]

for y in wasm:
    print(y)
    sizes(y["name"], "benchmarks/{}".format(y["name"]))

for x in wasm:
    print(x)
    run("sparse", x["name"], "benchmarks/{}".format(x["name"]), x["function"], x["pc"], x["var"], 2)
    meta_sparse(x["name"], "benchmarks/{}".format(x["name"]), x["function"], x["pc"], x["var"])
    run("fast", x["name"], "benchmarks/{}".format(x["name"]), x["function"], x["pc"], x["var"], -1)
    run("orig", x["name"], "benchmarks/{}".format(x["name"]), x["function"], x["pc"], x["var"], 1)
    run("bfs", x["name"], "benchmarks/{}".format(x["name"]), x["function"], x["pc"], x["var"], 0)

#run("sparse", "rg.wasm", "benchmarks/rg.wasm", "641", 1, "%12", 2)
#run("sparse", "sqlite.wasm", "benchmarks/sqlite.wasm", "87", 1, "%7", 2)
#run("sparse", "wasm3-wasi.wasm", "benchmarks/wasm3-wasi.wasm", "49", 1, "%13", 2)
#run("sparse", "fd.wasm", "benchmarks/fd.wasm", "427", 1, "%5", 2)
#run("sparse", "d3wasm.wasm", "benchmarks/d3wasm.wasm", "3151", 1, "%3", 2)
#run("sparse", "sha256.wasm", "benchmarks/sha256.wasm", "3", 1, "%65", 2)
#
#run("fast", "rg.wasm", "benchmarks/rg.wasm", "641", 1, "%12", -1)
#run("fast", "fd.wasm", "benchmarks/fd.wasm", "427", 1, "%5", -1)
#run("fast", "wasm3-wasi.wasm", "benchmarks/wasm3-wasi.wasm", "49", 1, "%13", -1)
#run("fast", "d3wasm.wasm", "benchmarks/d3wasm.wasm", "3151", 1, "%3", -1)
#run("fast", "sqlite.wasm", "benchmarks/sqlite.wasm", "87", 1, "%7", -1)
#run("fast", "sha256.wasm", "benchmarks/sha256.wasm", "3", 1, "%65", -1)
#
#run("orig", "rg.wasm", "benchmarks/rg.wasm", "641", 1, "%12", 1)
#run("orig", "fd.wasm", "benchmarks/fd.wasm", "427", 1, "%5", 1)
#run("orig", "sqlite.wasm", "benchmarks/sqlite.wasm", "87", 1, "%7", 1)
#run("orig", "wasm3-wasi.wasm", "benchmarks/wasm3-wasi.wasm", "49", 1, "%13", 1)
#run("orig", "d3wasm.wasm", "benchmarks/d3wasm.wasm", "3151", 1, "%3", 1)
#run("orig", "sha256.wasm", "benchmarks/sha256.wasm", "3", 1, "%65", 1)
#
#run("bfs", "rg.wasm", "benchmarks/rg.wasm", "641", 1, "%12", 0)
#run("bfs", "fd.wasm", "benchmarks/fd.wasm", "427", 1, "%5", 0)
#run("bfs", "sqlite.wasm", "benchmarks/sqlite.wasm", "87", 1, "%7", 0)
#run("bfs", "wasm3-wasi.wasm", "benchmarks/wasm3-wasi.wasm", "49", 1, "%13", 0)
#run("bfs", "d3wasm.wasm", "benchmarks/d3wasm.wasm", "3151", 1, "%3", 0)
#run("bfs", "sha256.wasm", "benchmarks/sha256.wasm", "3", 1, "%65", 0)


result.close()
result_size.close()
result_meta_sparse.close()
