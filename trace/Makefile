# Workspace rules keep Cargo outputs and run paths consistent.
.DEFAULT_GOAL := all
.PHONY: all release format fmt lint check mostlyclean clean

all:
	$(MAKE) -C .. trace

release:
	$(MAKE) -C .. trace

format:
	$(MAKE) -C .. format

fmt:
	$(MAKE) -C .. fmt

lint:
	$(MAKE) -C .. lint

check:
	$(MAKE) -C .. trace-check

mostlyclean:
	$(MAKE) -C .. mostlyclean

clean:
	$(MAKE) -C .. clean

