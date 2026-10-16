Recorded easy playthrough
=========================

`easy.trace` preserves a complete human-recorded Game 1 playthrough from October 15, 2026. It contains 4,045 pictures (1:07:25 at 60 Hz), with victory observed at 1:02:03. Tests run it through the main game and independent cartridge execution, require complete trace consumption, and check victory. Tests never regenerate this fixture.

    make test TEST_ARGS=recorded_easy_playthrough
    make validate TRACE=tests/easy.trace


Performance workloads
=====================

`make performance-build` builds `target/release/examples/adventure_performance`. Each non-idle workload runs the saved trace, retaining the main game's rendering and collision work and validating against both implementations. `headless` uses no SDL or sample generation. `synthesis` additionally renders 800 mono float samples per picture at 48 kHz. `cached` builds the real sound bank and consumes the effect player's samples in virtual picture time, including sound-bank construction once per playthrough. These sample-generation benchmarks deliberately compute audio without a device; the public CLI's headless modes remain silent.

`sdl` uses the normal paced window with continuous synthesis; `sdl-cached` selects callback-driven cached effects. `idle-sdl` and `idle-sdl-cached` run the same picture count at Game Selection without tracing or controls. SDL workloads retain normal window size, cached presentation, audio backend, and pacing. Closing early fails the workload. Every non-idle run requires victory and its recorded endpoint.

    target/release/examples/adventure_performance headless 10

`make performance` collects five trials of ten headless and synthesis playthroughs, then paced idle and trace runs. `make audio-performance` compares continuous synthesis and cached output with isolated sample generation and reversed-order paced pairs. The driver uses child user/system CPU time, wall time, and macOS `coreaudiod`/`WindowServer` CPU deltas. Service totals include other applications; two no-game baselines expose background service activity. It records source hashes, toolchain, SDL version/linkage, and machine metadata beside JSONL samples in `target/`, refusing to replace existing results. Service measurements require macOS. Historical timings for removed implementations do not describe these workloads, which include raw validation.


Stella rendering reference
==========================

`stella-game-one.pixels` contains 160×193 indexed pixels captured from the original cartridge in Stella 7.0 with Game 1 started and no joystick movement. The cartridge SHA-256 is `b0326b45e5cab066b2d03f1e683708d82de3ffe60b2e968bf88c5494237f0a96`.

Stella's debugger script advances twelve frames, presses Reset for six frames using `swchb $3e`, releases it with `swchb $3f`, then advances six frames and executes `saveSnap`. The 320×228 snapshot has two horizontal display pixels per TIA pixel. The fixture takes every second horizontal pixel, removes thirteen top border scanlines, and maps black to 0, gray to 8, and yellow to 26. RGB palette differences therefore do not affect geometry. The final kernel scanline is excluded because the functional game paints that partially blanked ROM line in full.

The starting player occupies x=78–81, the Gold Key x=31–38, and the gate x=76–82. The key's first row is scanline 83 and the player's first row is scanline 147. The main game compares its entire 193-line raster against this fixture.

Position regression cases also use debugger snapshots of the cartridge's `PositionDrawable` routine with a one-bit normal sprite. Recorded `(game x, screen x)` pairs include `(0,159)`, `(1,0)`, `(32,31)`, `(80,79)`, `(159,158)`, `(164,3)`, `(165,156)`, `(179,10)`, `(225,156)`, `(239,10)`, `(240,11)`, and `(255,26)`. These cover screen wrapping and the reset landing in horizontal blanking or a following scanline.
