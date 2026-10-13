# Workspace rules keep Cargo outputs and run paths consistent.
.DEFAULT_GOAL := all
.PHONY: all release format fmt lint check mostlyclean clean run smoke

all:
	$(MAKE) -C .. scanline

release:
	$(MAKE) -C .. scanline

format:
	$(MAKE) -C .. format

fmt:
	$(MAKE) -C .. fmt

lint:
	$(MAKE) -C .. lint

check:
	$(MAKE) -C .. scanline-check

mostlyclean:
	$(MAKE) -C .. mostlyclean

clean:
	$(MAKE) -C .. clean

run:
	$(MAKE) -C .. scanline-run

smoke:
	$(MAKE) -C .. scanline-smoke

