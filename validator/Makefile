.NOTPARALLEL:
.DELETE_ON_ERROR:
.PHONY: all format fmt lint check run mostlyclean clean
CARGO ?= cargo
export CARGO_TARGET_DIR := $(abspath ../target/validator)

all: ../target/validator-rom.bin
	$(CARGO) build --release

../.tools/dasm:
	$(MAKE) -C .. .tools/dasm

../target/validator-rom.bin: ../original/adventure-gameplay.asm ../.tools/dasm Makefile
	mkdir -p ../target
	../.tools/dasm ../original/adventure-gameplay.asm -f3 -o$@ -l../target/validator-rom.lst -s../target/validator-rom.sym
	printf '%s  %s\n' b0326b45e5cab066b2d03f1e683708d82de3ffe60b2e968bf88c5494237f0a96 $@ | sha256sum -c -

format:
	$(CARGO) fmt

fmt:
	$(CARGO) fmt --check

lint: fmt ../target/validator-rom.bin
	$(CARGO) clippy --all-targets -- -D warnings

check: lint
	$(CARGO) test $(TEST_ARGS)

run: all
	$(CARGO_TARGET_DIR)/release/adventure-validator $(ARGS)

mostlyclean:
	rm -rf $(CARGO_TARGET_DIR)/debug $(CARGO_TARGET_DIR)/release/deps $(CARGO_TARGET_DIR)/release/build $(CARGO_TARGET_DIR)/release/incremental $(CARGO_TARGET_DIR)/release/.fingerprint
	rm -f ../target/validator-rom.lst ../target/validator-rom.sym

clean: mostlyclean
	rm -rf $(CARGO_TARGET_DIR)
	rm -f ../target/validator-rom.bin
