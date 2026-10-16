# The root Makefile owns the executable and workspace checks.
.DEFAULT_GOAL := all
.PHONY: all release format fmt lint check test mostlyclean clean run smoke flamegraph
all release format fmt lint check test mostlyclean clean run smoke flamegraph:
	$(MAKE) -C .. $@
