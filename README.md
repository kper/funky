# Funky [![Build Status](https://travis-ci.org/kper/funky.svg?branch=master)](https://travis-ci.org/kper/funky)

Funky is a wasm interpreter. This project is an assignment for the Lehrveranstaltung Abstrakte Maschinen. It will not support module-loading and wat files.

Focus of this implementation is not performance but the general ability to execute arbitrary wasm code.

You will find the parser for parsing the binary wasm files in the folder `wasm-parser`. The logic for the runtime and execution will be in the `src` folder. The code for validating the AST is in the `validation` folder.

In addition, I am working on static taint analysis for webassembly. This means you can see which variables have the same value, statically.

## Building

Due to a error in cargo, it is not possible to build the taint checker from the root directory. Please go the folder `idfs` and execute `cargo build` again.

## Usage

```
./funky <input> <function> [<args>...] [--stage0 | --stage1] [--spec]
./funky (-h | --help)
./funky --version
```

## Current core spec coverage

~~~
                     File	Ok	Fail	%
         address.json.csv	54	00	100.0%
           align.json.csv	47	00	100.0%
   binary-leb128.json.csv	00	00	0.0%
          binary.json.csv	00	00	0.0%
           block.json.csv	44	00	100.0%
              br.json.csv	76	00	100.0%
           br_if.json.csv	88	00	100.0%
        br_table.json.csv	146	00	100.0%
      break-drop.json.csv	03	00	100.0%
            call.json.csv	69	00	100.0%
   call_indirect.json.csv	107	00	100.0%
        comments.json.csv	00	00	0.0%
           const.json.csv	300	00	100.0%
     conversions.json.csv	522	00	100.0%
          custom.json.csv	00	00	0.0%
            data.json.csv	00	00	0.0%
            elem.json.csv	00	00	0.0%
      endianness.json.csv	68	00	100.0%
         exports.json.csv	02	00	100.0%
             f32.json.csv	2500	00	100.0%
     f32_bitwise.json.csv	360	00	100.0%
         f32_cmp.json.csv	2400	00	100.0%
             f64.json.csv	2500	00	100.0%
     f64_bitwise.json.csv	360	00	100.0%
         f64_cmp.json.csv	2400	00	100.0%
             fac.json.csv	05	02	71.4%
     float_exprs.json.csv	776	18	97.7%
  float_literals.json.csv	83	00	100.0%
    float_memory.json.csv	36	24	60.0%
      float_misc.json.csv	440	00	100.0%
         forward.json.csv	04	00	100.0%
            func.json.csv	93	00	100.0%
       func_ptrs.json.csv	00	00	0.0%
          global.json.csv	45	00	100.0%
         globals.json.csv	45	00	100.0%
             i32.json.csv	364	00	100.0%
             i64.json.csv	374	00	100.0%
              if.json.csv	93	00	100.0%
         imports.json.csv	00	00	0.0%
   inline-module.json.csv	00	00	0.0%
       int_exprs.json.csv	75	00	100.0%
    int_literals.json.csv	30	00	100.0%
          labels.json.csv	25	00	100.0%
   left-to-right.json.csv	95	00	100.0%
         linking.json.csv	00	00	0.0%
            load.json.csv	37	00	100.0%
       local_get.json.csv	19	00	100.0%
       local_set.json.csv	19	00	100.0%
       local_tee.json.csv	55	00	100.0%
            loop.json.csv	41	00	100.0%
          memory.json.csv	45	00	100.0%
     memory_grow.json.csv	30	00	100.0%
memory_redundancy.json.csv	03	01	75.0%
     memory_size.json.csv	36	00	100.0%
     memory_trap.json.csv	05	00	100.0%
           names.json.csv	00	00	0.0%
             nop.json.csv	83	00	100.0%
          return.json.csv	63	00	100.0%
          select.json.csv	88	00	100.0%
skip-stack-guard-page.json.csv	00	00	0.0%
           stack.json.csv	05	00	100.0%
           start.json.csv	00	00	0.0%
           store.json.csv	09	00	100.0%
          switch.json.csv	26	00	100.0%
           table.json.csv	00	00	0.0%
           token.json.csv	00	00	0.0%
           traps.json.csv	00	00	0.0%
            type.json.csv	00	00	0.0%
       typecheck.json.csv	00	00	0.0%
     unreachable.json.csv	05	00	100.0%
unreached-invalid.json.csv	00	00	0.0%
          unwind.json.csv	41	00	100.0%
utf8-custom-section-id.json.csv	00	00	0.0%
utf8-import-field.json.csv	00	00	0.0%
utf8-import-module.json.csv	00	00	0.0%
utf8-invalid-encoding.json.csv	00	00	0.0%
~~~

## Run it

```
cargo run --bin funky -- ./testsuite/block.0.wasm "break-bare"
```

## Run the debugger

```
cargo run --bin hustensaft -- ./testsuite/block.0.wasm "break-bare"
```

## Wait, there is more

You will find a custom debugger in `debugger`.
