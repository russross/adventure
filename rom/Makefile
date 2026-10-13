# Assembly outputs form groups; the analyzer never generates assembly source.
ifeq ($(filter grouped-target,$(.FEATURES)),)
$(error ROM builds require GNU Make 4.3 or newer; use gmake -C rom)
endif
.NOTPARALLEL:
.DELETE_ON_ERROR:
.PHONY: all gameplay analysis verify lint check mostlyclean clean
DASM ?= $(abspath tools/dasm)
PYTHON ?= python3
UV ?= uv

all: build/adventure.bin build/adventure.lst build/adventure.sym gameplay analysis

gameplay: build/adventure-gameplay.bin build/adventure-gameplay.lst build/adventure-gameplay.sym

build:
	mkdir -p $@

# The pinned assembler is built locally and remains available after mostlyclean.
$(abspath tools/dasm): Makefile
	mkdir -p tools
	curl -fL -o tools/dasm.tar.gz https://github.com/dasm-assembler/dasm/archive/refs/tags/v2.20.17.tar.gz
	tar -xzf tools/dasm.tar.gz -C tools
	$(MAKE) -C tools/dasm-2.20.17
	cp tools/dasm-2.20.17/bin/dasm $@

# Grouped targets rebuild all peers when a requested output is missing or stale.
build/adventure.bin build/adventure.lst build/adventure.sym &: adventure.asm $(DASM) Makefile | build
	$(DASM) adventure.asm -f3 -obuild/adventure.bin -lbuild/adventure.lst -sbuild/adventure.sym

build/adventure-gameplay.bin build/adventure-gameplay.lst build/adventure-gameplay.sym &: adventure-gameplay.asm $(DASM) Makefile | build
	$(DASM) adventure-gameplay.asm -f3 -obuild/adventure-gameplay.bin -lbuild/adventure-gameplay.lst -sbuild/adventure-gameplay.sym

build/control-flow.json: adventure.bin tools/analyze.py Makefile | build
	$(PYTHON) tools/analyze.py > $@.tmp
	mv $@.tmp $@

analysis: build/control-flow.json

lint:
	$(UV) sync --locked
	$(UV) run --locked ruff format --check tools/analyze.py tools/verify.py
	$(UV) run --locked ruff check tools/analyze.py tools/verify.py
	$(UV) run --locked ty check

verify: all
	cmp adventure.bin build/adventure.bin
	cmp adventure.bin build/adventure-gameplay.bin
	$(PYTHON) tools/verify.py

check: lint verify

mostlyclean:
	rm -f build/adventure.lst build/adventure.sym build/adventure-gameplay.lst build/adventure-gameplay.sym
	rm -f build/control-flow.json build/control-flow.json.tmp build/data-labels.json
	rm -rf tools/__pycache__ tools/dasm-2.20.17 tools/dasm.tar.gz

clean: mostlyclean
	rm -rf build
	rm -f tools/dasm
