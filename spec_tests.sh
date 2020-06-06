#!/usr/bin/env bash
set -euo pipefail


echo "[*] Downloading test-suite"

rm -rf testsuite-master
curl -L -o testsuite.zip https://github.com/WebAssembly/testsuite/archive/master.zip

unzip testsuite.zip >/dev/null
rm -rf testsuite.zip

echo "[*] Download finished"

cd testsuite-master;
for f in *.wast; do
    wast2json $f
done;
