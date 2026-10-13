Adventure ROM analysis
======================

This project preserves and analyzes the 4,096 bytes in `rom/adventure.bin`. `adventure.asm` is the preserved address-labeled baseline. `adventure-gameplay.asm` is a separate editable edition with gameplay names and narrative comments. Both editions' instructions, explicit data bytes, and vector words must assemble to the original ROM without any differences. The original ROM is an input and is never rewritten.

The original SHA-256 is `b0326b45e5cab066b2d03f1e683708d82de3ffe60b2e968bf88c5494237f0a96`.

The Rust executable is a functional port, not a CPU emulator. Its game routines and static data appear in `src/adventure.rs` in the assembly's lexical order. The two assembly editions and ROM remain inputs to analysis and verification; compiling or running the game does not rewrite them.


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

`make check` completes Ruff formatting checks, Ruff lint checks, and `uv run ty check` before invoking behavioral verification. Verification compares both editions with all ROM bytes. The baseline keeps its checks for basic-block labels, matched return contexts, graphics/code separation, graphics page containment, and scanline path timing. The gameplay edition additionally checks that DASM marks every symbol as referenced, its code-label addresses exactly cover the named control targets, and each of its 464 emitted graphical rows matches the underlying sprite bits or decoded room walls. Expected failures are reported without stack traces.

`make -C rom mostlyclean` removes both editions' assembly listings and symbols, analysis intermediates, Python bytecode, and downloaded assembler build intermediates, preserving both assembled ROMs and the local assembler executable. `make -C rom clean` additionally removes `rom/build/` and the downloaded assembler executable. Neither target modifies the original dump, either assembly source, documentation, Python sources, dependency configuration, or persistent user data.


Rust execution workflow
-----------------------

The Rust entry creates `SdlHardware` and an `Adventure` instance, then calls `cartridge_entry`. That routine calls `power_on` and repeatedly calls `run_game`; the latter translates one passage through the original main loop. The number and order of frame boundaries come from the original calls, rather than a separate frame-phase scheduler.

`power_on` clears named RAM, the functional TIA, and the sound channel controls, positions the side barriers at x=13 and x=150, samples the host's controls, and initializes the selected world. The initial sample uses `FrameHost::sample_controls`, which processes input without presenting, generating audio, sleeping, or consuming the frame limit. A quit or input error returns before initialization and the scheduler. World initialization changes precisely the original copied world fields, player/saved rooms and y coordinates, victory flag, held-object identity, and console-button history. It retains counters, sound state, saved x coordinates, fire history, and prepared display data. Game 3 randomization occurs before its additional frame boundary and picture, as in the original code. Reset and Select held at startup are recorded as already pressed; a release followed by a press produces the next edge. The first scheduler pass also sees the sampled joystick and difficulty settings.

`run_game` calls console polling, sound advancement, and inactivity tracking in that order. Its victory branch calls begin-frame, draw-room, and display preparation before returning to the outer loop. Otherwise, it tests the chalice's room and sets victory when that room is the Golden treasury, then continues through that turn's original play calls. The three sequences are:

1. Move player with both axes, move the held object, begin frame, prepare display, draw room, handle pickup/drop.
2. Move player vertically, update dark-room light, begin frame, update bat, update gates, draw room, update Grundle, update Yorgle.
3. Begin frame, move player horizontally, update Rhindle, update magnet, draw room, return to the loop.

Display preparation happens once along the normal turn. The next two pictures deliberately reuse its object images, positions, colors, sizes, and wall data while other state changes. Drawing repositions the player from the current player record. This preserves the original ordering and stale-display behavior instead of substituting a freshly computed scene for every picture.


Named state and typed data
-------------------------

`Ram` contains named player, safe-player, light, dot, item, dragon, bat, gate, selection, attachment, visibility, input-history, inactivity, and sound fields. `PreparedDisplay` inside RAM owns the persistent wall reference, two selected bitmap references, and copied object coordinates. Positions use a closed `Room` enum plus wrapping byte coordinates. The three named dragons each have a position, direction, and dragon state; the bat separately has its position, direction, animation, optional cargo record, and flight timer. There is no runtime RAM byte array, CPU register file, instruction decoder, program counter, or emulated call stack.

All transient uses of the original pointer pair and scratch cells are locals, arguments, or returned values. Room lookup returns a typed room definition. Image lookup returns a bitmap. Object lookup returns a position from a live named record or immutable scenery. Contact-object identities in pickup and gate handling are independent local values. Reaction selection returns the chosen pair and direction; it does not reuse contact, loop-index, or pointer storage. Creature identity, reaction list, ignored record, and speed are explicit function arguments.

Object identities preserve catalog order, but represent identities rather than ROM byte offsets. The empty-object descriptor remains a valid sentinel whose position references the bridge record and whose bitmap is blank. The bat's initial dummy cargo address has no game-state effect and becomes `None`; later cargo is a typed live record. Positions in immutable gate, number, and credits records are copied for reading; attempted gate entry cannot mutate those immutable records, just as writes to ROM did not change them.

Directions use four booleans. Independent right, left, up, and down steps retain the original order and wrapping arithmetic. Opposite directions can coexist in initial records and cancel their coordinate changes. A missing reaction preserves the creature's previous direction; an eligible pair at equal coordinates supplies an idle direction and stops it. These cases must remain distinct.

Original signed-flag comparisons that interpret wrapped coordinates remain explicit signed comparisons of wrapping differences. In particular, gate exits and top/bottom screen handling cannot all be replaced with ordinary unsigned threshold tests. Attachment offsets and the magnet's temporary y adjustment also wrap as bytes. Game 3 uses the same five-bit candidate generation, seed update, inclusive bounds, and reverse constraint order as the ROM; it writes back the updated inactivity seed.

The room table contains 31 typed records with wall references, color choices, mirror/repeat behavior, foreground priority, player width, thin barriers, and four typed exits. Direct exits contain `Room`; game-dependent exits index one of six three-game destination groups. The object catalog contains nineteen typed descriptors with position source, state source, image choices, color choices, and sprite style. A literal color and an inactivity-counter color are different variants; resolving the latter preserves the original transformation and final shift.

Wall layouts are arrays of seven `PlayfieldRow` values. Where the ROM shares a bottom band with a following layout, each typed layout includes that shared logical row. Sprite arrays retain the original row bytes and terminating zero. Gate image choices reference offsets into one shared array; dragon image choices reference walking, dead, walking again for the swallowed state, and biting images. Image-range ceilings remain inclusive and end at 255. Graphics and map data occur after the same preceding routines as in the assembly; reaction lists and starting-world data remain between their neighboring routines.


Functional rendering and frame boundaries
-----------------------------------------

The TIA owns persistent first/second sprite objects, the player square, and left/right barriers. Native operations set colors, wall patterns, priority, position, width, scale, duplication, row data, enable state, and row delay. Positioning collapses the original coarse delay, reset strobe, and fine adjustment into one coordinate call. Sprite scales form a closed normal/double/quadruple domain; copy patterns select the TIA's single, double, or triple offsets.

`draw_room` positions both selected objects and the current player, clears collision latches, initializes bitmap and wall-band indices, clears the second object's row, enables its delayed-row behavior, and preloads the first wall band. The picture then processes row counts 104 down to 8 inclusive. The first iteration skips the second-object fetch exactly as the original kernel does.

Each iteration stages the next second-object row when its vertical comparison permits, prepares the first-object row, computes player visibility, and determines whether walls need refreshing. Horizontal sync paints line A using the previously committed object/player state. Then the player enable and first-object row are applied; setting that row also commits the staged second-object row. Refresh iterations replace the wall band next. Horizontal sync paints line B. The prepared bitmap row advances only when nonzero, so a terminating zero continues to supply blank rows. Six refreshes follow the initial band. Cleanup blanks the picture and clears both sprite rows.

The visible framebuffer is 160 by 194 indexed-color pixels. Rendering unpacks four-pixel playfield cells in the original bit order and mirror/repeat arrangement. Sprite scaling and duplication operate on the same eight-bit row shapes. Horizontal positions wrap within the 160-pixel drawable domain. Both thin barriers use their fixed positions and two-pixel widths; the player width comes from the room's appearance. SDL displays this at a two-to-one horizontal pixel aspect, initially in a 640 by 388 resizable window, with a fixed NTSC RGB palette.

Game coordinates and screen coordinates have different origins. For game x values 0–164, a normal sprite begins at x−1, an enlarged sprite begins at x, and a player or barrier begins at x−2, with signed correction followed by modulo-160 wrapping. Adventure's positioning loop places reset writes in horizontal blanking for game x values 165–239: the normal sprite origin becomes `(x % 15) − 4`. For 240–255 the reset lands in the following visible line and the normal origin becomes `x − 229`. Enlarged sprites add one to the normal origin; solid drawables subtract one. These mappings include the distinct sprite/solid decode delays described by [Stella's player positioning](https://github.com/stella-emu/stella/blob/master/src/emucore/tia/Player.cxx) and [ball positioning](https://github.com/stella-emu/stella/blob/master/src/emucore/tia/Ball.cxx). They apply equally to visible pixels and contacts; gameplay records retain their original coordinates.

The test fixture `tests/stella-game-one.pixels` comes from independent rendering of the original cartridge in Stella 7.0. Both versions compare all 160×193 full kernel pixels against it, with palette colors normalized to indices. The final partially blanked ROM scanline is excluded because the functional model paints it as a complete line. Coordinate cases independently recorded from Stella cover screen seams, horizontal-blank resets, and a positioning delay crossing into another scanline. The ROM kernel comparison still verifies row output ordering through the functional compositor, while this independent fixture validates its placement assumptions.

For each pixel, the compositor evaluates walls, player, sprites, and barriers independently and accumulates player/wall, player/barrier, player/object, and object-pair contacts. Contacts do not depend on which color or layer wins. If room foreground priority is enabled, walls/player precede objects; otherwise first sprite/left barrier precede second sprite/right barrier, then walls/player, then background. Contacts persist until the next `draw_room` clears them, so game handlers see the preceding picture's observations. Only the selected object slots can report player/object or object/object contact.

Horizontal sync neither presents nor sleeps. `begin_frame` blanks further drawing and calls the host's vertical boundary with the completed framebuffer. SDL processes queued events, converts indexed colors to RGB, presents the picture, and performs the only pacing sleep, targeting 60 presentations per second. The returned controller/switch snapshot becomes the game's current input. A quit event or exhausted bounded-run limit returns `None` and unwinds the game cleanly.

CPU cycles, timer polling durations, sub-line register write times, horizontal delay loops, and analog television artifacts are outside this functional hardware model. The final kernel line B is painted as a whole line rather than reproducing the original mid-line blanking. The order of row preparation, commits, wall updates, and frame boundaries remains part of the functional contract.


Controls and audio
------------------

SDL maintains independent held keys and latching TV/difficulty switches. The left joystick uses arrows and keypad 8/2/4/6, with Space/Left Ctrl/keypad 5 as fire. The right joystick uses Y/H/G/J and F. F1/Tab is Select; F2/Enter/keypad Enter is Reset. F3/F4 selects color/monochrome. F5/F6 sets left difficulty A/B; F7/F8 sets right difficulty A/B. Both switches default to B. Escape, Ctrl+Q, Cmd+Q, and window close quit. Losing focus releases held keys but retains the TV/difficulty settings. Releasing one alias cannot release another key for the same control.

The executable prints an aligned, compact keyboard reference after SDL initialization and before entering the game, and shares that reference with `--help`. Host shortcuts use non-repeating SDL key-down events: M toggles mute, and number-row 1/2/3 restores the window to 1x/2x/3x. A 1x picture is 320×194 window pixels, preserving the existing two-to-one horizontal pixel expansion; 2x is 640×388 (the startup default) and 3x is 960×582. Window shortcuts restore these dimensions even after manual resizing. The logical picture size, game state, input history, and frame pacing do not change.

Mute starts disabled. Toggling clears queued samples; enabling mute pauses the SDL audio device and suppresses sample queuing, while disabling mute resumes playback from the current effect. Synthesis, fractional sample accounting, and game sound counters continue while muted. Muting never modifies the game channel controls, and host shortcut keys never enter the game joystick/console state. Losing focus does not change mute state or window size.

Console polling recognizes newly pressed Reset and Select by their prior sampled states. Fire recognition also retains its preceding sample. The initial history reproduces the cleared ROM RAM's button-edge behavior, rather than treating every currently held button as a new press. Inactivity considers joystick directions and Reset/Select, including the unused right joystick directions; fire alone does not clear the ROM's counter.

Right difficulty B removes the sword from dragon avoidance reactions, and A includes it. Left difficulty selects the roar/bite countdown within each game's pair of starting values. Color/monochrome selects the room and descriptor's respective color choices. Keyboard mappings follow the [Stella manual](https://stella-emu.github.io/docs/index.html#Keyboard); the NTSC presentation palette follows [Stella's palette table](https://github.com/stella-emu/stella/blob/master/src/common/PaletteHandler.cxx).

`Sound` stores waveform, frequency, and volume for two channels, masking them to four, five, and four bits respectively. The translated sound routine retains decrement-before-dispatch ordering and all six event expressions. Victory updates the foreground color; roar, swallow, dragon death, drop, and pickup retain their original parameters and durations. Ordinary play advances effects once per three-picture scheduler pass, while victory advances once per picture. Audio generation never decrements these game counters.

`begin_frame` passes the current channel controls alongside the picture to `FrameHost::vertical_sync`. SDL owns a persistent `Synthesizer` with separate divider, pulse, noise, feedback, and clock-gating state for each channel. Control changes and zero volume retain oscillator state; constructing a new host resets the synthesizer. The oscillator circuit implements all 16 waveform settings using the [TIA channel behavior documented by Stella](https://github.com/stella-emu/stella/blob/master/src/emucore/tia/AudioChannel.cxx). The native audio rate is the NTSC color clock (3,579,545 Hz) divided by 114. This clock is internal to audio generation and does not introduce CPU timing, scanline waits, or sleeps.

Each accepted vertical boundary generates one sixtieth of a second of audio from the current controls. SDL requests 48,000 Hz mono floating-point samples with a 512-sample device buffer; generation uses the obtained sample rate and carries fractional samples between frames. The generator integrates native output levels over each PCM sample and preserves fractional clocks across buffers. It adds both channels linearly, scales full combined volume to 0.6, and applies a 20 Hz DC-removal filter. This approximates audio output without modeling analog mixing or intra-frame sound-write timing. SDL asynchronously consumes samples queued before the existing frame-pacing sleep, using its [audio queue API](https://docs.rs/sdl2/0.38.0/sdl2/audio/struct.AudioQueue.html).

When more than three frames of audio remain queued, the host clears the backlog before adding the new frame, bounding latency and memory in unpaced runs. The synthesizer still advances through every generated frame. There is no additional audio wait or shutdown drain; quitting closes the audio device and discards pending samples. Failure to initialize or queue audio propagates through the existing CLI error path. Test hosts consume channel controls without opening a device.


Rust build and validation
-------------------------

Rust source resides in `directport/`, `scanline/`, `rawemulator/`, and `trace/`. The root Cargo workspace shares dependency versions, one root lockfile, and `target/`. Plain `make` builds all workspace release binaries; `make directport-run` and `make scanline-run` use Cargo to build and run the selected release binary. Each cartridge consumer embeds bytes through its crate-local `adventure.bin` symlink to `rom/adventure.bin`; no Rust build requires generated assembly output. Component Makefiles forward to the root rules. Checks run formatting, Clippy with warnings as errors, then tests. Smoke captures reside in `target/`. `mostlyclean` preserves release executables while removing shared compiler intermediates and generated captures; `clean` also removes deployment outputs. Neither target changes source, ROM input, configuration, or persistent user data.

The test-only `src/rom_oracle.rs` executes the supplied original ROM with ordinary call continuations. It is compiled only for tests. Test serialization maps named fields to the original RAM layout solely for comparison; the running port does not use that map or the ROM file. Comparisons retain meaningful direction bits and button histories while excluding discarded pointer/scratch bytes and unused direction bits.

Differential checks cover world initialization, every one-byte Game 3 seed, room boundaries in all games, wrapping coordinate edges, open-castle exits, secret entry, bridge rollback bounds, pickup/attachment, dragon states and difficulties, bat cargo, gates, magnet attraction, lighting clamps, display rotation, every image-range state, inactivity overflow, sound parameters, and Reset/Select edges. The original kernel also drives a test raster through its output writes, with positioning collapsed to a coordinate call and final blanking treated as a whole-line boundary. Every room's resulting indexed pixels, line count, and contacts are compared with the functional kernel. Independent audio checks exercise all waveform periods, divider pitch, asymmetric tones, buffer continuity under control changes, channel mixing, volume scaling, silence, and DC removal. Independent compositor checks exercise delayed-row commits, hidden-layer contacts, wall mirror/repeat, and scaled duplicate sprites.

The headed SDL smoke session delivers SDL Enter and arrow events through the real host, starts Game 1, retrieves the Gold Key, checks that its sound controls generate audible pickup samples while SDL queues playback, verifies mute toggling (including ignored key repeat) and all three real SDL window dimensions while movement remains held, saves a BMP, and closes after 105 presented frames. CLI bounded runs, snapshots, and optional unpaced mode support additional SDL checks without altering the game loop. Expected CLI and SDL failures report a concise error and exit without a stack trace.


Independent Rust refactor
-------------------------

The preserved Rust implementation resides in `directport/`, and the preserved assembly and analysis sources reside in `rom/`. Assembly and analysis run independently through `rom/Makefile`, read `rom/adventure.bin`, and write `rom/build/`. Python environments and downloaded assembler tools remain within `rom/`.

`scanline/` adds an independent Rust game with the same gameplay contract. Its software groups immutable data by purpose and separates world state from display snapshots; its scheduler models individual picture phases. Its hardware rasterizer consumes complete scanline values and supports only the widths, barriers, walls, and priorities Adventure uses. Row staging belongs to game display construction rather than hardware drawable state. The SDL host retains frame-boundary input, audio, and pacing. The running executable has no dependency on the preserved translation; that package is used only by differential tests.

[scanline/SEMANTICS.md](scanline/SEMANTICS.md) specifies state ownership, scheduling, rendering, and validation. `make scanline-run`, `make scanline-check`, and `make scanline-smoke` operate through the workspace. Both ports use the tracked cartridge for differential tests, the same calibrated placement contract, independent runtime implementations, and a shared Stella pixel fixture.


Semantic trace verification
---------------------------

Both Rust ports can record boot-to-exit game observations with `--record FILE` and replay them headlessly with `--replay FILE`. The shared contracts, text grammar, normalization, input consumption, collision observations, and independent cartridge validation are specified in [trace/SEMANTICS.md](trace/SEMANTICS.md). `make validate TRACE=FILE` builds and runs the headless original-instruction validator. Recording and replay preserve the existing game scheduling and frame-host interfaces.


Frame implementation
--------------------

`frame/` follows all gameplay, input, presentation, audio, coordinate, and collision contracts above. It prepares a complete picture and composes whole layers instead of sending scanlines to video. Separate geometric coverage preserves collision contacts even when higher-priority colors hide objects. The exact picture preparation and composition workflow is specified in [frame/SEMANTICS.md](frame/SEMANTICS.md). `scanline/` remains unchanged.
