# Cargo tracks Rust source dependencies; DASM produces only the test reference fixture.
.NOTPARALLEL:
.DELETE_ON_ERROR:
.PHONY: all release run fmt lint check smoke mostlyclean clean
CARGO ?= cargo
export CARGO_TARGET_DIR := $(abspath ../target/refactored)

Cargo.lock: Cargo.toml
	$(CARGO) generate-lockfile
	touch $@

all: Cargo.lock
	$(CARGO) build --locked --bin refactored

release: Cargo.lock
	$(CARGO) build --locked --release --bin refactored

run: release
	cd .. && $(CARGO_TARGET_DIR)/release/refactored $(ARGS)

../.tools/dasm:
	$(MAKE) -C .. .tools/dasm

../target/refactored-oracle.bin: ../original/adventure.asm ../.tools/dasm Makefile
	mkdir -p ../target
	../.tools/dasm ../original/adventure.asm -f3 -o$@
	printf '%s  %s\n' b0326b45e5cab066b2d03f1e683708d82de3ffe60b2e968bf88c5494237f0a96 $@ | sha256sum -c -

fmt:
	$(CARGO) fmt --check

lint: fmt Cargo.lock ../target/refactored-oracle.bin
	$(CARGO) clippy --locked --all-targets -- -D warnings

check: lint
	$(CARGO) test --locked

smoke: Cargo.lock
	$(CARGO) run --locked --bin sdl_smoke

mostlyclean:
	rm -rf $(CARGO_TARGET_DIR)/debug $(CARGO_TARGET_DIR)/release/deps $(CARGO_TARGET_DIR)/release/build $(CARGO_TARGET_DIR)/release/incremental $(CARGO_TARGET_DIR)/release/.fingerprint $(CARGO_TARGET_DIR)/release/examples
	rm -f $(CARGO_TARGET_DIR)/release/*.d $(CARGO_TARGET_DIR)/.rustc_info.json ../target/refactored-oracle.bin
	rm -f ../target/refactored-smoke.bmp ../target/refactored-smoke.png

clean: mostlyclean
	rm -rf $(CARGO_TARGET_DIR)
