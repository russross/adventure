# Workspace rules keep Cargo outputs and run paths consistent.
.DEFAULT_GOAL := all
.PHONY: all release format fmt lint check mostlyclean clean run smoke

all:
	$(MAKE) -C .. directport

release:
	$(MAKE) -C .. directport

format:
	$(MAKE) -C .. format

fmt:
	$(MAKE) -C .. fmt

lint:
	$(MAKE) -C .. lint

check:
	$(MAKE) -C .. directport-check

mostlyclean:
	$(MAKE) -C .. mostlyclean

clean:
	$(MAKE) -C .. clean

run:
	$(MAKE) -C .. directport-run

smoke:
	$(MAKE) -C .. directport-smoke

