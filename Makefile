# Cargo tracks Rust sources, embedded assets, dependencies, and build settings.
.DEFAULT_GOAL := all
.NOTPARALLEL:
.DELETE_ON_ERROR:
CARGO ?= cargo
TEST_PACKAGES ?= --workspace
RUSTFMT ?= rustfmt
UV ?= uv
ROM_MAKE ?= $(shell command -v gmake 2>/dev/null || command -v make)
PYTHON ?= python3
DASM ?= $(abspath rom/tools/dasm)
MANIFESTS := Cargo.toml src/rawemulator/Cargo.toml src/trace/Cargo.toml
.PHONY: all release adventure run format fmt lint check test smoke validate
.PHONY: rom gameplay analysis verify rom-check mostlyclean clean flamegraph python-reference
.PHONY: performance-build performance audio-performance performance-check

Cargo.lock: $(MANIFESTS)
	$(CARGO) generate-lockfile
	touch $@

all release adventure: Cargo.lock
	$(CARGO) build --locked --release -p adventure --bin adventure

run: Cargo.lock
	$(CARGO) run --locked --release -p adventure --bin adventure -- $(ARGS)

format:
	$(CARGO) fmt --all
	$(RUSTFMT) tests/performance.rs tests/trace_fixture.rs

fmt:
	$(CARGO) fmt --all --check
	$(RUSTFMT) --check tests/performance.rs tests/trace_fixture.rs

lint: fmt Cargo.lock
	$(CARGO) clippy --locked --workspace --all-targets -- -D warnings

check: lint performance-check
	$(ROM_MAKE) -C rom check DASM="$(DASM)" PYTHON="$(PYTHON)" UV="$(UV)"
	$(MAKE) test TEST_ARGS="$(TEST_ARGS)"

test: Cargo.lock
	$(CARGO) test --locked $(TEST_PACKAGES) $(TEST_ARGS)

smoke: Cargo.lock
	$(CARGO) run --locked -p adventure --example sdl_smoke -- $(ARGS)

validate: adventure
	@test -n "$(TRACE)" || { echo 'Usage: make validate TRACE=playthrough.trace' >&2; exit 1; }
	target/release/adventure --verify "$(TRACE)" $(ARGS)

rom gameplay analysis verify rom-check:
	$(ROM_MAKE) -C rom $(if $(filter rom,$@),all,$(if $(filter rom-check,$@),check,$@)) DASM="$(DASM)" PYTHON="$(PYTHON)" UV="$(UV)"

flamegraph: Cargo.lock
	CARGO_PROFILE_RELEASE_DEBUG=true $(CARGO) flamegraph --release -p adventure --bin adventure $(FLAMEGRAPH_ARGS) -- $(ARGS)

python-reference: Cargo.lock
	$(CARGO) build --locked -p adventure --example python_reference

performance-build: Cargo.lock
	$(CARGO) build --locked --release -p adventure --example adventure_performance

performance-check:
	$(UV) run --project rom --locked ruff format --check tests/performance.py
	$(UV) run --project rom --locked ruff check tests/performance.py
	$(UV) run --project rom --locked ty check tests/performance.py

performance: performance-build
	$(PYTHON) tests/performance.py

audio-performance: performance-build
	$(PYTHON) tests/performance.py --audio-comparison

# Keep deployed executables and user captures while discarding compiler intermediates.
mostlyclean:
	rm -rf target/debug target/release/deps target/release/build target/release/incremental target/release/.fingerprint target/release/examples
	$(ROM_MAKE) -C rom mostlyclean

clean: mostlyclean
	$(CARGO) clean
	$(ROM_MAKE) -C rom clean
