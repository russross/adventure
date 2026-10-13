.NOTPARALLEL:
.DEFAULT_GOAL := all
.DELETE_ON_ERROR:
.PHONY: all format fmt lint check run mostlyclean clean
CARGO ?= cargo
DASM ?= $(abspath ../.tools/dasm)
export CARGO_TARGET_DIR := $(abspath ../target/validator)

Cargo.lock: Cargo.toml ../trace/Cargo.toml ../translation/Cargo.toml
	$(CARGO) generate-lockfile
	touch $@

all: Cargo.lock ../target/validator-rom.bin
	$(CARGO) build --locked --release

$(abspath ../.tools/dasm): ../original/Makefile
	$(MAKE) -C .. .tools/dasm

../target/validator-rom.bin: ../original/adventure-gameplay.asm $(DASM) Makefile
	mkdir -p ../target
	$(DASM) ../original/adventure-gameplay.asm -f3 -o$@
	printf '%s  %s\n' b0326b45e5cab066b2d03f1e683708d82de3ffe60b2e968bf88c5494237f0a96 $@ | sha256sum -c -

format:
	$(CARGO) fmt

fmt:
	$(CARGO) fmt --check

lint: fmt Cargo.lock ../target/validator-rom.bin
	$(CARGO) clippy --locked --all-targets -- -D warnings

check: lint
	$(CARGO) test --locked $(TEST_ARGS)

run: all
	$(CARGO_TARGET_DIR)/release/adventure-validator $(ARGS)

mostlyclean:
	rm -rf $(CARGO_TARGET_DIR)/debug $(CARGO_TARGET_DIR)/release/deps $(CARGO_TARGET_DIR)/release/build $(CARGO_TARGET_DIR)/release/incremental $(CARGO_TARGET_DIR)/release/.fingerprint
	rm -f ../target/validator-rom.lst ../target/validator-rom.sym

clean: mostlyclean
	rm -rf $(CARGO_TARGET_DIR)
	rm -f ../target/validator-rom.bin
