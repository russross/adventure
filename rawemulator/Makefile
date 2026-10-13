# Workspace rules keep Cargo outputs and run paths consistent.
.DEFAULT_GOAL := all
.PHONY: all release format fmt lint check mostlyclean clean run

all:
	$(MAKE) -C .. rawemulator

release:
	$(MAKE) -C .. rawemulator

format:
	$(MAKE) -C .. format

fmt:
	$(MAKE) -C .. fmt

lint:
	$(MAKE) -C .. lint

check:
	$(MAKE) -C .. rawemulator-check

mostlyclean:
	$(MAKE) -C .. mostlyclean

clean:
	$(MAKE) -C .. clean

run:
	$(MAKE) -C .. rawemulator-run

