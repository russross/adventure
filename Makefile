# Each implementation owns its dependency tracking and build outputs.
.NOTPARALLEL:
.DELETE_ON_ERROR:
.PHONY: all original gameplay analysis verify lint check rust rust-release rust-check run rust-smoke
.PHONY: refactored refactored-release refactored-check refactored-run refactored-smoke
.PHONY: trace-check validator validator-check validate mostlyclean clean
DASM ?= .tools/dasm
PYTHON ?= python3
UV ?= uv
CARGO ?= cargo

all: original rust refactored validator

original:
	$(MAKE) -C original all DASM="$(abspath $(DASM))" PYTHON="$(PYTHON)"

gameplay analysis verify:
	$(MAKE) -C original $@ DASM="$(abspath $(DASM))" PYTHON="$(PYTHON)"

# Other build directories share the locally built assembler through this file target.
.tools/dasm: original/Makefile
	$(MAKE) -C original "$(abspath .tools/dasm)"

lint:
	$(MAKE) -C original lint UV="$(UV)"

check: lint trace-check rust-check refactored-check validator-check verify

rust:
	$(MAKE) -C translation all CARGO="$(CARGO)"

rust-release:
	$(MAKE) -C translation release CARGO="$(CARGO)"

rust-check:
	$(MAKE) -C translation check CARGO="$(CARGO)" TEST_ARGS="$(TEST_ARGS)"

run: rust-release
	translation/target/release/adventure $(ARGS)

rust-smoke:
	$(MAKE) -C translation smoke CARGO="$(CARGO)"

refactored:
	$(MAKE) -C refactored all CARGO="$(CARGO)"

refactored-release:
	$(MAKE) -C refactored release CARGO="$(CARGO)"

refactored-check:
	$(MAKE) -C refactored check CARGO="$(CARGO)" DASM="$(abspath $(DASM))" TEST_ARGS="$(TEST_ARGS)"

refactored-run:
	$(MAKE) -C refactored run CARGO="$(CARGO)" ARGS="$(ARGS)"

refactored-smoke:
	$(MAKE) -C refactored smoke CARGO="$(CARGO)"

trace-check:
	$(MAKE) -C trace check CARGO="$(CARGO)"

validator:
	$(MAKE) -C validator all CARGO="$(CARGO)" DASM="$(abspath $(DASM))"

validator-check:
	$(MAKE) -C validator check CARGO="$(CARGO)" DASM="$(abspath $(DASM))" TEST_ARGS="$(TEST_ARGS)"

validate: validator
	@test -n "$(TRACE)" || { echo 'Usage: make validate TRACE=playthrough.trace' >&2; exit 1; }
	target/validator/release/adventure-validator "$(TRACE)"

# Source files, locks, configurations, and recorded user traces are never cleanup outputs.
mostlyclean:
	$(MAKE) -C original mostlyclean
	$(MAKE) -C trace mostlyclean
	$(MAKE) -C validator mostlyclean
	$(MAKE) -C translation mostlyclean
	$(MAKE) -C refactored mostlyclean

clean:
	$(MAKE) -C original clean
	$(MAKE) -C trace clean
	$(MAKE) -C validator clean
	$(MAKE) -C translation clean
	$(MAKE) -C refactored clean
