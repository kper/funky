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

rm -f report.csv
echo "Path,Status,Case,Args" > report.csv
for f in testsuite/*.json; do
    fold_start "$f" "$f"
    echo "--- Running $f ---"
    if timeout 120 ./run_spec_tests.py "$f"; then
        echo "--- Finished $f ---"
    else
        echo "--- !Timeout during $f! ---"
    fi
    fold_end "$f"
done
