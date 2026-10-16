Adventure
=========

Build and play:

    make
    make run

The executable is `target/release/adventure`, based on the frame-oriented Rust game in `src/`. The independent Python port remains in `py/`; run it with `make -C py run`. Rust builds require SDL2 development files, pkg-config, and Rust with rustfmt and Clippy. The cartridge and assembly analysis tools remain in `rom/`.


Record, replay, and validate
---------------------------

| Command                         | Behavior                                                  |
|---------------------------------|-----------------------------------------------------------|
| `adventure`                     | Interactive play with display and sound                    |
| `adventure --record FILE`       | Record consumed gameplay inputs                           |
| `adventure --replay FILE`       | Play an input recording with display and sound             |
| `adventure --trace FILE`        | Record a trace and validate live against the cartridge     |
| `adventure --verify FILE`       | Validate a trace against both implementations, headlessly  |
| `adventure --validate`          | Validate interactive play without saving a trace           |

Add `--headless` for silent, full speed execution without SDL, or `--display` for display, sound, and normal pacing. Both switches work with input replay and trace verification. For example:

    make run ARGS='--record play.input'
    make run ARGS='--replay play.input --headless'
    make run ARGS='--trace play.trace'
    make validate TRACE=play.trace
    make run ARGS='--verify play.trace --display'

Every trace workflow runs the main game and an independent original-instruction emulator together. At each frame boundary the raw emulator catches up before the single pacing sleep. Input playback runs only the main game. `--record` can accompany `--trace` or `--validate`, using separate output paths. Recording replaces its output file and finishes on normal quit.

`--frames N` bounds live play; it is required for headless live runs. `--unpaced` removes display pacing. `--snapshot FILE.bmp` saves the final displayed picture. `--audio synthesized|cached` selects the displayed audio path; continuous synthesis is the default. `adventure --help` lists options and controls.


Controls
--------

| Action                | Keys                              |
|-----------------------|-----------------------------------|
| Move                  | Arrow keys; keypad 8/2/4/6         |
| Fire / drop           | Space; Left Ctrl; keypad 5         |
| Select game           | F1; Tab                           |
| Reset / start         | F2; Enter; keypad Enter            |
| Color / monochrome    | F3 / F4                           |
| Left difficulty A / B | F5 / F6                           |
| Right difficulty A/B  | F7 / F8                           |
| Mute                  | M                                 |
| Window size 1x/2x/3x  | Number-row 1 / 2 / 3               |
| Quit                  | Escape; Ctrl+Q; Cmd+Q              |

Displayed modes start unmuted at 960×582 with both difficulty switches at B. Mute, resizing, and quit remain live during replay and verification. Early quit ends playback without reporting complete verification. The right joystick uses Y/H/G/J and F; its directions affect inactivity.


Development
-----------

    make lint
    make test
    make smoke
    make -C py test

`make check` also verifies ROM reconstruction; it requires GNU Make 4.3+, a C compiler, curl, tar, sha256sum, Python, and uv. ROM builds select `gmake` when installed; `ROM_MAKE` and `DASM` provide overrides. Cargo uses one workspace lockfile and shared `target/` directory. `make mostlyclean` preserves the release executable; `make clean` removes build assets too.

[NOTES.md](NOTES.md) contains the consolidated gameplay, rendering, audio, recording, validation, Python, and ROM workflows. [tests/README.md](tests/README.md) describes retained fixtures and performance tools.
