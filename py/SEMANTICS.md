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

Events select a pending sound effect and duration. Envelope advancement decrements duration before publishing a separate active effect/remaining/cadence command; pending events cannot alter a previously published voice. Normal cadence is once per three pictures; victory uses every picture. Victory also updates the retained room foreground. The six envelopes and register masks match the refactor. Synthesis integrates the native NTSC clock through persistent divider/noise/pulse feedback latches and a 20 Hz DC-removal filter, then stores signed-16-bit PCM.

The host lazily caches complete effect/cadence clips, with offsets per envelope step and a tapered 25 ms release. Consecutive commands continue playback; interruptions, restarts, and unmute seek the current step. pygame applies a two-millisecond fade-in on clip starts and fade-out on silence/mute. Playback timing belongs to the audio device; gameplay alone advances effect counters. The pygame host uses signed-16-bit clips and pygame fade transitions rather than the Rust callback's exact interruption ramps and floating-point output. Sound clips are generated lazily and rendering retains complete indexed/scaled surfaces rather than Rust's native packed-texture and partial-upload paths.


Verification and cleanup
------------------------

`make -C py check` runs Ruff format/lint and `ty check`. `make -C py test` first checks these, builds the current frame example through `make frame-python-reference`, and runs standard-library unittest checks. The example emits 420 accepted indexed pictures plus two channels of sound registers for each of three scripted games, followed by all movable positions, safe-player position, game, victory, held identity, carrying offsets, inactivity counters, and gates. Python executes the same console edges, motion, fire, color, and difficulty inputs and compares every emitted byte. Fixture observations are development-only; runtime code has no recording or replay adapters.

Compositor tests compare independent pixel footprints across priorities, barriers, both widths, and seams; hidden-geometry changes must replace contacts even when pixels stay unchanged. Sprite expansion checks both slots at startup eligibility, zero terminators, and a wrapping eligibility boundary that distinguishes clearing from retaining the last row. `make -C py smoke` opens the real pygame window, checks 3x startup, unchanged scaling, exposure repaint, key repeat/mute, all shortcut sizes, Gold Key pickup, and quit, then closes SDL and saves `target/py-smoke.bmp`.

`mostlyclean` removes Python and test bytecode caches, Ruff caches, and the generated smoke capture while preserving the runnable virtual environment. `clean` additionally removes the environment. Neither target changes source, configuration, or persistent data.
