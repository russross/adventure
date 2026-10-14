# Cargo tracks Rust source dependencies; DASM produces only the test reference fixture.
.NOTPARALLEL:
.DEFAULT_GOAL := all
.DELETE_ON_ERROR:
.PHONY: all release run flamegraph format fmt lint check smoke mostlyclean clean
CARGO ?= cargo
DASM ?= $(abspath ../.tools/dasm)
export CARGO_TARGET_DIR := $(abspath ../target/refactored)

Cargo.lock: Cargo.toml ../trace/Cargo.toml ../translation/Cargo.toml
	$(CARGO) generate-lockfile
	touch $@

all: Cargo.lock
	$(CARGO) build --locked --bin refactored

release: Cargo.lock
	$(CARGO) build --locked --release --bin refactored

run: release
	cd .. && $(CARGO_TARGET_DIR)/release/refactored $(ARGS)

# Keep release optimization while including full symbols for profiling stacks.
flamegraph: Cargo.lock
	CARGO_PROFILE_RELEASE_DEBUG=true $(CARGO) flamegraph --release --bin refactored $(FLAMEGRAPH_ARGS) -- $(ARGS)

$(abspath ../.tools/dasm): ../original/Makefile
	$(MAKE) -C .. .tools/dasm

../target/refactored-oracle.bin: ../original/adventure.asm $(DASM) Makefile
	mkdir -p ../target
	$(DASM) ../original/adventure.asm -f3 -o$@
	printf '%s  %s\n' b0326b45e5cab066b2d03f1e683708d82de3ffe60b2e968bf88c5494237f0a96 $@ | sha256sum -c -

format:
	$(CARGO) fmt

fmt:
	$(CARGO) fmt --check

lint: fmt Cargo.lock ../target/refactored-oracle.bin
	$(CARGO) clippy --locked --all-targets -- -D warnings

check: lint
	$(CARGO) test --locked $(TEST_ARGS)

smoke: Cargo.lock
	$(CARGO) run --locked --bin sdl_smoke

mostlyclean:
	rm -rf $(CARGO_TARGET_DIR)/debug $(CARGO_TARGET_DIR)/release/deps $(CARGO_TARGET_DIR)/release/build $(CARGO_TARGET_DIR)/release/incremental $(CARGO_TARGET_DIR)/release/.fingerprint $(CARGO_TARGET_DIR)/release/examples
	rm -f $(CARGO_TARGET_DIR)/release/*.d $(CARGO_TARGET_DIR)/.rustc_info.json ../target/refactored-oracle.bin
	rm -f ../target/refactored-smoke.bmp ../target/refactored-smoke.png

clean: mostlyclean
	rm -rf $(CARGO_TARGET_DIR)
