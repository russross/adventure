Recorded easy playthrough
=========================

`easy.trace` is a human-recorded complete Game 1 playthrough, recorded on October 15, 2026. It contains 4,045 pictures (1:07:25 at 60 Hz), with victory observed at 1:02:03. The original-instruction `rawemulator` verified the recording before it became a regression fixture.

The standard Rust tests validate the entire recording against the original cartridge and replay it through directport, scanline, and frame, checking that each port finishes with victory. Run these checks with `make test TEST_ARGS=recorded_easy_playthrough`, or validate the cartridge directly with `make validate TRACE=tests/easy.trace`. The fixture is checked in as recorded; tests do not regenerate it from a port.


Performance workloads
---------------------

`make performance` builds three release examples and runs `tests/performance.py` on macOS. It writes each completed measurement to `target/performance.jsonl` and refuses to overwrite existing results. Move that file before collecting another set. `target/performance.environment.json` records machine/toolchain versions and SHA-256 hashes of the trace and relevant sources. CPU time is child user plus system time, including all application threads; wall time is recorded separately. Samples run sequentially. Headless measurements use five trials of ten complete traces each, after a warmup. SDL measurements use one complete 4,045-picture interval each at 60 Hz.

The shared `performance.rs` body supplies headless and SDL workloads. `headless` verifies the entire semantic recording with rendering and collisions, without SDL or sample generation. `synthesis` adds continuous 48 kHz mono synthesis, discarding the samples after making their work observable to the optimizer. Frame also supports `cached`, which builds its actual sound bank, feeds commands to its actual effect player, and consumes 800 samples per accepted picture while the player is active. Sound-bank construction is included once per playthrough. `sdl` verifies the trace through the actual paced window/audio host using its default synthesized audio; `sdl-cached` selects frame's cached output. `idle-sdl` and `idle-sdl-cached` run the same number of pictures at the default Game Selection screen without a trace or held controls. SDL modes retain the normal startup window size, display caching, audio backend, and pacing. Closing a window early fails the workload. Cached modes are rejected for directport and scanline.

The runner measures cumulative `coreaudiod` and `WindowServer` CPU around each workload and takes two 30-second no-game baselines around the SDL series. Service measurements are system-wide and can include other applications. Baseline subtraction estimates incremental service cost; it is not per-client attribution. Unpaced synthesis never opens an audio device. Paced SDL playback is required to exercise the real callback and audio server over the full recording duration. Application CPU includes startup, trace parsing, session validation, and shutdown; internal wall timing begins after parsing. No GPU time or kernel-wide CPU accounting is collected.

For an individual workload, run `make performance-build` followed by `/usr/bin/time -l target/release/examples/frame_performance headless 10`, replacing `frame` and `headless` as needed. Both replay modes require the recorded victory and complete trace consumption; the host rejects the extra boundary used to terminate replay before generating additional audio or presenting another picture.

`make audio-performance` compares both frame audio modes and preserves results separately in `target/audio-performance.jsonl` and `target/audio-performance.environment.json`. It measures headless/no-audio, continuous synthesis, and cached rendering in five trials of ten repetitions. It then measures two paired full SDL playthroughs per mode, reversing mode order on the second pair, followed by one idle run per mode. Two 30-second no-game baselines bracket the SDL series. Cached rendering in virtual picture time isolates PCM generation/copying cost; actual cached SDL playback remains driven by device callbacks.


Mac measurements, October 15, 2026
---------------------------------

Apple M4, macOS 15.7.9, Rust 1.99.0, release profile. This baseline used the cached-audio hosts in scanline and frame; their source hashes are preserved beside the raw measurements. The actual loaded libraries were Homebrew `sdl2-compat` 2.32.74 and SDL3 3.4.18. Output used the built-in MacBook Air speakers at 48 kHz. Headless numbers below are median application CPU seconds per complete 4,045-picture trace, from five trials of ten repetitions each. They include semantic verification, rendering, and collision work.

| Implementation | Headless | With continuous synthesis | Headless speedup over directport |
| -------------- | -------- | ------------------------- | -------------------------------- |
| directport     | 0.5202 s | 0.5397 s                  | 1.0×                             |
| scanline       | 0.1003 s | 0.1124 s                  | 5.2×                             |
| frame          | 0.0130 s | 0.0248 s                  | 39.9×                            |

The paced measurements below each lasted approximately 67.7 seconds and use application user plus system CPU seconds and raw service CPU seconds. They are single observations, not repeated-trial medians. Service accounting includes other applications. The SDL comparisons retain the normal software renderer and 2× window in directport, versus the default renderer, retained textures, and display caching in scanline/frame; scanline also uses a 2× window, while frame uses 3×; they measure complete implementation snapshots rather than attributing every difference to the game renderer.

| Implementation | Workload | Application | coreaudiod | WindowServer |
| -------------- | -------- | ----------- | ---------- | ------------ |
| directport     | Idle     | 12.62 s     | 7.26 s     | 18.24 s      |
| directport     | Trace    | 12.93 s     | 7.22 s     | 17.22 s      |
| scanline       | Idle     | 1.89 s      | 6.97 s     | 12.31 s      |
| scanline       | Trace    | 4.64 s      | 7.11 s     | 14.63 s      |
| frame          | Idle     | 1.83 s      | 7.01 s     | 12.48 s      |
| frame          | Trace    | 4.06 s      | 6.67 s     | 13.94 s      |

Both 30-second no-game baselines measured 0.00 seconds of `coreaudiod` CPU. `WindowServer` used 5.08 and 4.75 seconds, respectively, so its raw SDL totals contain substantial background display activity and should not be attributed entirely to the game.

Continuous synthesis itself added approximately 12–20 milliseconds of application CPU per trace. In contrast, the audio service consumed approximately 10% of one core throughout both idle and active SDL playback, including the cached-effect implementations. The large headless speedups therefore translate into smaller improvements once SDL and backend overhead are included.

The installed [compatibility layer forwards pause to SDL3](https://github.com/libsdl-org/sdl2-compat/blob/release-2.32.74/src/sdl2_compat.c#L7813), whose [pause implementation marks the logical device paused](https://github.com/libsdl-org/SDL/blob/release-3.4.18/src/audio/SDL_audio.c#L1770); the physical mixer continues producing silence. This is consistent with the measured idle service cost despite the application's paused effect callback. The benchmark does not alter device lifetime or any runtime implementation.


Frame audio comparison
----------------------

With both frame audio paths available in the same release build on the M4 above, headless medians used five trials of ten playthroughs each. Full SDL values are means of two complete playthroughs per mode, with reversed order in the second pair. Each SDL playthrough lasted approximately 67.7 seconds. Headless values include rendering, semantic verification, and audio generation; cached sound-bank construction is included once per playthrough. The no-audio headless baseline was 0.01298 CPU seconds.

| Audio mode  | Headless application | SDL application | SDL coreaudiod |
| ----------- | -------------------- | --------------- | -------------- |
| Synthesized | 0.02465 s            | 4.077 s         | 6.625 s        |
| Cached      | 0.01708 s            | 3.939 s         | 6.615 s        |

Single 67.7-second idle runs measured 1.769 application CPU seconds and 6.42 `coreaudiod` seconds with synthesis, versus 1.753 and 6.47 with cached audio. Both 30-second no-game baselines measured 0.00 audio-service CPU seconds. Pausing the cached callback therefore did not remove the backend's idle cost on this machine.

Cached rendering saved approximately 7.6 milliseconds of isolated CPU work per trace. With SDL, cached playback saved approximately 0.14 application CPU seconds, or 0.2 percentage points of one core averaged over the playthrough. Audio-service CPU was effectively unchanged. Synthesized audio remains the default because this absolute saving is small and continuous phase preserves compatibility across effects and silence. Cached output remains available through `--audio cached`; `--audio synthesized` selects the default explicitly. Raw samples and source/toolchain metadata are in `target/audio-performance.jsonl` and `target/audio-performance.environment.json`, separate from the earlier cross-implementation baseline. An interrupted collection made before validation finished is excluded from these results and archived as `target/audio-performance.precheck.jsonl`.


Stella rendering reference
==========================

`stella-game-one.pixels` contains 160×193 indexed pixels captured from the original cartridge in Stella 7.0 with Game 1 started and no joystick movement. The cartridge SHA-256 is `b0326b45e5cab066b2d03f1e683708d82de3ffe60b2e968bf88c5494237f0a96`.

Stella's debugger script advances twelve frames, presses Reset for six frames using `swchb $3e`, releases it with `swchb $3f`, then advances six frames and executes `saveSnap`. The 320×228 snapshot has two horizontal display pixels per TIA pixel. The fixture takes every second horizontal pixel, removes thirteen top border scanlines, and maps black to 0, gray to 8, and yellow to 26. RGB palette differences therefore do not affect the geometry comparison. The final kernel scanline is excluded because the functional ports deliberately paint that partially blanked ROM line in full.

The starting player occupies x=78–81, the Gold Key occupies x=31–38, and the gate occupies x=76–82. The key's first row is scanline 83 and the player's first row is scanline 147. Both Rust versions compare their entire 193-line raster against this independently rendered fixture.

Position regression cases also use debugger snapshots of the original `PositionDrawable` routine with a one-bit normal sprite. Recorded `(game x, screen x)` pairs include `(0,159)`, `(1,0)`, `(32,31)`, `(80,79)`, `(159,158)`, `(164,3)`, `(165,156)`, `(179,10)`, `(225,156)`, `(239,10)`, `(240,11)`, and `(255,26)`. These cases cover screen wrapping and the reset landing in horizontal blanking or a following scanline.
