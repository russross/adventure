# Capture digital output from the hash-checked, unmodified Stella 7.0 audio channel.
.DEFAULT_GOAL := check
.DELETE_ON_ERROR:
BUILD := ../../target/stella-audio-reference
UPSTREAM := https://raw.githubusercontent.com/stella-emu/stella/7.0/src/emucore/tia
CXX ?= c++
CXXFLAGS ?= -O2 -std=c++17 -Wall -Wextra -Werror
.PHONY: check capture mostlyclean clean

$(BUILD):
	mkdir -p $@

$(BUILD)/AudioChannel.cxx: Makefile | $(BUILD)
	curl -fsSL $(UPSTREAM)/AudioChannel.cxx -o $@.tmp
	printf '%s  %s\n' 348a1269d50f22b634d6b12c65cdafbac1fc3e80d30fbece55f3dbdbbe847d74 $@.tmp | shasum -a 256 -c -
	mv $@.tmp $@

$(BUILD)/AudioChannel.hxx: Makefile | $(BUILD)
	curl -fsSL $(UPSTREAM)/AudioChannel.hxx -o $@.tmp
	printf '%s  %s\n' 1f9e25bcd8d68698e82937d65297fb6159e0c0263d335f6b5d1bfccb92c718c0 $@.tmp | shasum -a 256 -c -
	mv $@.tmp $@

$(BUILD)/capture: capture.cxx bspf.hxx Serializable.hxx $(BUILD)/AudioChannel.cxx $(BUILD)/AudioChannel.hxx Makefile
	$(CXX) $(CXXFLAGS) -I. -I$(BUILD) capture.cxx $(BUILD)/AudioChannel.cxx -o $@

$(BUILD)/audio.txt: $(BUILD)/capture
	$< > $@

capture: $(BUILD)/audio.txt

check: $(BUILD)/audio.txt
	cmp ../stella-audio.txt $<

# Captured output survives intermediate cleanup; the checked-in fixture is never a build output.
mostlyclean:
	rm -f $(BUILD)/capture $(BUILD)/AudioChannel.cxx $(BUILD)/AudioChannel.hxx $(BUILD)/*.tmp

clean: mostlyclean
	rm -f $(BUILD)/audio.txt
