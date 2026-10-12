# Serialize the assembler's three outputs and the ordered verification workflow.
.NOTPARALLEL:
.DELETE_ON_ERROR:
.PHONY: all gameplay analysis verify lint check rust rust-release rust-check run mostlyclean clean

DASM ?= .tools/dasm
PYTHON ?= python3
UV ?= uv
CARGO ?= cargo

all: build/adventure.bin build/adventure.lst build/adventure.sym gameplay analysis rust

gameplay: build/adventure-gameplay.bin build/adventure-gameplay.lst build/adventure-gameplay.sym

build:
	mkdir -p $@

# Keep the tool local; no global installation or Git operation is necessary.
.tools/dasm: Makefile
	mkdir -p .tools
	curl -fL -o .tools/dasm.tar.gz https://github.com/dasm-assembler/dasm/archive/refs/tags/v2.20.17.tar.gz
	tar -xzf .tools/dasm.tar.gz -C .tools
	$(MAKE) -C .tools/dasm-2.20.17
	cp .tools/dasm-2.20.17/bin/dasm $@

# One invocation emits all artifacts; serialization prevents competing writers.
build/adventure.bin build/adventure.lst build/adventure.sym: adventure.asm $(DASM) Makefile | build
	$(DASM) adventure.asm -f3 -obuild/adventure.bin -lbuild/adventure.lst -sbuild/adventure.sym

build/adventure-gameplay.bin build/adventure-gameplay.lst build/adventure-gameplay.sym: adventure-gameplay.asm $(DASM) Makefile | build
	$(DASM) adventure-gameplay.asm -f3 -obuild/adventure-gameplay.bin -lbuild/adventure-gameplay.lst -sbuild/adventure-gameplay.sym

build/control-flow.json: ADVNTURE.BIN tools/analyze.py Makefile | build
	$(PYTHON) tools/analyze.py > $@.tmp
	mv $@.tmp $@

analysis: build/control-flow.json

lint:
	$(UV) run ruff format --check tools
	$(UV) run ruff check tools
	$(UV) run ty check

verify: all
	cmp ADVNTURE.BIN build/adventure.bin
	cmp ADVNTURE.BIN build/adventure-gameplay.bin
	$(PYTHON) tools/verify.py

# Formatting and static checks finish before behavioral verification starts.
check: lint rust-check
	$(MAKE) verify DASM="$(DASM)" PYTHON="$(PYTHON)"

# Each Rust implementation owns its build directory and dependency tracking.
rust:
	$(MAKE) -C translation all

rust-release:
	$(MAKE) -C translation release

rust-check:
	$(MAKE) -C translation check

run: rust-release
	translation/target/release/adventure $(ARGS)

.PHONY: rust-smoke
rust-smoke:
	$(MAKE) -C translation smoke

mostlyclean:
	$(MAKE) -C translation mostlyclean
	$(MAKE) -C refactored mostlyclean
	rm -f build/adventure.lst build/adventure.sym build/control-flow.json build/control-flow.json.tmp build/data-labels.json
	rm -f build/adventure-gameplay.lst build/adventure-gameplay.sym
	rm -rf tools/__pycache__ .tools/dasm-2.20.17 .tools/dasm.tar.gz
	rm -rf target/debug target/release/deps target/release/build target/release/incremental target/release/.fingerprint target/release/examples
	rm -f target/release/*.d target/.rustc_info.json target/rust-smoke.bmp target/rust-smoke.png

clean: mostlyclean
	$(MAKE) -C translation clean
	rm -rf build target .tools

# The independent refactor shares no runtime code with the preserved translation.
.PHONY: refactored refactored-release refactored-check refactored-run refactored-smoke
refactored:
	$(MAKE) -C refactored all

refactored-release:
	$(MAKE) -C refactored release

refactored-check:
	$(MAKE) -C refactored check

refactored-run:
	$(MAKE) -C refactored run ARGS="$(ARGS)"

refactored-smoke:
	$(MAKE) -C refactored smoke
