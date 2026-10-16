Adventure ROM analysis
======================

This project preserves and analyzes the 4,096 bytes in `rom/adventure.bin`. `adventure.asm` is the preserved address-labeled baseline. `adventure-gameplay.asm` is a separate editable edition with gameplay names and narrative comments. Both editions' instructions, explicit data bytes, and vector words must assemble to the original ROM without any differences. The original ROM is an input and is never rewritten.

The original SHA-256 is `b0326b45e5cab066b2d03f1e683708d82de3ffe60b2e968bf88c5494237f0a96`.

The main Rust game is a functional port with explicit picture phases and whole-picture composition. Its gameplay and static data live in `src/adventure.rs`; validation independently executes the original cartridge in `src/rawemulator/`. The two assembly editions and ROM remain inputs to analysis and verification; compiling or running the game does not rewrite them.


Reconstruction workflow
-----------------------

1. Read the ROM at logical addresses `$F000–$FFFF`. Physical 6507 cartridge addresses are `$1000–$1FFF`; the high logical address bits are not connected. Read the three little-endian vector words at `$FFFA`, `$FFFC`, and `$FFFE`. All three contain `$F000`.
2. Decode official 6502 instructions from the vector targets. Ordinary instructions fall through, conditional branches have taken and untaken successors, direct jumps have only their target, and calls have a callee and a continuation. Returns end this first traversal. Reject overlapping instruction bytes, instructions outside the ROM, unsupported opcodes, and unresolved indirect control transfers. Do not decode unreachable bytes just because they resemble instructions.
3. Repeat traversal with an explicit tuple of outstanding JSR sites. A call pushes its site and enters its callee; an RTS pops the site and enters the instruction immediately after that JSR. Track visited `(PC, call-site tuple)` states to terminate loops. Branches retain both possible successors without attempting to solve register values. Require at most five outstanding calls and agreement with the first traversal's reachable instruction set.
4. Identify basic-block entries at vector targets, branch targets and fallthroughs, jump targets, call targets and continuations, and starts of disconnected reachable instruction ranges. Calls end blocks. Keep shared tails as shared instruction addresses rather than inventing separate copies or forcing every instruction into one exclusive routine.
5. Label code entries `L_xxxx` and noncode boundaries `D_xxxx`. Label additional timing addresses in the kernel. Preserve exact addressing modes, including absolute indexed reads whose base is below `$0100`. Emit noncode bytes explicitly and emit vector words pointing to the address-based code label. Do not use `incbin` to reproduce executable sections.

The supplied ground truths allow ordinary calls and returns, no overlapping instruction entries, and separate executable and graphics bytes. No computed jumps occur in this dump. The single TXS at `$F2FA` initializes SP to `$FF` before ordinary calls begin; it does not change a return target.

The graph records 1,236 instructions, 2,492 executable bytes, 355 basic blocks, 34 distinct JSR targets, and 4,581 instruction/call-context states. One maximum-depth path uses JSR sites `$F34B → $F7D9 → $F808 → $F715 → $F744`, entering `$F7CB → $F7EA → $F715 → $F728 → $F757`.

The remaining 1,604 bytes include data, six vector bytes, eleven trailing zero bytes, and the five unreachable startup-shaped bytes at `$F003–$F007`. Those five bytes decode independently as `SEI; CLD; JMP $F306`, but no reachable edge enters them. They remain explicit bytes with that observation in the source. Lack of reachability alone does not establish their original author's intent.


Reset and frame scheduling
--------------------------

The vector entry jumps to `$F2EF`, disables interrupts, clears decimal mode, clears TIA write registers `$04–$2C`, initializes SP, and clears RAM `$80–$FF`. It positions the missiles through `$F371`, initializes a selected game through `$F3D3`, and enters the scheduler at `$F306`.

Each scheduler pass processes switch edges through `$F384`, advances audio/effects through `$FA23`, and updates inactivity through `$F2B2`. When `$DE` is zero, it schedules three display phases. The phase index in Y changes which direction bits can move the player. Phase 0 samples the joystick, updates the held object, synchronizes the frame, prepares display pointers, displays, and handles player/object contact. Phase 1 synchronizes, updates the carrying actor and gates, displays, and updates two other actors during overscan. Phase 2 updates the remaining actors and displays before returning to `$F306`. When `$DE` is nonzero, the shorter path synchronizes, displays retained state, prepares display state, and returns to the scheduler.

Each display uses `$F008`. Before it, `$F0EC` waits for the prior overscan timer, asserts blanking, issues horizontal waits, asserts VSYNC for three lines, clears VSYNC, and starts the vertical-blank timer. Display preparation and game work occur while that timer counts down. `$F008` positions P0, P1, and the ball, applies HMOVE immediately after a horizontal wait, clears collision latches, waits for vertical-blank expiry, runs the kernel, blanks the display, clears sprite graphics, starts the overscan timer, and returns. Collision handlers consume the latches accumulated during the preceding display.

Switch processing can enter `$F3D3` by a branch or fallthrough as well as by JSR. Its eventual RTS returns to the active caller of `$F384` when entered through that routine. Likewise, `$FA9B` jumps to the shared audio tail at `$FA90` without adding a stack level. The call graph attributes nested calls to the outstanding call context, not to whichever named entry has the nearest lower address.


Display preparation and data lookup
-----------------------------------

RAM `$8A` holds the displayed room. `$F271` computes `$93/$94 = $FE1B + 9*A`. There are 31 nine-byte room records before `$FF32`. Each record contains a little-endian playfield pointer, two color encodings, CTRLPF, and four exit values. Tagged exit values are resolved by `$F6D5` through the eighteen bytes at `$FF32`, using the selected game. Direct room values pass through unchanged.

The object descriptor array begins at `$FF44` and contains nineteen nine-byte records through `$FFEE`. Descriptor offsets are multiples of nine. A record contains a position pointer, a state pointer, a graphics-selection pointer, two color encodings, and NUSIZ. Position records contain room/x/y and can reside in RAM or ROM. State values can also reside in either. Offset `$A2` supplies the empty-object sentinel used by display and interaction routines.

Graphics-selection tables contain three-byte records: an inclusive state ceiling followed by a little-endian bitmap pointer. `$F2A1` chooses the first ceiling greater than or equal to the state, leaving Y at that record's first byte. `$FF` terminates each selection table. Display preparation takes the pointer from the next two bytes. Several bitmap pointers refer to suffixes of the same stream; data labels must allow these shared interiors.

`$F235` scans descriptor offsets cyclically, retaining up to two objects whose room matches `$8A`. `$F10F` copies the selected room's playfield pointer to `$80/$81`, the two selected bitmap pointers to `$82/$83` and `$84/$85`, and their x/y coordinates to `$86/$87` and `$88/$89`. It also sets object colors and sizes, playfield behavior, and missile enable values. `$8B/$8C` supplies the ball position.

Game selection `$DD` uses encoded values `$00`, `$02`, and `$04`. The pointer words at `$F45A` select one of two 49-byte initial-state images; the latter two selections share the second image. Initialization copies the selected image backward into RAM `$A1–$D1`. The third selection additionally randomizes placements using the eleven constraint triples at `$F439`.

Actor search lists are zero-terminated pairs of RAM addresses. The selector walks the pairs, finds records sharing a room, and applies the exclusion at `$D6`. Position comparison produces active-low direction bits; movement adjusts x/y, then resolves gates and room-edge transitions. Lists occur at `$F7A7`, `$F7C2`, `$F7DD`, `$F927`, and `$F9DA`. These are data addresses, never indirect instruction targets.


Gameplay edition
----------------

The gameplay edition remains an independent assembly source, preserving all opcodes, addressing modes, instruction addresses, data bytes, shared tails, and vectors. It does not require a generator or an external include file. Named functions use PascalCase, local control targets use dot-prefixed names scoped with DASM `subroutine`, working memory uses lower-case names, and game constants use uppercase names. Every defined symbol has a named use; ordinary fallthroughs and return continuations have no labels merely for being block boundaries.

There are 155 distinct referenced code addresses and 156 code labels: `BeginFrame` and its local timer-loop target intentionally share an address, because callers use the function name and its branch uses the local name. The callable entries keep their own names. `RunGame` and the cartridge startup have program-level names, while `RememberConsoleSwitches` names the shared tail entered from both console polling and world initialization. Its name is used by a branch, and entering it never adds a call-stack level.

Gameplay names are attached to the traced records: `$A4` is Rhindle, `$A9` is Yorgle, `$AE` is Grundle, `$B3` is the magnet, `$B6` the sword, `$B9` the chalice, `$BC` the bridge, `$BF/$C2/$C5` the Gold/White/Black Keys, and `$CB` the bat. The dot record begins at `$A1`. Descriptor offsets `$36/$3F/$48` select Rhindle/Yorgle/Grundle, and `$09/$12/$1B` select the three castle gates. The code verifies these associations through their reaction lists, colors, positions, and gate-key pairings.

Room names describe recognizable locations and retain their original numeric IDs. The facades are `$11` Golden Castle, `$0F` White Castle, and `$10` Black Castle. Their interiors are `$12` Golden treasury, `$1A` White Castle interior, and `$1B` Black Castle interior. Other names identify the blue, red, and dark mazes and ordinary passages; suffixes on less distinctive passages preserve the room ID instead of claiming a unique official name. `$1E` is the Warren Robinett credits room.

The new presentation corrects several interpretations in the preserved baseline commentary:

*   Indexed operands `$00–$04,X` in object movement are record fields when X holds a RAM address. They do not access similarly numbered TIA registers. The new edition names room, x, y, direction, and state fields; startup memory clearing retains its literal base address.
*   `$F9AD` contains castle interior rooms and `$F9B0` contains the outside facade rooms. Entry through a gate uses `$F9AD`; leaving an interior uses `$F9B0`.
*   Display preparation prefers the second slot for the bridge or dark-room light, with the light taking precedence when both are selected. It tests object identities, not a vertical coordinate, at `$F158–$F166`.
*   The player passes the thin right wall when the dot occupies the second display slot. Separately, the secret passage's right-edge transition requires the dot to have left its hiding room. The latter predicate alone does not bypass the wall collision check.

The chalice wins when its room equals `$12`; the test does not depend on who carries it. RESET during active play restores the player to `$11`, revives dragons, stops the sound, and clears the held object without relocating all items. RESET after victory restores a starting world. SELECT outside the game-number room restores the selected world; SELECT in that room increments the chosen game.

Sprite previews beside `.byte` rows use `█` for a set bit and a space for a clear bit, with bit 7 on the left. Each emitted sprite byte has its own aligned preview, including its terminating zero. The gate variants point to suffixes of one shared bitmap rather than duplicating bytes or introducing unused interior labels. Room previews unpack PF0 bits 4–7, PF1 bits 7–0, and PF2 bits 0–7 into the left twenty wall cells, then mirror or repeat them according to the first room using that layout. Thin side barriers and sprite horizontal scaling are described separately. Shared bottom wall bands are repeated in comments only; their bytes remain emitted once.

The edition uses [Atari's Adventure instructions](https://atari.com/pages/adventure) for player-facing names and checks map terminology against the [Park/McFadden historical disassembly](https://6502disassembly.com/2600-adventure/Adventure.bin.html). Opcode decoding, data interpretation, new narrative comments, and glyph previews are derived from this project's ROM. The naming cross-check does not replace the independent control-flow analysis or byte comparison.


Kernel and synchronization timing
---------------------------------

Timing is in 6502 CPU cycles, with 76 cycles per scanline. Cycle zero is the resumption point after WSYNC. A cycle annotation identifies instruction completion relative to that point. WSYNC itself costs three cycles for the zero-page store, followed by the hardware wait. The timing model includes taken-branch and branch-page penalties. Indexed indirect reads normally cost five cycles and add one when the effective address crosses a page.

The kernel initializes `$8E` to `$68` and processes down through `$08` inclusive: 97 logical rows with two scanline phases each. `$8F` is the playfield byte offset; `$90/$91` are P0/P1 bitmap offsets. The initial playfield triple is read before the first visible-line boundary.

| Boundary | Resumption | Behavior |
|----------|------------|----------|
| `$F056`  | `$F058`    | Initial line A: clear VBLANK at cycle 5 and reach `$F072` at cycle 8, skipping the first P1 fetch. |
| `$F064`  | `$F066`    | Regular line A: conditionally fetch P1, prepare P0 and ball values, choose the next line-B path. |
| `$F097`  | `$F099`    | Refresh line B: write ENABL at cycle 3, GRP0 at 6, and PF0/PF1/PF2 at 17/27/37. |
| `$F0BD`  | `$F0BF`    | Ordinary line B: write ENABL at cycle 3 and GRP0 at 6; retain playfield registers. |

VDELP1 is enabled. The line-A GRP1 write prepares P1 graphics; the line-B GRP0 write transfers that value into P1's delayed graphics latch. Bitmap offsets advance only when the fetched byte is nonzero. A zero byte stays selected for the remainder of the display. All 24 bitmap starts selected by the ROM tables reach their first zero within their starting page, so the display fetches incur no page penalty.

Line A reaches its WSYNC store in 41–75 cycles on refresh rows or 42–76 on ordinary rows. These are conservative bounds over all branch choices; choices need not correspond to one feasible game state. Refresh line B reaches the next line-A WSYNC in 66 cycles, ordinary line B in 33. The playfield offsets read are 0–20: the initial triple plus refreshes at row counts `$60`, `$50`, `$40`, `$30`, `$20`, and `$10`. All referenced playfield streams retain these offsets within one page.

After the last row, decrement produces `$07`; storing it to VBLANK asserts bit 1. This occurs at cycle 24 of the actual ordinary final line B. A hypothetical refresh exit would blank at cycle 57. There is no final horizontal wait before kernel cleanup. The kernel consequently has 194 line phases, without a claim that its final phase is visible for a full line.

Timer waits occur at `$F02C` and `$F0EC`, each using `LDA INTIM; BNE` for seven cycles on a taken iteration. The vertical-blank timer is loaded with `$2A` at `$F10B`; the overscan timer is loaded with `$20` at `$F0CE`. Both use TIM64T, selecting a 64-cycle prescaler. These are nominal 2,688-cycle and 2,048-cycle countdowns; this analysis does not infer an exact complete-frame length from their loads alone.

The sync routine has seven horizontal waits at `$F0F3`, `$F0F7`, `$F0F9`, `$F0FB`, `$F0FF`, `$F101`, and `$F105`. VSYNC is asserted three cycles after the fourth boundary and cleared three cycles after the seventh boundary, exactly 228 CPU cycles later. Blanking remains enabled until the kernel clears it.

Object positioning waits horizontally at `$F0E2`. The DEY/BPL loop at `$F0E4` then consumes `5*Y+4` cycles for its initial Y, after which indexed writes select RESP0/RESP1/RESM0/RESM1/RESBL and their corresponding fine-motion register. This is a calibrated horizontal positioning delay, not a timer-polling wait. HMOVE boundaries also occur at `$F01F` and `$F37F`.

Hardware behavior follows the [Atari Stella Programmer's Guide](https://www.atariage.com/2600/programming/2600_101/docs/stella.html). Instruction and data boundaries come from this ROM and its traced operations; no published Adventure disassembly supplies the labels.


Build and verification workflow
-------------------------------

`make rom` assembles both sources with DASM format 3. The baseline produces `rom/build/adventure.bin`, `rom/build/adventure.lst`, and `rom/build/adventure.sym`. The gameplay edition produces the corresponding `rom/build/adventure-gameplay.*` files; `make gameplay` builds just that edition. Make also runs the ROM analyzer into `rom/build/control-flow.json`. Both assemblies are independent of that analyzer and contain no generated-source prerequisite. Editing comments or labels never requires regeneration from the ROM.

The default assembler is `rom/tools/dasm`. If absent, Make downloads and builds upstream DASM v2.20.17 locally, using `make -C` for its build. `DASM=/absolute/path/to/dasm` overrides the default. Compilation is serialized because one assembler invocation produces all three outputs. Each output, the source, assembler executable, and Makefile participate in dependency tracking. Removing any generated assembly output regenerates the group; successful incremental builds do not remove caches first.

The analyzer and verifier are executable dependency-free Python 3.9+ scripts. `uv sync` installs only the development formatting/type-checking tools recorded in `rom/uv.lock`. Type checking covers this project's `rom/tools/` sources; downloaded assembler research files are not project Python sources.

`make -C rom check` completes Ruff formatting checks, Ruff lint checks, and `uv run ty check` before invoking behavioral verification. Verification compares both editions with all ROM bytes. The baseline keeps its checks for basic-block labels, matched return contexts, graphics/code separation, graphics page containment, and scanline path timing. The gameplay edition additionally checks that DASM marks every symbol as referenced, its code-label addresses exactly cover the named control targets, and each of its 464 emitted graphical rows matches the underlying sprite bits or decoded room walls. Expected failures are reported without stack traces.

`make -C rom mostlyclean` removes both editions' assembly listings and symbols, analysis intermediates, Python bytecode, and downloaded assembler build intermediates, preserving both assembled ROMs and the local assembler executable. `make -C rom clean` additionally removes `rom/build/` and the downloaded assembler executable. Neither target modifies the original dump, either assembly source, documentation, Python sources, dependency configuration, or persistent user data.


Rust gameplay
=============

State and ownership
-------------------

`Adventure` owns `World`, current `Controls`, `Sound`, `Video`, retained contacts, a private `PreparedDisplay`, and a `Scheduler`. The prepared display owns selected object identities and its rotation cursor together with frozen geometry and appearance. Contacts are produced independently from that geometry at drawing operations; consumers associate them with the selected identities at the time of consumption. Preparation does not clear contacts, which matters when victory refreshes preparation after drawing. The scheduler owns the preceding picture's final player visibility and the next picture phase. `World` contains live entity records, safe player position, carrying state and offsets, gate and dragon states, selected game, victory status, sampled button histories, inactivity counters, and effect duration. A world initialization replaces only starting-world records and the specified player fields; it leaves display state, contacts, scheduler, input histories until resampling, inactivity counters, and other retained values intact. Power-on clears all of these resources.

`Room`, `RoomEdge`, `ObjectId`, `RecordId`, `Game`, `DragonId`, and `GateId` identify closed domains. Coordinates remain bytes because wrapping differences and byte overflow affect gameplay. Movable objects identify their world records independently of rendering; gates, credits, and the game number have fixed positions. `ObjectId` contains real identities only. An empty display slot is `None`; its display preparation explicitly uses the bridge position, a blank image, normal width, and the idle-transformed black color. It never acts as a bridge in gameplay queries. No eligible target retains an actor's previous direction; an eligible target at equal coordinates supplies idle motion.

`Carrying` combines a `HeldObject` and byte offsets. `HeldObject::Item` accepts only the nine `Carryable` identities: sword, bridge, the three keys, bat, dot, chalice, and magnet. `Empty` retains its offsets. `StartupLight` preserves the cleared startup identity until initialization clears it; its record is the light, so observations and an interrupted startup retain the original state. Bat cargo accepts a carryable item or a named dragon. Bat attachment resolves that identity to a live record and can clear the player's held item when both refer to that record.

`src/adventure.rs` defines world and retained state, startup, and picture scheduling. `src/adventure/rules.rs` defines movement, interactions, actor priorities, and effect updates. `original.rs` prepares the original visual profile, including coordinate calibration, bitmap termination, slot-specific row retention, and room definitions; `assets.rs` contains immutable wall and sprite data. Initialization assigns the world directly. Each room definition gives named north/east/south/west exits, artwork, palettes, darkness, mirroring, and barriers. Exits are fixed rooms or an explicit Game 1/later-games choice. Castle openings and conditional credits entry remain ordered movement rules. Darkness controls both lighting and foreground priority. Object identity order controls display rotation. Original images terminate at zero; generic prepared sprite rows permit blank interior rows. Wall artwork decodes to twenty boolean cells per band, each covering four pixels. Only normal and quadruple-width sprites occur in the original profile.

`src/controls.rs` provides typed logical input reads. Each read returns a boolean, direction, or difficulty and retains the consumed value in current controls. Live play consumes host values; replay supplies the recorded value at that same logical read. `src/observation.rs` converts core identities and state into optional validation observations. These two adapters are private child modules of `adventure` with files directly under `src/`. Blocking, contact selection, actor priorities, movement, carrying, and state transitions belong to the core. Observation methods never change world state or return gameplay decisions. Recording leaves live inputs unchanged.

`runtime.rs` defines the host-independent `FrameHost` boundary and owns optional recording and tracing services through `Adventure::session`. `set_trace` replaces a trace and removes any prior validator; `enable_validation` pairs the installed trace with its oracle in one private enum variant. Validation cannot exist without its trace. Catch-up and completion use that owned trace's clock, and completion flushes recording and trace endpoints before finishing the oracle. `hardware.rs` owns SDL presentation, event delivery, and pacing and reexports the neutral host trait. `input.rs` defines controls; `sound.rs` owns audio controls and synthesis; `video.rs` prepares geometry, detects contacts, and composes pictures through separate operations. The private `streaming_audio.rs` owns the persistent synthesizer and PCM queue. Unit tests, the test-only ROM oracle, and the SDL smoke example are in `src/tests/`; integration tests are in `tests/`.

Cargo features `sdl` and `validation` are enabled by default. Disabling defaults builds the game library, headless host, geometry, synthesis, and recording with neither SDL nor raw-emulator dependencies. The interactive CLI and SDL examples require both features; the Python reference exporter requires neither. `make lint CARGO_FEATURES=--no-default-features` and `make test TEST_PACKAGES='-p adventure' CARGO_FEATURES=--no-default-features` exercise the portable configuration.


Startup, turns, and frame boundaries
------------------------------------

1. `run(host)` clears world, audio controls, raster, display snapshot, and scheduling phase. It polls initial controls without presentation or pacing and initializes the selected world. Game 3 initialization randomizes records in the specified placement order using the wrapping seed update, rejection sampling, and inclusive room bounds. It then presents the retained picture and draws with retained display information before clearing victory and held-object status. Host termination and errors unwind immediately.
2. The `Player` phase polls Reset/Select edges, advances effects once, and advances inactivity once. An already won game presents the previous picture, draws retained display state, and refreshes the display snapshot afterward; it stays in `Player`. An active game detects a chalice in the Gold Treasury, then still completes this three-picture turn even if that test sets victory. It moves the player on both axes and updates the held object before presentation. Returned controls become current controls. It selects display appearance and images, draws, processes pickup/drop from the resulting contacts, and advances to `BatAndGates`.
3. `BatAndGates` moves the player vertically and updates the dark-room light before presentation. After receiving input it updates bat cargo and gates, draws the retained display snapshot, then updates Grundle and Yorgle. It advances to `RemainingActors`.
4. `RemainingActors` presents first, then moves the player horizontally, updates Rhindle and the magnet, and draws retained display state. It returns to `Player`. `advance_turn` drives picture steps until the phase returns to `Player`; `advance_picture` exposes the individual scheduling phases.

Every presentation supplies the already completed framebuffer and current channel controls to `FrameHost::present`. SDL polls events, generates queued PCM, converts changed indexed rows to texture pixels, redraws when needed, and reaches the sole pacing point. Horizontal drawing never presents or sleeps. Accepted frame boundaries advance the frame count and pacing even when the window does not need repainting. The paced host targets 60 frames per second, sleeping for the remaining frame interval and resetting its deadline if processing falls behind. `--unpaced` bypasses this pacing point. A bounded run or quit returns `None`; an SDL failure returns an error, propagated to the concise CLI error handler.

SDL uses its default renderer selection, preferring acceleration while permitting software fallback. It does not request renderer vsync; the explicit host pacing remains authoritative. SDL continues to scale the texture to the logical 320×194 display and letterbox arbitrary window sizes. `TexturePixels` chooses the first advertised ARGB8888, ABGR8888, RGBA8888, BGRA8888, RGB888, or BGR888 format, falling back to the original RGB24 when none is advertised. Its precomputed 128-entry palette uses SDL's color mapping and native byte order for packed formats, with opaque alpha. The complete retained upload buffer uses that format's byte stride. Snapshot export converts it to RGB24 on demand, preserving the established 24-bit BMP output.

SDL retains a streaming texture and the last successfully uploaded indexed picture. The first presentation converts and uploads every row, including an all-black initial picture. Later presentations compare complete indexed rows, convert changed rows, and upload contiguous changed spans with the full upload row pitch. The indexed comparison cache commits only after all uploads succeed, so a failed partial upload can be retried. `DisplayRefresh` preserves redraw requests across control sampling. Changed pictures, exposure, showing, restoration, maximization, display changes, resize events, and size shortcuts force a complete clear/copy/present of the transient window backbuffer. An unchanged picture with no refresh request skips those rendering calls while input, audio synthesis, frame counting, and pacing continue. Render-target resets force a redraw; device resets recreate the streaming texture and invalidate the indexed cache for a full upload. A recreation request dominates later redraw events until handled, and a failed window copy retains its redraw request.

SDL's `unsafe_textures` feature supplies canvas-owned textures without a self-referential Rust lifetime: each private texture is used solely with its creating canvas. Replaced texture handles remain renderer-owned, and SDL releases all texture allocations when the renderer is destroyed. No manual destruction or application unsafe code is used.

World initialization can introduce an additional presentation during Game 3 selection or reset. This is part of the preserved initialization workflow, rather than a fourth regular phase. Inputs sampled by a presentation affect subsequent work; changing the placement of a poll or update across that boundary changes behavior.

Initialization places Game 1's actors and items in its fixed layout, or uses the common Games 2/3 layout. It closes all gates, sets all dragons to wandering, and clears bat animation, cargo, and commitment. Game 1 starts actors with all four directions set, so opposing steps cancel; later games start dragons moving left/up and the bat moving left/down. Player and safe-player room become Game Selection and y becomes zero, while their x coordinates remain live. Light, carrying offsets, counters, sound duration/effect and channel controls, display snapshot, and scheduling phase remain live. Switch histories are resampled at the initialization tail; fire history is retained.

Game 3 places magnet, bat, black key, white key, gold key, bridge, sword, Grundle, Yorgle, Rhindle, then chalice. For each attempt the seed becomes `(seed >> 5) + seed + 1` with byte wrapping; its low five bits select a room identity. Most objects accept West Hall through Red Hall inclusively; the black key ends at Gold Treasury, the white key at Black Maze Entrance, and the chalice accepts Black Maze 13 through White Castle Interior. Rejected candidates still advance the seed. Coordinates and the final seed remain live.


Display snapshot and frame construction
---------------------------------------

The cursor scans the eighteen real identities cyclically, stopping after selecting two room inhabitants or returning to its starting identity; it retains the stopping identity for the next refresh. The light prefers the second slot; the bridge prefers that slot unless it contains the light. Empty slots never enter the rotation. Appearance, two selected object identities, their artwork, positions, colors, and widths are retained for the second and third pictures; the player position is read again for each picture. Only selected slots contribute object contacts. Darkness controls foreground priority, and inactivity transforms colors.

Display preparation independently expands the selected wall bands and each object's bitmap into complete 194-row layers. Each object snapshot contains only its calibrated screen position, color, width, and complete bitmap rows; drawing does not reread live object positions or state. `draw_room` combines these cached layers with the current player x and 194 freshly computed player visibility values into one complete `Picture`, prepares its geometry, updates the game's retained contacts, then calls `Video::compose`. Only the final player visibility value survives the drawing operation separately in scheduler state. Gameplay never reads contacts from the renderer.

Logical rows descend from 104 through 8. Each emits the preceding artwork or player visibility followed by the current value. Both object images start blank, while the preceding picture's final player visibility persists into the first row. The second image skips its first fetch and retains its last fetched bitmap value if the wrapping vertical comparison subsequently becomes ineligible; the first image supplies zero on an ineligible row. Nonzero image rows advance their cursors; zero terminators stop them. Wall bands advance on logical rows divisible by sixteen before the second physical row. The independent expansion functions retain these geometries without interleaving wall, player, and sprite preparation or painting during preparation.

The renderer fills background, then paints complete wall/player coverage, then the complete second object and its barrier, then the complete first object and its barrier. In rooms with foreground priority, object colors cannot overwrite walls or the player. Otherwise the first slot wins over the second and both win over foreground. Each barrier uses its associated object's color. Barriers occupy x=11 and x=148, width two, across the entire picture. Wall cells cover four pixels and repeat or mirror on the right half. Sprite pixels scale by one or four and wrap at the 160-pixel screen seam; player width and wrapping remain unchanged. Coordinate calibration follows the cartridge positioning routine.

`Geometry::prepare` expands a complete picture into independent wall, player, object, and barrier masks, using three 64-bit words per row per layer. `Geometry::contacts` reduces intersections without painting or inspecting colors and priority. Barriers do not count as object bitmap contacts. `Video::compose` consumes geometry and appearance without modifying contacts, so replacement visual geometry can coexist with the original collision geometry. `Video::paint` provides the original combined workflow: prepare geometry, replace contacts, then compose. Gameplay consumes contacts at its established operation boundaries. `painted_lines` becomes 194 after composition; presentation, input sampling, effects, audio, and the three-picture gameplay turn keep their existing boundaries.

Within geometry preparation, consecutive identical wall rows reuse their expanded mask. The player covers at most two horizontal spans split at the screen seam. Widths at least 160 cover the whole screen once, and byte coordinates normalize modulo 160. Each object expands only set artwork bits at its selected scale. Enabled barriers cover two pixels on every row. Composition fills the background, paints foreground coverage, then the second and first object/barrier masks; foreground priority subtracts foreground coverage from those masks. Mask iteration visits only set pixels.

`Video` retains the last complete immutable `Picture` and its contacts. Equal picture values reuse pixels and restore contacts without composition; equality includes appearance, every wall and sprite row, calibrated positions, widths, player visibility, and barriers. Changed pictures prepare independent geometry and contacts, then compare each complete row's masks, colors, and priority with retained composition state. Only changed rows are recomposed, including old locations erased by motion. `Frame::changed_rows` contains contiguous recomposed spans for the latest paint; hidden geometry changes may yield a span without changing visible colors. Storage is reused across pictures. SDL continues comparing against its last successfully uploaded pixels, independently of this latest-paint metadata, because presentations and compositions need not be one-to-one and device resets require a complete upload.


World interactions and effects
------------------------------

Movement first applies independent directional steps with wrapping arithmetic, then resolves open-castle exits, top/left/bottom boundaries, and the right boundary including the secret passage. Player rollback uses the preceding picture's contacts and the saved position. Blocking queries walls, left barrier, right barrier, first object slot, then second object slot, stopping at the first blocking result. These ordered queries also determine which positive contacts are observed during validation. The light does not block. Bridge traversal overrides blocking only during the horizontal phase within its wrapping corridor. The dot bypasses the thin right barrier only in the second visible slot; credits entry separately requires the dot to have left its hiding room.

Pickup uses the newly drawn picture and current fire edge. Scenery and dragons cannot be picked up. The first touching slot takes precedence. Dropping can be followed by pickup during the same handler. Carrying offsets are captured after the directional pickup displacement; attachment updates wrap coordinates and resolve world boundaries. The bat chooses cargo according to its ordered preferences, holds direction during committed flight, attaches cargo at its fixed offset, and steals the player's held identity when its live record matches the cargo.

Gates process black, white, then gold. Key contact advances gate state; entry is resolved before animation. Immutable scenery cannot be relocated through gate entry. Gates explicitly distinguish Startup, Open, Closed, Closing, and Opening. Private progress values represent remaining closing updates or elapsed opening updates. Key contact advances progress before animation; opening normally completes after elapsed update 27, but a key contact at that endpoint skips completion and continues through the extended wrapping sequence. Startup advances to Open. The original byte encoding is exposed only for artwork and validation conversion. Dragon bites use a positive remaining-update count: decrement first, return to wandering at zero, otherwise capture on contact during the final four updates. During a wandering update sword contact is processed after player contact and can override a newly started bite. The magnet chooses the first eligible attraction toward a local position eight wrapping y units below its live position; calculating this target never changes the magnet record. Light placement is updated only in the designated phase and clamps its horizontal range.

Wandering dragons first flee the sword when right difficulty is A; Yorgle next flees the Gold Key regardless of difficulty. Rhindle then pursues player, chalice, white key; Yorgle pursues player, chalice; Grundle pursues player, chalice, bridge, magnet, black key. Only objects in the dragon's room are eligible, and the first eligible rule wins. Fleeing compares coordinates from the feared object toward the dragon, independently on each axis. Rhindle moves three steps; the other dragons move two. Left difficulty B/A starts Game 1's bite at 48/24 remaining updates and later games at 16/10. The observation adapter converts these to the original increasing-byte counters.

An uncommitted bat pursues chalice, sword, bridge, gold key, white key, black key, Rhindle, Yorgle, Grundle, then magnet, excluding its cargo. It moves three steps before checking a wrapping square reach of -4 through +3 on both axes; crossing a room boundary can prevent pickup. Pickup starts 240 committed updates. The optional positive remaining count decrements before motion, and even the final committed update retains the old heading; target selection resumes on the next update. A magnet attracts gold key, white key, black key, sword, bridge, then chalice, moving only the first eligible object one step. A target already aligned with either actor still takes precedence over later candidates.

Sound effects decrement duration before selecting a complete voice `(waveform, frequency, volume)`. The six typed effects define gameplay register writes with the original widths and counters; victory also updates the retained foreground color. Effects normally advance once per three-picture turn, or once per picture after victory. Event handlers select the next effect and duration; channel controls change at the subsequent envelope update. Silence zeros channel volumes. World initialization retains existing sound state; active-play Reset clears duration and silences sound at the usual update boundary.

`SdlHardware::new` opens the streaming output directly. `streaming_audio.rs` owns a persistent two-channel synthesizer and SDL mono float queue. It requests 48 kHz and a 512-sample device buffer, then uses the obtained rate. Each accepted picture accumulates the rate, generates the quotient by sixty, and retains the fractional remainder. Both oscillators, fractional native clocks, and DC-filter state persist through all effects, silence, reset, and mute. No effect boundary reseeds phase. Generation follows simulation pictures. Before delivery, a backlog above three pictures is cleared. Muting immediately clears and pauses delivery at the next presentation while synthesis continues into discarded samples. Unmuting resumes with the current controls and inherited phase. Frame-limit exhaustion returns before generating more samples. Audio playback never advances gameplay effect counters.


Compact input recording
-----------------------

`--record FILE` writes a compact boot-to-exit input recording; `--replay FILE` starts a fresh game and injects recorded values at logical input reads. `src/recording.rs` owns this independent session, attached through `Adventure::session.recording`. No world observations, collision records, raster, sound, or state validation participate. Recording can run alongside `--trace`; playback and verification cannot be combined with live-play options or each other.

The UTF-8 format starts with `adventure-input 1`. Each changed consumed value is written as `input READ CONTROL VALUE`, where READ is the zero-based count of all logical gameplay input reads since boot, CONTROL uses the trace control vocabulary, and VALUE uses the trace bit encoding. All reads advance this clock, including unchanged values and initialization reads. Each control initially has its normal default (color 1, everything else 0); initial default values and later unchanged values are omitted. Each saved value replaces that control's retained value until its next change. Host events that gameplay never reads are absent. This timing is independent of trace observations and their ordinals; the format is not a literal trace subset.

Normal termination writes `end FRAMES READS` and flushes the writer. FRAMES counts accepted presentations, including Game 3 initialization presentations; READS distinguishes cancellation before the first logical read from initialization followed by stopping before any presentation. Reads made before the final rejected presentation are part of the recording. Recording errors propagate through the run and CLI. Abnormal termination can leave an incomplete file, which playback rejects.

Playback parses the version, control domains, strictly increasing read indices, and explicit final endpoint before opening SDL. It clears game state through the ordinary boot path, ignores live gameplay values, and applies input changes when the read clock reaches their indices. The presentation wrapper stops before displaying picture FRAMES + 1; a recording canceled before its first read skips initialization. Mute, resizing, and quit remain live. Early quit succeeds without verification. Playback does not compare world state, output, consumed read counts, or remaining changes. Recordings depend on the implementation's logical read schedule; they are not a cross-version compatibility or validation contract.


Adventure traces
================

`adventure-trace` is a dependency-free library shared by the main game and original-cartridge validator. It establishes equivalence between their observed mechanics. It does not choose actor reactions, perform collision geometry, or decide whether a pickup, victory, or gate entry should occur. The validator executes the original instruction stream and independently produces the observations used as the correctness reference.


Recording and time
------------------

A session starts at the cartridge's boot state and emits `trace 1 adventure-original boot`. Every successful frame boundary advances a nominal 60 Hz clock by one picture, including the additional Game 3 initialization picture. A three-picture turn is not one clock tick. Host pacing, audio generation, and elapsed wall time do not advance the clock.

A moment serializes as `MM:SS:FF/point#ordinal`, where minutes are unbounded, seconds and the picture fraction are 00–59, and ordinal starts at zero independently for each point in each picture. The ordinal counts emitted records rather than raw observations. Semantic points preserve the call order around picture boundaries: console, history, inactivity, the three player movement passes, held-object update, display preparation, pickup, bat, gates, individual dragons, magnet, and light. Point names are not globally sorted; their order is their order in the stream.

Initialization emits `init 1`, `init 2`, or `init 3`, then observes the copied world, any randomized placements, and the initialization tail. Retained fields remain live. A world reset does not restart the clock. An explicit `end` at the accepted picture count terminates the run. Original-cartridge validation stops at the next attempted frame boundary after that count, after checking preceding work; the cartridge itself never halts. A boot header followed immediately by a zero-picture terminator represents cancellation before the initial control sample and replays without world initialization.

Recording can retain records in memory for tests or write incrementally to an owned writer. Writer failures are reported, and normal completion flushes the terminator. Replay checks the header, terminator, nondecreasing frames, and per-point ordinals before execution. Events after the terminator are rejected.


Text records
------------

The grammar uses closed lowercase names, decimal byte coordinates, whitespace-separated tokens, and no escaping. Examples:

    00:00:00/boot#0 trace 1 adventure-original boot
    00:00:00/init#0 init 1
    00:00:00/init#1 place bat white-castle-interior 32,32
    00:00:02/bat#0 motion bat below-gold-castle 29,35 step=-3,3 period=3
    00:00:05/bat#0 state bat carry sword 8,0
    00:00:08/bat#0 enter bat west-hall 3,44
    00:00:08/bat#1 enter sword west-hall 11,44
    00:00:11/bat#0 stop bat west-hall 12,53
    00:00:12/console#0 input reset 0
    00:00:12/player-both#0 contact player walls
    00:00:13/gates#0 state gold-gate mode opening
    00:00:13/gates#1 state gold-gate counter gate 29
    00:00:40/gates#0 state gold-gate mode open
    00:01:45/end#0 end

These lines illustrate independent record forms, rather than one executable trace. `place` provides an initial or discontinuous complete position. `enter` always gives the destination position, including a crossing whose exit returns to the same room. `motion` anchors the current position and supplies signed byte displacement and a positive picture period; its semantic point identifies the operation that advances that motion. `stop` supplies its final position.

State forms are `mode NAME`, `counter NAME VALUE`, `carry OBJECT|none X,Y`, `direction BITS`, `game 1|2|3`, and `won 0|1`. Carry offsets are wrapping bytes. Direction bits are right=1, left=2, up=4, down=8, so opposing directions remain representable. Input booleans use 0/1, difficulties use B=0/A=1, and joystick directions use those same four bits. Unknown names, unsupported versions, extra tokens, invalid clocks, out-of-range coordinates, and invalid input values produce parse errors.


Observation and normalization
-----------------------------

Adapters observe all fourteen movable records plus the saved player position, dragon modes/directions/bite counters, gate modes/progress, player and bat attachments, bat direction/animation/flight commitment, selected game, victory, and inactivity counters. Fixed scenery positions are defined by the immutable cartridge profile. Palette, sound synthesis, framebuffer pixels, scanlines, and temporary lookup pointers are outside this trace contract.

The ports commit complete state after each scheduled game operation. `PositionAssembly` lets a lower-level implementation write room, x, and y separately. At the same commit boundaries it supplies complete object observations in the declared profile order. A partially initialized position cannot be committed. Temporary stores that are restored within an operation, such as the magnet's temporary target offset, do not become placements. Boundary crossings are explicit observations at their completed coordinate stores and are retained even if the final room identity is unchanged.

The machine tracks each object's last committed position and owning operation. Unchanged observations from unrelated operations are redundant. At the owning operation, equal displacement and equal elapsed picture count establish a motion segment. The first displacement emits an anchor; subsequent matching samples are checked and omitted. A different displacement, cadence, stop, operation, discontinuity, or room crossing produces a record. A crossing re-anchors an established segment at its destination while retaining its displacement and cadence until a subsequent sample changes them. Duplicate unchanged samples in the same picture do not stop motion. Byte differences use wrapping arithmetic. A carried object's movement remains independently checked, and a bat boundary crossing emits a separate cargo crossing even on a self-looping room exit.

State changes are keyed by their typed aspect. Counter progression is omitted only after establishing an equal update cadence and checking the next expected value. Bat animation wraps at eight; other active counters wrap as bytes. Terminal dragon and gate modes determine their inactive progress value of zero. Positive gate/key contact adds one to that gate's normal increment; repeated reads of the same latched pair do not add multiple increments. Unexpected progress, stalls, timing changes, and flight completion remain explicit. Gate states and counter endpoints are checked rather than supplied by the recording.

Each adapter must provide the same complete commit coverage. More component writes or duplicate unchanged samples are acceptable; missing semantic commits, intermediate room crossings, and consumed positive collisions are not interchangeable with silence. Equivalence concerns this observed contract, not arbitrary private implementation state or exact rendering output.


Input and collision replay
--------------------------

Host event polling updates available controls. It does not itself create input records. A semantic read emits a field's first consumed value and subsequent changes from its last consumed value, even if the change causes no action. Last values are tracked independently per control across semantic points, pictures, and world initialization. Unchanged reads emit nothing and consume no ordinal. Inactivity reads both joystick directions, then reads console buttons only when both joysticks are idle. Console processing reads Select only after the Reset path permits it. Fire is read at pickup/drop; difficulty reads follow the dragon branches that use them. Display preparation samples the consumed color choice once; later palette reads within that preparation use that value. Window controls, mute, quit, and right-fire are not game inputs in this contract.

During replay the expected input record supplies its value at the logical read only when timestamp, point, control identity, ordinal, and stream position match. Other reads reuse that control's last consumed value; before its first record they use the caller's live value. Supplying a value does not execute the action it may trigger. The game or cartridge independently generates resulting state and collision observations. Missing or mistimed change records fail comparison; unchanged reads are not checked individually. A change within one point and picture is applied at the first matching read after preceding emitted records, since omitted reads have no recorded ordinal. Existing traces containing repeated unchanged input records must be recorded again.

Only a consumed positive collision bit produces a `contact`. Player movement respects the original short-circuit register-read order. Pickup checks its second slot only when the first does not touch. Actor and gate helpers record their positive reads even when their queried object is absent from the display pair. Contacts name the slot identities the game uses when it consumes the latches, including retained display state. They do not infer participants from current room positions; invisible objects cannot gain contacts from mere room proximity. Negative reads and unread positive latches produce no records. Collision clearing belongs to drawing and does not emit a record.

Replay normalizes actual observations with the same machine and compares emitted records in order. The first difference stops validation and reports expected versus observed text. A missing final record or remaining expected events also fails. Recorded state changes and collision answers are never injected.


Python runtime flow
===================

`make -C py run` uses `uv sync --locked`, then runs `main.py` in the project environment. `Host` initializes SDL video and mono signed-16-bit audio through pygame-ce. The game starts with color enabled, both difficulties at B, a 960×582 resizable window, and unpressed controls. Expected startup failures and unexpected exceptions produce concise stderr messages and unsuccessful exits. The host closes SDL resources on exit.


World state and initialization
------------------------------

`Adventure` owns `World`, `Controls`, `Video`, `Sound`, a retained `Display`, and a `Phase`. Movable positions are immutable `Position(room, x, y)` values in a dictionary keyed by `RecordId`. Actors keep directions and states separately. `ObjectId` determines display rotation and maps to a movable record when applicable; scenery has fixed positions. `ObjectId` contains only real identities. Empty visible slots and empty hands use `None`; only display preparation resolves an empty slot to the bridge position, blank artwork, normal width, and idle-transformed black color. Every coordinate change explicitly wraps to a byte before signed comparisons.

Each `run` clears world, sound, video caches, prepared display, retained player visibility, and scheduling phase before accepting the initially sampled controls. Initial control sampling occurs before world initialization and without presentation. Initialization retains player and safe-player x, display state, light, counters, fire history, carrying offsets, and sound state. It sets player and safe-player room to Game Selection and y to zero, places objects for the selected game, closes gates, revives dragons, and clears bat animation, cargo, and flight commitment. Game 1 directions cancel on both axes; later games begin with committed headings. Switch histories are sampled at the tail, victory is cleared, and the held identity becomes `None`.

Game 3 uses Game 2 coordinates and relocates magnet, bat, black key, white key, gold key, bridge, sword, Grundle, Yorgle, Rhindle, then chalice. Each rejection-sampling attempt changes the seed to `((seed >> 5) + seed + 1) & 255` and uses its low five bits. Bounds are inclusive: most records accept West Hall through Red Hall, black key ends at Gold Treasury, white key ends at Black Maze Entrance, and chalice accepts Black Maze 13 through White Castle Interior. It presents and draws retained display state before the initialization tail.


Picture scheduling and input
----------------------------

The Player phase polls console edges, advances the sound envelope and inactivity counters, then checks victory. Reset during play restores the player to Gold Castle, revives dragons, clears the held object, and stops the effect at its next update. Reset after victory initializes the world. Select initializes the current world and advances the selected game only from Game Selection. Inactivity advances once per turn, supplies cycling colors and the random placement seed, and resets its period counter on either joystick movement or console-button activity.

An already won Player phase presents, draws retained display state, refreshes the display snapshot, and stays in Player. Otherwise a chalice in Gold Treasury sets victory but still completes the active three-picture turn. Player movement on both axes and held-object attachment precede presentation; display preparation, drawing, and pickup/drop follow it. BatAndGates retries vertical player movement and updates the dark-room light before presentation; bat and gates update afterward, followed by drawing, Grundle, and Yorgle. RemainingActors presents first, retries horizontal movement, updates Rhindle and magnet, draws, and returns to Player.

Each presentation polls events and applies sound commands. Changed indexed pixels replace the retained paletted image. A changed picture or scaled size rebuilds the scaled image; unchanged pictures retain it. Picture changes, exposure, showing/restoring/maximizing, size/display changes, and renderer-reset events request redraw. A redraw clears the window to black and letterboxes the scaled picture to a 320:194 display aspect ratio before flipping. Idle boundaries still poll input, apply sound, and pace at 60 Hz. Returned controls affect subsequent work. Window shortcuts resize the existing window. Held keys drive joystick and momentary console buttons; color and difficulty switches persist. Focus loss clears held keys. M toggles once per key press. Quit returns immediately through the game loop and closes the host.


Movement and interactions
-------------------------

Movement rolls back to the safe player position using the preceding raster's wall, barrier, and object contacts. Light does not block movement; the dot bypasses the right barrier only in the second slot. The bridge overrides blocking only on the horizontal retry while unheld and within its wrapping corridor. Independent direction steps wrap coordinates. Open-castle exits precede north, west, south, and east edges. The separate secret east edge requires the dot to have left its hiding room. `rooms.py` supplies ordinary exits and game-dependent overrides.

Pickup/drop uses newly drawn contacts. A fire edge can drop and then immediately pick up. The first touching slot wins; scenery and dragons are excluded. Pickup displaces the item along the stored joystick direction, then records its wrapping offset. Attachment reapplies that offset and resolves boundaries without another directional step. Gates process black, white, then gold: key contact advances state, player/item entry resolves, and moving states advance. Magnet computes a local target eight wrapping y units below its position without mutating its live record and moves the first eligible gold key, white key, black key, sword, bridge, or chalice one step. An aligned first target still wins.

Dragons flee the sword at right difficulty A; Yorgle also flees the Gold Key. Pursuit order is player/chalice/white key for Rhindle, player/chalice for Yorgle, and player/chalice/bridge/magnet/black key for Grundle. Only same-room targets qualify; no target retains the old heading. Rhindle moves three steps, others two. Player contact starts the difficulty-dependent bite countdown, then sword contact may kill the dragon in that update. Countdown overflow restarts wandering; late contact swallows the player. A swallowing dragon attaches the player at its fixed offset.

The uncommitted bat pursues chalice, sword, bridge, keys in gold/white/black order, dragons in Rhindle/Yorgle/Grundle order, then magnet, excluding its cargo. It moves three steps before checking wrapping proximity. Pickup starts its flight commitment; while committed it retains its heading and increments the byte timer until overflow. Cargo attaches eight pixels to its right and steals the player's held identity when they refer to the same record.


Display and sound
-----------------

Display preparation rotates through the eighteen real object identities and keeps up to two inhabitants. The light preferentially occupies slot two, followed by the bridge. It expands wall bands and each selected sprite into independent complete 194-row layers, capturing calibrated positions, widths, colors, and priority; the following pictures reuse that snapshot. Dark rooms share a palette, foreground priority, and clamped player-centered light. Idle color transformation applies after fixed/cycling color choice. Only the bridge and light use quadruple-width sprites.

`wall_rows` expands seven wall bands independently; each logical row contributes the preceding band followed by the current band, advancing on multiples of sixteen. `sprite_rows` expands each bitmap separately into 194 rows, starting blank. The first physical row uses the preceding bits and the second uses the newly fetched bits. Nonzero bits advance the image cursor and a zero terminator stops it. The second slot skips its first fetch and retains bits across a wrapping eligibility boundary; the first slot clears ineligible bits.

`draw_room` combines these frozen layers with freshly calculated player rows and calibrated x in an immutable `Picture`. The first physical player row uses the preceding picture's final visibility; the second uses the current four-logical-row visibility test. Only the final visibility survives independently of the prepared display. Drawing never rereads live object positions or emits scanlines to the renderer.

`Video.paint(Picture)` fills the background, paints the complete wall/player foreground, then the complete second sprite/barrier and complete first sprite/barrier. Foreground priority prevents sprites and barriers overwriting walls/player. Integer row masks retain each layer's geometry separately; player/wall, player/barrier, player/sprite, and sprite/sprite contacts come from these masks independently of winning colors. Only the selected slots contribute object contacts. Sprite scale is one or four; positions wrap at 160 pixels. Barriers remain at x=11 and x=148, width two. Player width defaults to four; zero and full-screen widths retain their natural coverage. Indexed pixels retain all eight color bits; the host palette ignores bit zero.

The renderer retains the last immutable `Picture` and immutable `Contacts`. Equality includes every prepared row, appearance, position, width, and player visibility; an identical picture restores its contacts and reuses pixels. Hidden geometry changes invalidate retention even when visible colors remain identical. Changed pictures replace the full raster and contacts. Within each layer, repeated row masks share their expanded horizontal pixel offsets.

Events select a pending sound effect and duration. Envelope advancement decrements duration before publishing a separate active effect/remaining/cadence command; pending events cannot alter a previously published voice. Normal cadence is once per three pictures; victory uses every picture. Victory also updates the retained room foreground. The six envelopes and register masks match the Rust game. Synthesis integrates the native NTSC clock through persistent divider/noise/pulse feedback latches and a 20 Hz DC-removal filter, then stores signed-16-bit PCM.

The host lazily caches complete effect/cadence clips, with offsets per envelope step and a tapered 25 ms release. Consecutive commands continue playback; interruptions, restarts, and unmute seek the current step. pygame applies a two-millisecond fade-in on clip starts and fade-out on silence/mute. Playback timing belongs to the audio device; gameplay alone advances effect counters. The pygame host uses signed-16-bit clips and pygame fade transitions rather than the Rust callback's exact interruption ramps and floating-point output. Sound clips are generated lazily and rendering retains complete indexed/scaled surfaces rather than Rust's native packed-texture and partial-upload paths.


Verification and cleanup
------------------------

`make -C py check` runs Ruff format/lint and `ty check`. `make -C py test` first checks these, builds the current Rust example through `make python-reference`, and runs standard-library unittest checks. The example emits 420 accepted indexed pictures plus two channels of sound registers for each of three scripted games, followed by all movable positions, safe-player position, game, victory, held identity, carrying offsets, inactivity counters, and gates. Python executes the same console edges, motion, fire, color, and difficulty inputs and compares every emitted byte. Fixture observations are development-only; runtime code has no recording or replay adapters.

Compositor tests compare independent pixel footprints across priorities, barriers, both widths, and seams; hidden-geometry changes must replace contacts even when pixels stay unchanged. Sprite expansion checks both slots at startup eligibility, zero terminators, and a wrapping eligibility boundary that distinguishes clearing from retaining the last row. `make -C py smoke` opens the real pygame window, checks 3x startup, unchanged scaling, exposure repaint, key repeat/mute, all shortcut sizes, Gold Key pickup, and quit, then closes SDL and saves `target/py-smoke.bmp`.

`mostlyclean` removes Python and test bytecode caches, Ruff caches, and the generated smoke capture while preserving the runnable virtual environment. `clean` additionally removes the environment. Neither target changes source, configuration, or persistent data.


Execution, validation, and termination
======================================

The root workspace contains one root game package, `adventure`, with its sources in `src/`, plus the dependency-free `adventure-trace` library in `src/trace/` and cartridge-execution library in `src/rawemulator/`. `src/main.rs` parses the CLI before creating files or opening devices. No arguments start normal interactive play. `--record FILE` saves consumed inputs. `--replay FILE` reads an input recording and runs only the main game. `--trace FILE` saves semantic observations and validates them against the cartridge during play. `--verify FILE` reads a trace, checks main-game observations against it, and checks those independently produced observations against cartridge execution. `--validate` enables the same live cartridge comparison without a trace file. Input recording can accompany trace recording or live validation; playback and verification each own their endpoint and exclude live recording, validation flags, and frame limits.

Replay defaults to displayed, audible, paced playback; trace verification defaults to headless. `--display` explicitly enables display, sound, and 60 Hz pacing; `--headless` uses no SDL initialization, audio generation, or sleep and runs at full speed. These presentation options also apply to live play; headless live play requires `--frames N` to provide an endpoint. `--unpaced` disables pacing for a displayed run. `--snapshot FILE` saves a final displayed picture and requires a displayed host. Displayed modes start unmuted at 3x size, retain mute and resizing, and permit early quit. Neither mute nor resizing affects input or trace clocks. Early playback quit succeeds without claiming complete verification.

Every validation session installs a record observer on the main `Session`. A shared queue holds its emitted typed `Record` values. Trace recording writes each record to its writer while forwarding the same record for cartridge comparison; live validation writes to a sink. Replay forwards the actual main observations while checking them against the saved trace. The initial boot record is supplied explicitly when the validator attaches. Subsequent input and state events are forwarded without changing gameplay decisions. Ordinary input recording and playback do not install validation.

At each `Adventure::present_picture`, the main game has completed all work preceding its logical boundary. Before calling the host, `Validation::catch_up(frame)` drains the queued events into the raw emulator's streaming session. Raw execution resumes the previous suspended boundary, runs original instructions, and stops just before the next `$F0EC` frame-entry instruction, when its semantic frame equals the main session's current frame. The bus's `beginning_frame` flag distinguishes the timer-loop reentry from a fresh frame. All queued observations must have matched by this boundary. A mismatch, missing observation, unsupported instruction, or exhausted 100,000-instruction allowance returns an error. The host then presents and performs the sole pacing sleep, so raw execution consumes the remaining frame budget rather than adding a second sleep. Accepted presentation advances the main semantic and input-recording clocks once; drawing and wall time never advance them.

Normal stop rejects the final attempted presentation without advancing the clock. The main session emits and flushes its `end`, then `Validation::finish(frame)` supplies remaining events and sets the raw bus's stopping frame. The raw CPU reaches its final attempted frame boundary, emits `end`, and must consume every expected event. Cancellation before boot initialization instead consumes the boot/end pair without executing world initialization. Game 3 initialization uses the same boundary path and therefore remains synchronized. Live validation storage is bounded to a single boundary's events; saved trace verification retains the parsed source trace. The normalizer's per-object state remains live across queue drains.


Original-cartridge execution
----------------------------

The raw emulator embeds `rom/adventure.bin` through `src/rawemulator/adventure.bin`; its public `Validator` exposes construction, frame catch-up, and termination rather than a standalone CLI. Its bus, CPU, and collision-only TIA remain private. Execution begins at the reset vector with RAM-backed stack operations, addressing and branch page penalties, instruction cycle counts, horizontal waits, and RIOT countdown underflow. ROM-specific hooks observe complete operation results and consumed inputs. Coordinate writes assemble complete positions through the shared `PositionAssembly` model.

The TIA advances pixel clocks and independently computes playfield reflection, normal and quadruple sprites, delayed P1 graphics, ball/missile widths, reset strobes, fine motion, blanking, and collision clearing. Layer priority and colors cannot affect collision answers. It never uses the main game's rules, rasterizer, or contacts. Unsupported executed instructions fail explicitly; divergence diagnostics include ROM PC and hardware state. This hardware subset supports the fixed cartridge rather than arbitrary Atari programs.

The separate test-only ROM executor in `src/tests/rom_oracle.rs` supports direct operation/state comparisons and drives `reference_video.rs`, a minimal independent scanline raster used only by those tests. The running executable does not include this raster. The main game's complete-layer compositor is checked against it for every room, vertical byte, and applicable sprite width, including retained state and hidden contacts. `tests/stella-game-one.pixels` independently checks all 160×193 full kernel lines against Stella, excluding the cartridge's partially blanked last line. Calibrated coordinate cases cover screen seams and reset timing in horizontal blanking or a subsequent scanline.


Build, checks, and development tools
------------------------------------

Plain `make` builds only `target/release/adventure`. `make run ARGS='...'` builds incrementally and runs it. The root Cargo workspace owns one lockfile and target directory; Cargo tracks source changes, deletions, embedded cartridge inputs, and compiler settings. Component Makefiles forward to the root. `make format` applies formatting; `make lint` checks formatting then Clippy with warnings as errors; `make test` runs all Rust library, CLI, recording, trace, and hardware tests. `make check` runs lint, ROM verification, and Rust tests. SDL2 development files and pkg-config are needed for Rust builds; assembly reconstruction additionally needs GNU Make 4.3+, a C compiler, curl, tar, sha256sum, Python, and uv. Rust builds use tracked inputs without an assembly prerequisite.

`make smoke` runs the headed SDL smoke example. It injects real SDL controls, starts Game 1, retrieves the Gold Key, checks audible pickup samples, mute/repeat behavior, and all shortcut window sizes, saves `target/adventure-smoke.bmp`, and closes after 105 pictures. It then verifies the recorded session through a fresh displayed host. SDL-specific unit tests can run serially under dummy drivers with `SDL_VIDEODRIVER=dummy SDL_AUDIODRIVER=dummy make test TEST_ARGS='--lib -- --include-ignored --test-threads=1'`. Dummy drivers exercise host behavior without claiming physical display/audio verification.

The saved human victory fixture `tests/easy.trace` remains immutable test input: 4,045 pictures with victory observed at 1:02:03. Standard trace tests execute it against both main and raw implementations and require victory. Generated sessions additionally cover all games, short stopping phases, resets, difficulty/color changes, key pickup, and castle entry. Long live validation runs exercise streaming without a file. CLI tests force invalid SDL drivers to prove headless workflows initialize no devices, distinguish display defaults, reject conflicting modes, and report corrupt traces cleanly.

`make python-reference` builds the test-only Rust example used by `make -C py test`; three scripted games each compare 420 exact rasters and sound-register pairs plus final world positions and counters. `make flamegraph` runs installed cargo-flamegraph with optimized debug symbols; `ARGS` supplies game arguments and `FLAMEGRAPH_ARGS` supplies profiling options. `make performance` build the single `adventure_performance` example and collect application/service timings. Benchmark artifacts stay under `target/`; they never regenerate the saved trace. Historical multi-implementation measurements are not current performance claims.

`mostlyclean` discards Rust compiler intermediates and ROM analysis intermediates while retaining the deployed release executable, assembled ROMs, and assembler. User captures and recordings are preserved by mostlyclean. `clean` also removes deployment/build assets. Neither target touches configuration, databases, backups, or persistent application data. Python environments and caches belong to `py/Makefile`; ROM analysis tools and their uv environment belong to `rom/Makefile`.


Coordinate calibration
----------------------

Game coordinates and screen coordinates have different origins. For game x values 0–164, a normal sprite begins at x−1, an enlarged sprite begins at x, and a player or barrier begins at x−2, with signed correction followed by modulo-160 wrapping. Adventure's positioning loop places reset writes in horizontal blanking for game x values 165–239: the normal sprite origin becomes `(x % 15) − 4`. For 240–255 the reset lands in the following visible line and the normal origin becomes `x − 229`. Enlarged sprites add one to the normal origin; solid drawables subtract one. These mappings include the distinct sprite/solid decode delays described by [Stella's player positioning](https://github.com/stella-emu/stella/blob/master/src/emucore/tia/Player.cxx) and [ball positioning](https://github.com/stella-emu/stella/blob/master/src/emucore/tia/Ball.cxx). They apply equally to visible pixels and contacts; gameplay records retain their original coordinates.


Controls and switch history
---------------------------

SDL maintains independent held keys and latching TV/difficulty switches. The left joystick uses arrows and keypad 8/2/4/6, with Space/Left Ctrl/keypad 5 as fire. The right joystick uses Y/H/G/J and F. F1/Tab is Select; F2/Enter/keypad Enter is Reset. F3/F4 selects color/monochrome. F5/F6 sets left difficulty A/B; F7/F8 sets right difficulty A/B. Both switches default to B. Escape, Ctrl+Q, Cmd+Q, and window close quit. Losing focus releases held keys but retains the TV/difficulty settings. Releasing one alias cannot release another key for the same control.

The executable prints a compact keyboard reference before entering the game and shares that reference with `--help`. Host shortcuts use non-repeating SDL key-down events: M toggles mute, and number-row 1/2/3 restores the window to 1x/2x/3x. A 1x picture is 320×194 window pixels, preserving the existing two-to-one horizontal pixel expansion; 2x is 640×388 and 3x is 960×582. Window shortcuts restore these dimensions even after manual resizing. The logical picture size, game state, input history, and frame pacing do not change.

Console polling recognizes newly pressed Reset and Select by their prior sampled states. Fire recognition also retains its preceding sample. The initial history reproduces the cleared ROM RAM's button-edge behavior, rather than treating every currently held button as a new press. Inactivity considers joystick directions and Reset/Select, including the unused right joystick directions; fire alone does not clear the ROM's counter.

Right difficulty B removes the sword from dragon avoidance reactions, and A includes it. Left difficulty selects the roar/bite countdown within each game's pair of starting values. Color/monochrome selects the room and descriptor's respective color choices. Keyboard mappings follow the [Stella manual](https://stella-emu.github.io/docs/index.html#Keyboard); the NTSC presentation palette follows [Stella's palette table](https://github.com/stella-emu/stella/blob/master/src/common/PaletteHandler.cxx).
