.DEFAULT_GOAL := all
.PHONY: all run format check mostlyclean clean
UV ?= uv

all: uv.lock
	$(UV) sync --locked

uv.lock: pyproject.toml
	$(UV) lock

run: all
	$(UV) run --locked python3 main.py

format: all
	$(UV) run ruff format .
	$(UV) run ruff check --fix .

check: all
	$(UV) run ruff format --check .
	$(UV) run ruff check .
	$(UV) run ty check

mostlyclean:
	rm -rf __pycache__ tests/__pycache__ .ruff_cache
	rm -f ../target/py-smoke.bmp

clean: mostlyclean
	rm -rf .venv

.PHONY: test
# Build the reference through its workspace before checking the independent Python runtime.
test: check
	$(MAKE) -C .. frame-python-reference
	$(UV) run --locked python3 -m unittest discover -s tests

.PHONY: smoke
smoke: all
	$(UV) run --locked python3 -m tests.sdl_smoke
