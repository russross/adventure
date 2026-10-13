# Workspace rules keep Cargo outputs and run paths consistent.
.DEFAULT_GOAL := all
.PHONY: all release format fmt lint check mostlyclean clean run smoke

all:
	$(MAKE) -C .. frame

release:
	$(MAKE) -C .. frame

format:
	$(MAKE) -C .. format

fmt:
	$(MAKE) -C .. fmt

lint:
	$(MAKE) -C .. lint

check:
	$(MAKE) -C .. frame-check

mostlyclean:
	$(MAKE) -C .. mostlyclean

clean:
	$(MAKE) -C .. clean

run:
	$(MAKE) -C .. frame-run

smoke:
	$(MAKE) -C .. frame-smoke

