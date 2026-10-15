Python Adventure
================

Run from the repository root:

    make -C py run

Python 3.11+ and `uv` are required. Make installs the locked pygame-ce dependency into `py/.venv`. The game runs independently of Rust, the assembler, and cartridge files.

Enter starts or resets the game; arrows move; Space drops an object. Tab selects a game. F3/F4 select color/monochrome, F5/F6 set left difficulty A/B, and F7/F8 set right difficulty A/B. M toggles sound, 1/2/3 resize the window, and Escape quits. Keypad movement and the other keyboard alternatives match the Rust version; `uv run --directory py python3 main.py --help` lists them.

The port preserves the refactor's three-picture turn, byte-wrapped movement, maps and starting layouts, Game 3 placement sequence, rendered collision geometry, rotating two-object display, castle gates, dragon behavior, bat cargo, magnet, bridge, secret passage, victory, and sound envelopes. Rendering uses the same artwork, calibrated coordinates, scanline staging, and NTSC palette. The host presents at 60 pictures per second with the original display proportions.

`model.py` defines world records and controls. `game.py` owns scheduling, interactions, actors, and display preparation. `rooms.py` describes named map connections and room appearances; `artwork.py` contains wall and sprite data. `video.py` composes pixels and contacts. `sound.py` defines effect envelopes and the tone/noise circuit; `host.py` handles pygame input, window presentation, and playback. Runtime code has no recording, replay, tracing, profiling, or test infrastructure.

`make -C py check` runs formatting, lint, and type checks. `mostlyclean` removes Python caches while preserving the runnable environment; `clean` also removes that environment. Sources and configuration remain intact. The detailed runtime flow is in [NOTES.md](NOTES.md).
