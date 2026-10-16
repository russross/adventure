Recorded easy playthrough
=========================

`easy.trace` preserves a complete human-recorded Game 1 playthrough from October 15, 2026. It contains 4,045 pictures (1:07:25 at 60 Hz), with victory observed at 1:02:03. Tests run it through the main game and independent cartridge execution, require complete trace consumption, and check victory. Tests never regenerate this fixture.

    make test TEST_ARGS=recorded_easy_playthrough
    make validate TRACE=tests/easy.trace


Performance workloads
=====================

`make performance-build` builds `target/release/examples/adventure_performance`. Each non-idle workload runs the saved trace, retaining the main game's rendering and collision work and validating against both implementations. `headless` uses no SDL or sample generation. `synthesis` additionally renders 800 mono float samples per picture at 48 kHz. The sample-generation benchmark deliberately compute audio without a device; the public CLI's headless modes remain silent.

`sdl` uses the normal paced window with continuous synthesis. `idle-sdl` runs the same picture count at Game Selection without tracing or controls. SDL workloads retain normal window size, cached presentation, audio backend, and pacing. Closing early fails the workload. Every non-idle run requires victory and its recorded endpoint.

    target/release/examples/adventure_performance headless 10

`make performance` collects five trials of ten headless and synthesis playthroughs, then paced idle and trace runs. The driver uses child user/system CPU time, wall time, and macOS `coreaudiod`/`WindowServer` CPU deltas. Service totals include other applications; two no-game baselines expose background service activity. It records source hashes, toolchain, SDL version/linkage, and machine metadata beside JSONL samples in `target/`, refusing to replace existing results. Service measurements require macOS. Historical timings for removed implementations do not describe these workloads, which include raw validation.


Stella rendering reference
==========================

`stella-game-one.pixels` contains 160×193 indexed pixels captured from the original cartridge in Stella 7.0 with Game 1 started and no joystick movement. The cartridge SHA-256 is `b0326b45e5cab066b2d03f1e683708d82de3ffe60b2e968bf88c5494237f0a96`.

Stella's debugger script advances twelve frames, presses Reset for six frames using `swchb $3e`, releases it with `swchb $3f`, then advances six frames and executes `saveSnap`. The 320×228 snapshot has two horizontal display pixels per TIA pixel. The fixture takes every second horizontal pixel, removes thirteen top border scanlines, and maps black to 0, gray to 8, and yellow to 26. RGB palette differences therefore do not affect geometry. The original fixture contains the first 193 complete rows and remains unchanged.

The starting player occupies x=78–81, the Gold Key x=31–38, and the gate x=76–82. The key's first row is scanline 83 and the player's first row is scanline 147. `stella-game-one-final-row.pixels` adds the 194th row captured independently with installed Stella 7.0 on October 16, 2026, using the same script in a temporary configuration directory. The capture's preceding 193 rows exactly matched the existing fixture. Snapshot row 206 contains five yellow TIA pixels then 155 black pixels. Tests compare both fixtures against the complete picture.

The cartridge's final VBLANK store completes at CPU cycle 24. Stella's one-color-clock VBLANK delay and 68-clock horizontal blank yield `24*3+1-68 = 5` visible pixels. See [Stella's TIA implementation](https://github.com/stella-emu/stella/blob/7.0/src/emucore/tia/TIA.cxx). The original profile supplies this visible prefix to independent geometry and painting; the test-only functional raster clips its final row and the raw oracle applies the register delay. Tests distinguish contacts at x=4 from blanked contacts at x=5.

Position regression cases also use debugger snapshots of the cartridge's `PositionDrawable` routine with a one-bit normal sprite. Recorded `(game x, screen x)` pairs include `(0,159)`, `(1,0)`, `(32,31)`, `(80,79)`, `(159,158)`, `(164,3)`, `(165,156)`, `(179,10)`, `(225,156)`, `(239,10)`, `(240,11)`, and `(255,26)`. These cover screen wrapping and the reset landing in horizontal blanking or a following scanline.


Stella audio reference
======================

`stella-audio.txt` contains digital volume output from an uninterrupted Stella 7.0 audio channel. Each line contains decimal waveform, frequency, and volume controls followed by hexadecimal volume digits, one per complete `phase0`/`phase1` native clock pair. The 154 segments cover all waveform modes, changes while silent, resumption, and interleaved pickup/drop/roar/slain/swallow/victory register envelopes. Rust tests keep one oscillator alive across the entire fixture and compare every clock before PCM integration and filtering.

Run `make audio-reference` to reproduce and compare the capture. `tests/stella_audio/Makefile` downloads [AudioChannel.cxx](https://github.com/stella-emu/stella/blob/7.0/src/emucore/tia/AudioChannel.cxx) and [AudioChannel.hxx](https://github.com/stella-emu/stella/blob/7.0/src/emucore/tia/AudioChannel.hxx), verifies their SHA-256 hashes, and compiles the unmodified files with a C++17 compiler. The local harness supplies register writes; its platform and serialization shims contain no synthesis behavior. Capture output stays under `target/`, and the build never overwrites the checked-in fixture. Normal tests use the saved fixture without downloads or compilation.

PCM tests separately verify buffer-boundary invariance, channel mixing, filter behavior, and fractional sample counts at 44,101 Hz. SDL tests pause the consumer to check exact backlog replacement and verify that mute/discard policies retain synthesis phase. Gain and the 20 Hz DC-removal filter remain explicit presentation choices.
