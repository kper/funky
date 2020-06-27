# Funky [![Build Status](https://travis-ci.org/kper/funky.svg?branch=master)](https://travis-ci.org/kper/funky)

Funky is a wasm interpreter. This project is an assignment for the Lehrveranstaltung Abstrakte Maschinen. It will not support module-loading and wat files.

Focus of this implementation is not performance but the general ability to execute arbitrary wasm code.

You will find the parser for parsing the binary wasm files in the folder `wasm-parser`. The logic for the runtime and execution will be in the `src` folder. The code for validating the AST is in the `validation` folder.

## Usage

```
./funky <input> <function> [<args>...] [--stage0 | --stage1] [--spec]
./funky (-h | --help)
./funky --version
```
