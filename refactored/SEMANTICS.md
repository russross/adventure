Refactored Adventure
====================

This implementation uses the gameplay contract in [the project semantics](../SEMANTICS.md), including its room records, reaction priorities, wrapping comparisons, switch histories, collision interpretation, and audio synthesis. The preserved implementation in `../translation/` is the behavioral reference. The refactor neither calls it nor reads cartridge bytes during normal execution. The translation is a development dependency solely for differential tests and the reference raster used by the test-only ROM executor.


State and ownership
-------------------

`Adventure` owns four independent resources: `World`, current `Controls`, `Sound`, and `Video`. It also owns a private `PreparedDisplay` and `PicturePhase`. `World` contains live entity records, safe player position, carrying offsets, gate and dragon states, selected game, victory status, visible object identities and rotation cursor, sampled button histories, inactivity counters, and effect duration. Display caches are outside world state. A world initialization replaces only starting-world records and the specified player fields; it leaves display caches, input histories until resampling, inactivity counters, and other retained values intact.

`Room`, `ObjectId`, `RecordId`, `Game`, `DragonId`, and `GateId` identify closed domains. Coordinates remain bytes because wrapping differences and byte overflow affect gameplay. A position source is either a live record or immutable scenery. A state source selects a dragon, gate, game number, bat animation, or fixed value. Reaction pairs preserve their priority order. No reaction retains a creature's previous direction; a matching pair at equal coordinates supplies idle motion.

All core game code is in `src/adventure.rs`: world and display types, startup and picture scheduling, movement, interactions, actors, effects, rendering, and immutable tables. Tables are grouped into starting worlds, actor preferences, maps, artwork, and the object catalog within that file. The catalog's index order controls visible-object rotation. Room and object identity values remain stable because the map and rotation use them. Images terminate at zero; state range ceilings are inclusive. Wall artwork decodes to twenty boolean cells per band, each covering four pixels. Only normal and quadruple-width sprites occur in this game.

Hardware remains in four top-level modules: `hardware.rs` owns SDL presentation, event delivery, and pacing; `input.rs` defines controls; `sound.rs` owns audio controls and synthesis; `video.rs` composes scanlines and accumulates contacts. `lib.rs` exports those modules and `adventure`; `main.rs` handles the CLI. All tests, the test-only ROM oracle, and the explicitly configured SDL smoke binary are in `src/tests/`. Unit tests remain child modules of the implementation they exercise, allowing comparison of private state without widening the runtime API.


Startup, turns, and frame boundaries
-----------------------------------

1. `run(host)` clears world, audio controls, raster, display snapshot, and scheduling phase. It polls initial controls without presentation or pacing and initializes the selected world. Game 3 initialization randomizes records in reverse constraint order using the wrapping seed update, rejection sampling, and inclusive room bounds. It then presents the retained picture and draws with retained display information before clearing victory and held-object status. Host termination and errors unwind immediately.
2. The `Player` phase polls Reset/Select edges, advances effects once, and advances inactivity once. An already won game presents the previous picture, draws retained display state, and refreshes the display snapshot afterward; it stays in `Player`. An active game detects a chalice in the Gold Treasury, then still completes this three-picture turn even if that test sets victory. It moves the player on both axes and updates the held object before presentation. Returned controls become current controls. It selects display appearance and images, draws, processes pickup/drop from the resulting contacts, and advances to `BatAndGates`.
3. `BatAndGates` moves the player vertically and updates the dark-room light before presentation. After receiving input it updates bat cargo and gates, draws the retained display snapshot, then updates Grundle and Yorgle. It advances to `RemainingActors`.
4. `RemainingActors` presents first, then moves the player horizontally, updates Rhindle and the magnet, and draws retained display state. It returns to `Player`. `advance_turn` drives picture steps until the phase returns to `Player`; `advance_picture` exposes the individual scheduling phases.

Every presentation supplies the already completed framebuffer and current channel controls to `FrameHost::present`. SDL polls events, synthesizes and queues audio, converts indexed pixels to RGB, presents, and reaches the sole pacing point. Horizontal drawing never presents or sleeps. The paced host targets 60 frames per second, sleeping for the remaining frame interval and resetting its deadline if processing falls behind. `--unpaced` bypasses this pacing point. A bounded run or quit returns `None`; an SDL failure returns an error, propagated to the concise CLI error handler.

World initialization can introduce an additional presentation during Game 3 selection or reset. This is part of the preserved initialization workflow, rather than a fourth regular phase. Inputs sampled by a presentation affect subsequent work; changing the placement of a poll or update across that boundary changes behavior.


Display snapshot and scanline construction
-----------------------------------------

Display preparation resolves the current room's palette, wall arrangement, priority, player width, and fixed thin barriers. Inactivity transforms both fixed and cycling colors. It rotates through inhabitants to choose at most two visible identities; the second slot preferentially receives the light, then the bridge. Both slots resolve their position, image range, color, and width through the same catalog path. This snapshot is reused for the second and third pictures. The player position is read anew for each picture.

Drawing starts a fresh 194-line raster and clears contacts. It creates a complete `Scanline` containing appearance, wall cells, two object rows with positions/colors/widths, player x, and player visibility. Both initial object rows are blank; player visibility retains the preceding picture's final row. Wall band zero starts each picture.

For logical rows 104 down through 8 inclusive, drawing prepares the next second-object bitmap row except on the first iteration, prepares the first-object row, and computes player visibility with the wrapping four-row test. It paints line A with the preceding committed rows and visibility. It then commits both prepared object rows and player visibility; at row counts divisible by sixteen it advances to the next wall band. It paints line B with those complete values. Nonzero bitmap rows advance their cursors; a zero terminator keeps supplying blank rows. Six wall refreshes follow the initial band. Final player visibility persists; object rows always begin the next picture blank.

`Video::paint(&Scanline)` has no persistent drawable registers, row delays, positioning strobes, or hidden commits. It composes one line and accumulates geometric contacts. Wall mirror/repeat, object scaling, screen wrapping, player width, fixed barriers at screen x=11 and x=148 with width two, and layer priorities have the same functional behavior as the translation. Object and player coordinates in a `Scanline` are actual screen positions. Game display construction converts the cartridge coordinates to those positions: for ordinary coordinates, normal objects begin at game x−1, wide objects at game x, and the player at game x−2. It also handles the calibrated blanking/scanline-crossing mapping described in the project semantics. The hardware rasterizer applies no additional positioning shift. Walls and the player share foreground color; each barrier shares its object's color. Contacts are independent of the visible winning color and remain available until the next picture begins. Only the two selected slots generate object contacts.


World interactions and effects
------------------------------

Movement first applies independent directional steps with wrapping arithmetic, then resolves open-castle exits, top/left/bottom boundaries, and the right boundary including the secret passage. Player rollback uses the preceding picture's contacts and the saved position. Bridge traversal overrides blocking only during the horizontal phase within its wrapping corridor. The dot bypasses the thin right barrier only in the second visible slot; credits entry separately requires the dot to have left its hiding room.

Pickup uses the newly drawn picture and current fire edge. Scenery and dragons cannot be picked up. The first touching slot takes precedence. Dropping can be followed by pickup during the same handler. Carrying offsets are captured after the directional pickup displacement; attachment updates wrap coordinates and resolve world boundaries. The bat chooses cargo according to its ordered preferences, holds direction during committed flight, attaches cargo at its fixed offset, and steals the player's held identity when its live record matches the cargo.

Gates process black, white, then gold. Key contact advances gate state; entry is resolved before animation. Immutable scenery cannot be relocated through gate entry. Dragons preserve wandering, death, swallowed-player attachment, and biting countdown behavior, including difficulty-dependent sword avoidance and bite delay. The magnet chooses the first eligible attraction with its temporary wrapped target offset. Light placement is updated only in the designated phase and clamps its horizontal range.

Sound effects decrement duration before selecting a complete voice `(waveform, frequency, volume)`. `Sound::voice` applies the hardware's bit widths together without resetting synthesis phase. Silence zeros volumes while retaining oscillator state. Victory also updates the retained foreground color. Synthesis retains the translation's two-channel oscillator circuit, fractional clocks, mixing, DC removal, queued-audio bounds, and mute behavior; audio generation never advances gameplay effect counters.


Build and verification
----------------------

`make -C refactored all`, `release`, `run`, `check`, and `smoke` are available through corresponding root `refactored*` targets. Cargo tracks Rust dependencies and writes outputs to `target/refactored/`. Normal builds require SDL2 and pkg-config and do not require the assembler or a ROM fixture. The CLI supports the same controls, frame limit, unpaced runs, and BMP snapshots as the translation.

Checks run formatting, Clippy with warnings as errors, then tests. The Makefile reconstructs a test-only ROM from `original/adventure.asm` using the existing DASM tool rule and verifies its known SHA-256 before compiling the tests. ROM differential tests cover initialization including every Game 3 seed, wrapping boundaries, castle exits, bridge rollback, pickup, actors, gates, magnet, darkness, image ranges, inactivity, sound, and switch edges. The ROM kernel raster uses the preserved translation's compositor; the new compositor's pixels, line counts, and contacts are compared independently for every room. Additional compositor tests check hidden contacts and wide sprite seam geometry. A shared 160×193 Game 1 fixture captured from the cartridge in Stella 7.0 independently verifies the full picture, and recorded coordinate cases cover screen seams and positioning delays that land in horizontal blanking or another scanline.

Long trajectory tests compare exact presented pixels, line counts, audio controls, final contacts, and all public persistent records against `translation/`, with 400 turns for each of three games and three input seeds, including forced victory and subsequent switches. Boundary termination tests stop after different accepted frame counts. The headed SDL smoke runner starts play, retrieves the Gold Key, verifies pickup audio, mute/repeat behavior, and window sizes, writes `target/refactored-smoke.bmp`, and closes after 105 presentations.

`mostlyclean` removes refactor intermediates and test captures while preserving the release executable. `clean` also removes its deployment executable. Neither target changes source, configuration, saved data, or the preserved translation.
