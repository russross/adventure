Adventure Rust ports
====================

Run the frame-oriented game:

    make run

Run the Python port with `make -C py run`; see [py/README.md](py/README.md). `make run`, `make rust-check`, and `make rust-smoke` select `frame`; `translation-*` aliases select `directport`.

`frame/` is the ongoing Rust development target. It prepares complete pictures and paints each whole object before the next, with independent coverage preserving hidden collisions and foreground priority. Its host retains textures, uploads changed rows, redraws on picture or window changes, and synthesizes sound continuously by default. `--audio cached` selects cached effects; `--audio synthesized` explicitly selects continuous waveform phase. Either option works with live play, recording, replay, and trace verification. Its rendering and runtime contracts are in [frame/SEMANTICS.md](frame/SEMANTICS.md).

`directport/` preserves the translation in ROM order. `scanline/` preserves the next step in the progression, with complete scanlines, continuous audio synthesis, and its independent ROM checks. Both remain available through explicit package targets. `py/` independently implements the frame runtime in Python without recording/replay.

SDL2 development files, `pkg-config`, and Rust with rustfmt and Clippy are required. Assembly and full-project checks also need GNU Make 4.3 or newer, a C compiler, `curl`, `tar`, `sha256sum`, Python 3.9 or newer, and `uv`. Make downloads and builds the pinned DASM assembler locally when needed. Normal execution uses compiled assets and does not read a cartridge file. The preserved assembly editions and analysis tools reside in `rom/`; the reference `rom/adventure.bin` and rendering fixture are tracked inputs.


Build and checks
----------------

From a fresh checkout, build all implementations or run all checks:

    make
    make check

The root Cargo workspace owns all Rust builds, one `Cargo.lock`, and the shared `target/` directory. Plain `make` builds release binaries for all versions, including smoke runners. Run `make run` or `make frame-run`; executables reside in `target/release/`. Each crate has an `adventure.bin` symlink to `rom/adventure.bin`, used by the emulator and test oracles. Cargo builds and tests work directly without generated ROM fixtures or DASM.

    make rust-check
    make rust-smoke
    make -C py test

Checks run formatting, Clippy with warnings as errors, and behavioral tests. The SDL smoke runners open windows, inject controls to retrieve the Gold Key, check audio, mute, and window resizing, save `target/frame-smoke.bmp` for frame, and close automatically.

ROM targets select `gmake` when installed (override with `ROM_MAKE`); direct ROM commands on macOS use `gmake -C rom`. `make rom` independently assembles both disassemblies and runs analysis under `rom/build/`. `make verify` compares both rebuilt cartridges with the tracked ROM and checks control flow, graphics, and timing. Python configuration, lockfile, virtual environment, and development tools belong to `rom/`. `DASM=/absolute/path/to/dasm` selects an existing assembler; otherwise `rom/tools/dasm` is built locally.

For a bounded run or BMP capture:

    make run ARGS='--frames 120 --snapshot target/frame.bmp'

`--unpaced` disables frame pacing for automated runs. The normal host targets 60 presentations per second; picture composition never sleeps. `make -C frame mostlyclean` preserves the release executable while removing intermediates. `make -C frame clean` removes that executable as well.

`make directport-run`, `make directport-check`, and `make directport-smoke` build, check, and exercise the translation through its own Makefile. Both versions use calibrated object placement checked against an independent Stella capture.


Record and replay the frame version
-----------------------------------

    make frame-run ARGS='--record playthrough.input'
    make frame-run ARGS='--replay playthrough.input'

Recording saves only changes to inputs observed by gameplay and the stopping boundary. Replay re-emulates the game with those inputs, using SDL audio and pacing. Mute, window size, and quit remain available. It does not validate game state. Files depend on the frame version's logical input read schedule.

For the existing validation trace workflow, use `--trace playthrough.trace` and `--verify playthrough.trace` with `frame-run`. The other versions retain their existing option names.


Record and validate a playthrough
--------------------------------

Run these commands from the project root:

    make run ARGS='--trace playthrough.trace'

Press Enter to start, play through to victory, then press Escape to finish the recording. The trace begins at boot and includes game selection, resets, consumed inputs, object changes, and observed collisions. Recording writes to the named file during play and flushes the final `end` record on normal exit. Use a different filename for each run; recording replaces an existing file.

Validate the completed trace against the original machine-language cartridge:

    make validate TRACE=playthrough.trace

This builds `rawemulator`, which embeds the tracked cartridge and executes its original instruction stream with the recorded inputs. The validator computes its own collision latches and compares its normalized game events with the trace. It opens no window and runs without frame pacing. Success prints `ROM verified N frames`; a mismatch reports the first expected and observed event with the CPU location and exits unsuccessfully.

To replay the trace through the frame runtime with SDL presentation, audio, and verification:

    make run ARGS='--verify playthrough.trace'

The preserved directport and scanline versions use `--record` for validation traces and `--replay` for headless trace verification; select them through `directport-run` and `scanline-run`.

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

Both difficulty switches start at B; frame and Python start in color at 3x size. Window shortcuts restore 320×194, 640×388, or 960×582 pixels. Muting leaves gameplay effect counters running; unmuting resumes the current envelope step. The right joystick uses Y/H/G/J and F; its movement also affects inactivity.

The detailed contracts are in [SEMANTICS.md](SEMANTICS.md) and [frame/SEMANTICS.md](frame/SEMANTICS.md). The game is in [adventure.rs](frame/src/adventure.rs); its compositor is in [video.rs](frame/src/video.rs).
