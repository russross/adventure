Adventure Rust ports
====================

Run the refactored game:

    make refactored-run

`translation/` keeps the Rust translation in ROM order. `refactored/` is an independent Rust implementation with all core game code and immutable data in `src/adventure.rs`, hardware modules directly in `src/`, and tests including the ROM oracle in `src/tests/`. Its hardware interface accepts complete scanlines instead of persistent drawable operations. Both versions retain line-by-line rendering, the three-picture gameplay cycle, and frame-boundary pacing.

SDL2 development files, `pkg-config`, and Rust with rustfmt and Clippy are required. Assembly and full-project checks also need GNU Make 4.3 or newer, a C compiler, `curl`, `tar`, `sha256sum`, Python 3.9 or newer, and `uv`. Make downloads and builds the pinned DASM assembler locally when needed. Normal execution uses compiled assets and does not read a cartridge file. The preserved assembly editions and analysis tools reside in `original/`; the reference `ADVNTURE.BIN` and rendering fixture are tracked inputs.


Build and checks
----------------

From a fresh checkout, build all implementations or run all checks:

    make
    make check

The root Makefile delegates assembly and analysis to `original/Makefile` and Rust builds to each implementation's Makefile. Assembly and analysis outputs reside in `original/build/`. Cargo and Python lockfiles are tracked; generated ROM fixtures, downloaded tools, virtual environments, and compiled outputs are rebuilt locally and excluded from Git. `DASM=/absolute/path/to/dasm` uses an existing assembler for assembly and test/validator fixtures.

    make refactored
    make refactored-check
    make refactored-smoke

`refactored-check` runs formatting, Clippy with warnings as errors, and behavioral tests. The test-only ROM fixture is assembled from the preserved assembly and checked against its known hash. Tests compare game behavior with the ROM and exact frame/state sequences with the preserved Rust translation. DASM is built through the existing local tool rule when needed for checks.

The SDL smoke runner opens a window, injects controls to retrieve the Gold Key, checks audio, mute, and window resizing, saves `target/refactored-smoke.bmp`, and closes automatically. Build outputs reside in `target/refactored/`. `make refactored-run` builds the release executable before launching it.

For a bounded run or BMP capture:

    make refactored-run ARGS='--frames 120 --snapshot target/refactored.bmp'

`--unpaced` disables frame pacing for automated runs. The normal host targets 60 presentations per second; scanline drawing never sleeps. `make -C refactored mostlyclean` preserves the release executable while removing intermediates. `make -C refactored clean` removes that executable as well.

`make run`, `make rust-check`, and `make rust-smoke` build, check, and exercise the translation through its own Makefile. Both versions use calibrated object placement checked against an independent Stella capture.


Record and validate a playthrough
--------------------------------

Run these commands from the project root. Choose either version to record:

    make run ARGS='--record playthrough.trace'
    make refactored-run ARGS='--record playthrough.trace'

Press Enter to start, play through to victory, then press Escape to finish the recording. The trace begins at boot and includes game selection, resets, consumed inputs, object changes, and observed collisions. Recording writes to the named file during play and flushes the final `end` record on normal exit. Use a different filename for each run; recording replaces an existing file.

Validate the completed trace against the original machine-language cartridge:

    make validate TRACE=playthrough.trace

This builds the headless validator, verifies the assembled cartridge's SHA-256, then executes its original instruction stream with the recorded inputs. The validator computes its own collision latches and compares its normalized game events with the trace. It opens no window and runs without frame pacing. Success prints `ROM verified N frames`; a mismatch reports the first expected and observed event with the CPU location and exits unsuccessfully.

To replay and verify the same trace against either Rust version without opening a window:

    make run ARGS='--replay playthrough.trace'
    make refactored-run ARGS='--replay playthrough.trace'

`make trace-check` checks the shared format and state machine. `make validator-check` checks ROM execution against generated translation traces. The format and observation contract are documented in [trace/SEMANTICS.md](trace/SEMANTICS.md).


Controls
--------

| Action                 | Keys                              |
|------------------------|-----------------------------------|
| Move player            | Arrow keys; keypad 8/2/4/6         |
| Fire / drop object     | Space; Left Ctrl; keypad 5         |
| Select game            | F1; Tab                           |
| Reset / start          | F2; Enter; keypad Enter            |
| Color / monochrome     | F3 / F4                           |
| Left difficulty A / B  | F5 / F6                           |
| Right difficulty A / B | F7 / F8                           |
| Toggle mute            | M                                 |
| Window size 1x/2x/3x   | 1 / 2 / 3 (number row)             |
| Quit                   | Escape; Ctrl+Q; Cmd+Q              |

Both difficulty switches start at B; the display starts in color at 2x size. Window shortcuts restore 320×194, 640×388, or 960×582 pixels. Muting preserves game effects and oscillator phase. The right joystick uses Y/H/G/J and F; its movement also affects inactivity.

The detailed contracts are in [SEMANTICS.md](SEMANTICS.md) and [refactored/SEMANTICS.md](refactored/SEMANTICS.md). The refactor's game is in [adventure.rs](refactored/src/adventure.rs); its rasterizer is in [video.rs](refactored/src/video.rs).
