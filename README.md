Adventure functional port
=========================

Run the Rust game:

    make run

SDL2, `pkg-config`, and Rust are required. Cargo uses the installed SDL2 library through the `sdl2` crate; no SDL source build is needed. `make rust` builds `target/debug/adventure`. `make run` builds and launches `target/release/adventure` with release optimizations. The executable uses compiled world and graphics data and does not read a ROM file at runtime.

`src/adventure.rs` contains the named RAM records, translated subroutines in assembly order, and static graphics/world data in their corresponding source locations. `src/tia.rs` composes scanlines and collects contacts using persistent drawable objects. `src/hardware.rs` presents completed frames, queues mono audio through SDL, and samples controls. `src/sound.rs` generates both TIA sound channels, including their tone and noise modes, with persistent oscillator phase. Sound controls are applied at frame boundaries; CPU timing and analog circuitry are not emulated.


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

A compact control reference prints to the console at startup and with `--help`. Window size shortcuts restore 320×194, 640×388, or 960×582 pixels after manual resizing; the initial size is 2x. Mute silences playback while game effects and oscillator phase continue.

Both difficulty switches start at B and the display starts in color. The right joystick's Stella keys are also mapped: Y/H/G/J for up/down/left/right and F for fire. Adventure uses the left joystick to move the player; either joystick's movement affects its inactivity counter.

The core mappings follow [Stella's keyboard layout](https://stella-emu.github.io/docs/index.html#Keyboard). Tab and Enter are the requested additional Select and Reset aliases.


Checks and bounded runs
-----------------------

    make rust-check
    make rust-smoke

`rust-check` uses debug builds and runs formatting, Clippy with warnings treated as errors, then tests. A test-only ROM executor compares persistent game behavior and drives the original kernel's output sequence through the functional TIA interface. It is excluded from the running game. `rust-smoke` runs `src/bin/sdl_smoke.rs` in debug mode, opens an SDL window, sends real SDL key events to start the game and retrieve the Gold Key and checks its pickup audio, mute toggling, key repeat, and all three window sizes, saves `target/rust-smoke.bmp`, and closes automatically.

For a bounded executable run or a BMP capture:

    make run ARGS='--frames 120 --snapshot target/adventure.bmp'

`--unpaced` disables the vertical-sync pause for automated runs. Otherwise the executable presents at 60 frames per second. Horizontal synchronization only paints into the frame buffer. The display retains the original two-slot object rotation and row staging; it does not model CPU cycles, sub-line register deadlines, timer prescalers, horizontal positioning delays, or analog television effects.

`adventure.asm` remains the address-labeled baseline, and `adventure-gameplay.asm` remains the gameplay-oriented assembly edition. `make check` checks both assembly editions and the Rust port. `SEMANTICS.md` describes their workflows and invariants.
