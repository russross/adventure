.NOTPARALLEL:
.PHONY: all format fmt lint check mostlyclean clean
CARGO ?= cargo
export CARGO_TARGET_DIR := $(abspath ../target/trace)

all:
	$(CARGO) build

format:
	$(CARGO) fmt

fmt:
	$(CARGO) fmt --check

lint: fmt
	$(CARGO) clippy --all-targets -- -D warnings

check: lint
	$(CARGO) test

mostlyclean:
	rm -rf $(CARGO_TARGET_DIR)/debug $(CARGO_TARGET_DIR)/release/deps $(CARGO_TARGET_DIR)/release/build $(CARGO_TARGET_DIR)/release/incremental $(CARGO_TARGET_DIR)/release/.fingerprint

clean: mostlyclean
	rm -rf $(CARGO_TARGET_DIR)
