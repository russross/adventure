# Cargo owns Rust dependency tracking and the shared target directory.
.DEFAULT_GOAL := all
.NOTPARALLEL:
.DELETE_ON_ERROR:
CARGO ?= cargo
UV ?= uv
PYTHON ?= python3
DASM ?= $(abspath rom/tools/dasm)
PACKAGES := directport scanline rawemulator trace
MANIFESTS := Cargo.toml $(addsuffix /Cargo.toml,$(PACKAGES))
.PHONY: all release format fmt lint check rom gameplay analysis verify rom-check
.PHONY: $(PACKAGES) $(addsuffix -check,$(PACKAGES))
.PHONY: directport-run scanline-run rawemulator-run directport-smoke scanline-smoke
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

fmt:
	$(CARGO) fmt --all --check

lint: fmt Cargo.lock
	$(CARGO) clippy --locked --workspace --all-targets -- -D warnings

check: lint
	$(MAKE) -C rom check DASM="$(DASM)" PYTHON="$(PYTHON)" UV="$(UV)"
	$(CARGO) test --locked --workspace $(TEST_ARGS)

$(addsuffix -check,$(PACKAGES)): Cargo.lock
	$(CARGO) fmt --all --check
	$(CARGO) clippy --locked -p $(if $(filter trace-check,$@),adventure-trace,$(@:-check=)) --all-targets -- -D warnings
	$(CARGO) test --locked -p $(if $(filter trace-check,$@),adventure-trace,$(@:-check=)) $(TEST_ARGS)

# Run from the workspace root so user paths and smoke captures have one base.
directport-run scanline-run rawemulator-run: Cargo.lock
	$(CARGO) run --locked --release -p $(@:-run=) --bin $(@:-run=) -- $(ARGS)

directport-smoke scanline-smoke: Cargo.lock
	$(CARGO) run --locked -p $(@:-smoke=) --bin $@ -- $(ARGS)

validate: rawemulator
	@test -n "$(TRACE)" || { echo 'Usage: make validate TRACE=playthrough.trace' >&2; exit 1; }
	target/release/rawemulator "$(TRACE)"

rom gameplay analysis verify rom-check:
	$(MAKE) -C rom $(if $(filter rom,$@),all,$(if $(filter rom-check,$@),check,$@)) DASM="$(DASM)" PYTHON="$(PYTHON)" UV="$(UV)"

# Keep release executables usable while discarding compiler intermediates.
mostlyclean:
	rm -rf target/debug target/release/deps target/release/build target/release/incremental target/release/.fingerprint target/release/examples
	rm -f target/directport-smoke.bmp target/directport-smoke.trace target/scanline-smoke.bmp
	$(MAKE) -C rom mostlyclean

clean: mostlyclean
	$(CARGO) clean
	$(MAKE) -C rom clean
