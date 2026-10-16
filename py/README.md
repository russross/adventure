Python Adventure
================

Run from the repository root:

    make -C py run

Python 3.11+ and `uv` are required. Make installs the locked pygame-ce dependency into `py/.venv`. The game runs independently of Rust, the assembler, and cartridge files.

Enter starts or resets the game; arrows move; Space drops an object. Tab selects a game. F3/F4 select color/monochrome, F5/F6 set left difficulty A/B, and F7/F8 set right difficulty A/B. M toggles sound, 1/2/3 resize the window, and Escape quits. Keypad movement and the other keyboard alternatives match the Rust version; `uv run --directory py python3 main.py --help` lists them.

The port preserves the frame version's three-picture turn, byte-wrapped movement, maps and starting layouts, Game 3 placement sequence, rendered collision geometry, rotating two-object display, castle gates, dragon behavior, bat cargo, magnet, bridge, secret passage, victory, and sound envelopes. Rendering uses the same artwork, calibrated coordinates, whole-picture layers with the calibrated vertical delays, and NTSC palette. The host targets 60 pictures per second, starts at 960×582 (3x), and retains indexed and scaled pictures to avoid unchanged uploads and redraws. Exposure and resize still repaint.

`model.py` defines world records and controls. `game.py` owns scheduling, interactions, actors, and display preparation. `rooms.py` describes named map connections and room appearances; `artwork.py` contains wall and sprite data. `video.py` composes pixels and contacts. `sound.py` defines effect envelopes and the tone/noise circuit; `host.py` handles pygame input, window presentation, and playback. Runtime code has no recording, replay, tracing, or profiling.

`make -C py check` runs formatting, lint, and type checks. `make -C py test` also compares exact pictures, sound registers, and final positions against the Rust frame runtime; these development checks require Rust and SDL2. `make -C py smoke` opens a bounded real SDL play session covering pickup, mute, cached repainting, resize, and quit. `mostlyclean` removes Python caches while preserving the runnable environment; `clean` also removes that environment. Sources and configuration remain intact. The detailed runtime flow is in [SEMANTICS.md](SEMANTICS.md).
