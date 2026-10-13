Adventure Rust ports
====================

Run the refactored game:

    make scanline-run

`directport/` keeps the Rust translation in ROM order. `scanline/` is an independent Rust implementation with all core game code and immutable data in `src/adventure.rs`, hardware modules directly in `src/`, and tests including the ROM oracle in `src/tests/`. Its hardware interface accepts complete scanlines instead of persistent drawable operations. Both versions retain line-by-line rendering, the three-picture gameplay cycle, and frame-boundary pacing.

`frame/` copies the scanline gameplay and prepares complete pictures, painting each whole object before the next. Independent coverage masks preserve hidden collisions and foreground priority. Run it with `make frame-run`; `make frame-check` and `make frame-smoke` check its ROM equivalence and SDL behavior. Its rendering contract is in [frame/SEMANTICS.md](frame/SEMANTICS.md).

SDL2 development files, `pkg-config`, and Rust with rustfmt and Clippy are required. Assembly and full-project checks also need GNU Make 4.3 or newer, a C compiler, `curl`, `tar`, `sha256sum`, Python 3.9 or newer, and `uv`. Make downloads and builds the pinned DASM assembler locally when needed. Normal execution uses compiled assets and does not read a cartridge file. The preserved assembly editions and analysis tools reside in `rom/`; the reference `rom/adventure.bin` and rendering fixture are tracked inputs.


Build and checks
----------------

From a fresh checkout, build all implementations or run all checks:

    make
    make check

The root Cargo workspace owns all Rust builds, one `Cargo.lock`, and the shared `target/` directory. Plain `make` builds release binaries for all versions, including smoke runners. Run `make scanline-run` or `make directport-run`; executables reside in `target/release/`. Each crate has an `adventure.bin` symlink to `rom/adventure.bin`, used by the emulator and test oracles. Cargo builds and tests work directly without generated ROM fixtures or DASM.

    make scanline-check
    make scanline-smoke
    make directport-smoke

Checks run formatting, Clippy with warnings as errors, and behavioral tests. The SDL smoke runners open windows, inject controls to retrieve the Gold Key, check audio, mute, and window resizing, save `target/scanline-smoke.bmp` or `target/directport-smoke.bmp`, and close automatically.

ROM targets select `gmake` when installed (override with `ROM_MAKE`); direct ROM commands on macOS use `gmake -C rom`. `make rom` independently assembles both disassemblies and runs analysis under `rom/build/`. `make verify` compares both rebuilt cartridges with the tracked ROM and checks control flow, graphics, and timing. Python configuration, lockfile, virtual environment, and development tools belong to `rom/`. `DASM=/absolute/path/to/dasm` selects an existing assembler; otherwise `rom/tools/dasm` is built locally.

For a bounded run or BMP capture:

    make scanline-run ARGS='--frames 120 --snapshot target/scanline.bmp'

`--unpaced` disables frame pacing for automated runs. The normal host targets 60 presentations per second; scanline drawing never sleeps. `make -C scanline mostlyclean` preserves the release executable while removing intermediates. `make -C scanline clean` removes that executable as well.

`make directport-run`, `make directport-check`, and `make directport-smoke` build, check, and exercise the translation through its own Makefile. Both versions use calibrated object placement checked against an independent Stella capture.


Record and validate a playthrough
--------------------------------

Run these commands from the project root. Choose either version to record:

    make directport-run ARGS='--record playthrough.trace'
    make scanline-run ARGS='--record playthrough.trace'

Press Enter to start, play through to victory, then press Escape to finish the recording. The trace begins at boot and includes game selection, resets, consumed inputs, object changes, and observed collisions. Recording writes to the named file during play and flushes the final `end` record on normal exit. Use a different filename for each run; recording replaces an existing file.

Validate the completed trace against the original machine-language cartridge:

    make validate TRACE=playthrough.trace

This builds `rawemulator`, which embeds the tracked cartridge and executes its original instruction stream with the recorded inputs. The validator computes its own collision latches and compares its normalized game events with the trace. It opens no window and runs without frame pacing. Success prints `ROM verified N frames`; a mismatch reports the first expected and observed event with the CPU location and exits unsuccessfully.

To replay and verify the same trace against either Rust version without opening a window:

    make directport-run ARGS='--replay playthrough.trace'
    make scanline-run ARGS='--replay playthrough.trace'

`make trace-check` checks the shared format and state machine. `make rawemulator-check` checks ROM execution against generated translation traces. The format and observation contract are documented in [trace/SEMANTICS.md](trace/SEMANTICS.md).


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

The detailed contracts are in [SEMANTICS.md](SEMANTICS.md) and [scanline/SEMANTICS.md](scanline/SEMANTICS.md). The refactor's game is in [adventure.rs](scanline/src/adventure.rs); its rasterizer is in [video.rs](scanline/src/video.rs).
