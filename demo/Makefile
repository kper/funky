FUNKY_BIN=../target/debug/funky
.SUFFIXES:
all: run

demo.zig.wasm: demo.zig
	zig build-lib demo.zig -target wasm32-freestanding --name demo.zig --disable-gen-h

demo.c.o: demo.c
	clang -c --target=wasm32 -ffunction-sections -nostdlib -flto demo.c -o demo.c.o

demo.c.wasm: demo.c.o
	wasm-ld demo.c.o -o demo.c.wasm --no-entry --export-dynamic

run: demo.zig.wasm demo.c.wasm
	@echo "--- clang ---"
	@echo "Sum all natural numbers from 0..20"
	$(FUNKY_BIN) demo.c.wasm demo_sum 'I32(21)'
	@echo "--- ZIG ---"
	@echo "Showcase Zigs outer loop syntax"
	$(FUNKY_BIN) demo.zig.wasm labeled_for
	@echo "Sum up char values of 'hello' up to index 3"
	$(FUNKY_BIN) demo.zig.wasm sum_letters 'I32(3)'

clean:
	- rm -f demo.zig.wasm demo.zig.o
	- rm -f demo.c.wasm demo.c.o

.PHONY: clean run
