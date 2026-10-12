# Cargo owns source dependencies; the supplied cartridge is used only by tests.
.NOTPARALLEL:
.DELETE_ON_ERROR:
.PHONY: all release run fmt lint check smoke mostlyclean clean
CARGO ?= cargo

Cargo.lock: Cargo.toml
	$(CARGO) generate-lockfile
	touch $@

all: Cargo.lock
	$(CARGO) build --locked --bin adventure

release: Cargo.lock
	$(CARGO) build --locked --release --bin adventure

run: release
	cd .. && translation/target/release/adventure $(ARGS)

fmt:
	$(CARGO) fmt --check

lint: fmt Cargo.lock
	$(CARGO) clippy --locked --all-targets -- -D warnings

check: lint
	$(CARGO) test --locked

smoke: Cargo.lock
	cd .. && $(CARGO) run --locked --manifest-path translation/Cargo.toml --bin sdl_smoke

mostlyclean:
	rm -rf target/debug target/release/deps target/release/build target/release/incremental target/release/.fingerprint target/release/examples
	rm -f target/release/*.d target/.rustc_info.json ../target/rust-smoke.bmp ../target/rust-smoke.png

clean: mostlyclean
	rm -rf target
