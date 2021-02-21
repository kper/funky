#!/usr/bin/env bash
set -euo pipefail

fold_start() {
  echo -e "travis_fold:start:$1\033[33;1m$2\033[0m"
}

fold_end() {
  echo -e "\ntravis_fold:end:$1\r"
}


rm -rf testsuite
git clone https://github.com/WebAssembly/testsuite.git
cd testsuite
git checkout 0ef5db9f1914b930e4bff34dc7d425ac259b798a
cd ..

echo "[*] Download finished"

cd testsuite
for f in *.wast; do
    wast2json --no-check "$f"
done
cd ..

RUST_MIN_STACK=8388608 cargo run --release --bin test_runner2
