.NOTPARALLEL:
.DEFAULT_GOAL := all
.PHONY: all format fmt lint check mostlyclean clean
CARGO ?= cargo
export CARGO_TARGET_DIR := $(abspath ../target/trace)

Cargo.lock: Cargo.toml
	$(CARGO) generate-lockfile
	touch $@

all: Cargo.lock
	$(CARGO) build --locked

format:
	$(CARGO) fmt

fmt:
	$(CARGO) fmt --check

lint: fmt Cargo.lock
	$(CARGO) clippy --locked --all-targets -- -D warnings

check: lint
	$(CARGO) test --locked

mostlyclean:
	rm -rf $(CARGO_TARGET_DIR)/debug $(CARGO_TARGET_DIR)/release/deps $(CARGO_TARGET_DIR)/release/build $(CARGO_TARGET_DIR)/release/incremental $(CARGO_TARGET_DIR)/release/.fingerprint

clean: mostlyclean
	rm -rf $(CARGO_TARGET_DIR)
