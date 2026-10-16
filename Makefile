# Cargo owns Rust dependency tracking and the shared target directory.
.DEFAULT_GOAL := all
.NOTPARALLEL:
.DELETE_ON_ERROR:
CARGO ?= cargo
RUSTFMT ?= rustfmt
UV ?= uv
ROM_MAKE ?= $(shell command -v gmake 2>/dev/null || command -v make)
PYTHON ?= python3
DASM ?= $(abspath rom/tools/dasm)
PACKAGES := directport scanline frame rawemulator trace
MANIFESTS := Cargo.toml $(addsuffix /Cargo.toml,$(PACKAGES))
.PHONY: all release format fmt lint check test rom gameplay analysis verify rom-check
.PHONY: $(PACKAGES) $(addsuffix -check,$(PACKAGES))
.PHONY: directport-run scanline-run frame-run rawemulator-run directport-smoke scanline-smoke frame-smoke
.PHONY: validate mostlyclean clean

Cargo.lock: $(MANIFESTS)
	$(CARGO) generate-lockfile
	touch $@

all release: Cargo.lock
	$(CARGO) build --locked --release --workspace --bins

$(PACKAGES): Cargo.lock
	$(CARGO) build --locked --release -p $(if $(filter trace,$@),adventure-trace,$@)

# Formatting and linting precede behavioral checks, including ROM verification.
format:
	$(CARGO) fmt --all
	$(RUSTFMT) tests/performance.rs

fmt:
	$(CARGO) fmt --all --check
	$(RUSTFMT) --check tests/performance.rs

lint: fmt Cargo.lock
	$(CARGO) clippy --locked --workspace --all-targets -- -D warnings

check: lint
	$(ROM_MAKE) -C rom check DASM="$(DASM)" PYTHON="$(PYTHON)" UV="$(UV)"
	$(MAKE) test TEST_ARGS="$(TEST_ARGS)"

test: Cargo.lock
	$(CARGO) test --locked --workspace $(TEST_ARGS)

$(addsuffix -check,$(PACKAGES)): Cargo.lock
	$(CARGO) fmt --all --check
	$(CARGO) clippy --locked -p $(if $(filter trace-check,$@),adventure-trace,$(@:-check=)) --all-targets -- -D warnings
	$(CARGO) test --locked -p $(if $(filter trace-check,$@),adventure-trace,$(@:-check=)) $(TEST_ARGS)

# Run from the workspace root so user paths and smoke captures have one base.
directport-run scanline-run frame-run rawemulator-run: Cargo.lock
	$(CARGO) run --locked --release -p $(@:-run=) --bin $(@:-run=) -- $(ARGS)

directport-smoke scanline-smoke frame-smoke: Cargo.lock
	$(CARGO) run --locked -p $(@:-smoke=) --bin $@ -- $(ARGS)

validate: rawemulator
	@test -n "$(TRACE)" || { echo 'Usage: make validate TRACE=playthrough.trace' >&2; exit 1; }
	target/release/rawemulator "$(TRACE)"

rom gameplay analysis verify rom-check:
	$(ROM_MAKE) -C rom $(if $(filter rom,$@),all,$(if $(filter rom-check,$@),check,$@)) DASM="$(DASM)" PYTHON="$(PYTHON)" UV="$(UV)"

# Keep release executables usable while discarding compiler intermediates.
mostlyclean:
	rm -rf target/debug target/release/deps target/release/build target/release/incremental target/release/.fingerprint target/release/examples
	rm -f target/directport-smoke.bmp target/directport-smoke.trace target/scanline-smoke.bmp target/frame-smoke.bmp target/py-smoke.bmp
	$(ROM_MAKE) -C rom mostlyclean

clean: mostlyclean
	$(CARGO) clean
	$(ROM_MAKE) -C rom clean

# Default play and legacy implementation names follow the workspace packages.
.PHONY: run rust rust-release rust-check rust-smoke translation translation-release translation-run translation-check translation-smoke scanline-flamegraph
run: frame-run
rust rust-release: frame
rust-check: frame-check
rust-smoke: frame-smoke
translation translation-release: directport
translation-run: directport-run
translation-check: directport-check
translation-smoke: directport-smoke

scanline-flamegraph: Cargo.lock
	CARGO_PROFILE_RELEASE_DEBUG=true $(CARGO) flamegraph --release -p scanline --bin scanline $(FLAMEGRAPH_ARGS) -- $(ARGS)

.PHONY: frame-flamegraph frame-python-reference
frame-flamegraph: Cargo.lock
	CARGO_PROFILE_RELEASE_DEBUG=true $(CARGO) flamegraph --release -p frame --bin frame $(FLAMEGRAPH_ARGS) -- $(ARGS)

frame-python-reference: Cargo.lock
	$(CARGO) build --locked -p frame --example python_reference

# Release examples share the saved trace and identical workload boundaries.
.PHONY: performance-build performance audio-performance
performance-build: Cargo.lock
	$(CARGO) build --locked --release -p directport --example directport_performance -p scanline --example scanline_performance -p frame --example frame_performance

performance: performance-build
	$(PYTHON) tests/performance.py

audio-performance: performance-build
	$(PYTHON) tests/performance.py --audio-comparison
