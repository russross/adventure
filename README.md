Adventure Rust ports
====================

Run the refactored game:

    make refactored-run

`translation/` keeps the Rust translation in ROM order. `refactored/` is an independent Rust implementation organized around world records, movement, actor behavior, interactions, picture scheduling, and immutable game assets. Its hardware interface accepts complete scanlines instead of persistent drawable operations. Both versions retain line-by-line rendering, the three-picture gameplay cycle, and frame-boundary pacing.

SDL2, `pkg-config`, and Rust are required. Normal execution uses compiled assets and does not read a cartridge file. The preserved assembly editions and analysis tools reside in `original/`.


Build and checks
----------------

    make refactored
    make refactored-check
    make refactored-smoke

`refactored-check` runs formatting, Clippy with warnings as errors, and behavioral tests. The test-only ROM fixture is assembled from the preserved assembly and checked against its known hash. Tests compare game behavior with the ROM and exact frame/state sequences with the preserved Rust translation. DASM is built through the existing local tool rule when needed for checks.

The SDL smoke runner opens a window, injects controls to retrieve the Gold Key, checks audio, mute, and window resizing, saves `target/refactored-smoke.bmp`, and closes automatically. Build outputs reside in `target/refactored/`. `make refactored-run` builds the release executable before launching it.

For a bounded run or BMP capture:

    make refactored-run ARGS='--frames 120 --snapshot target/refactored.bmp'

`--unpaced` disables frame pacing for automated runs. The normal host targets 60 presentations per second; scanline drawing never sleeps. `make -C refactored mostlyclean` preserves the release executable while removing intermediates. `make -C refactored clean` removes that executable as well.

`make run`, `make rust-check`, and `make rust-smoke` build, check, and exercise the translation through its own Makefile. Both versions use calibrated object placement checked against an independent Stella capture. The root assembly and analysis targets still refer to the layout before those sources moved into `original/`.


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

The detailed contracts are in [SEMANTICS.md](SEMANTICS.md) and [refactored/SEMANTICS.md](refactored/SEMANTICS.md). The refactor's entry points are [game/lifecycle.rs](refactored/src/game/lifecycle.rs), [game/rendering.rs](refactored/src/game/rendering.rs), and [video.rs](refactored/src/video.rs).
