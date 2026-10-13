Adventure traces
================

`adventure-trace` is a dependency-free library shared by the translation, refactor, and original-cartridge validator. It establishes equivalence between their observed mechanics. It does not choose actor reactions, perform collision geometry, or decide whether a pickup, victory, or gate entry should occur. The validator executes the original instruction stream and independently produces the observations used as the correctness reference.


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


Original-cartridge validator
----------------------------

`rawemulator/` embeds the tracked cartridge through its local `adventure.bin` symlink to `rom/adventure.bin`. Cargo tracks that input directly; building requires no assembly step. Normal validation depends only on the shared trace library; the translation is a development dependency for generating test runs.

Execution starts at the reset vector and follows the original instructions, with RAM-backed stack operations, addressing and branch page penalties, instruction cycle counts, horizontal waits, and the RIOT countdown including underflow. The collision-only TIA advances pixel clocks, implements playfield reflection, the game's normal and quadruple sprites, delayed P1 graphics, ball/missile widths, reset strobes, fine motion, blanking, and collision clearing. Layer priority and colors cannot change collision answers. Unsupported executed instructions fail explicitly. This is the hardware subset required by the fixed cartridge, not a general-purpose Atari emulator.

ROM-specific observation points identify completed game operations and consumed input fields. Coordinate RAM writes feed the shared assembly state machine. No game routine, rasterizer, or contact implementation from either Rust port runs inside the validator. A bounded instruction allowance catches stalled execution. CPU and hardware diagnostics accompany divergences.

Checks cover parser and lifecycle errors, motion and cadence changes, wrapping, explicit room edges, component assembly, counter stalls and gate acceleration, writer failures, all three games, Game 3 initialization, termination at different phases, key pickup and gate entry, CPU-driven calibrated positioning, retained hidden contacts, wide-sprite wrapping, delayed graphics, and RIOT timing. The SDL smoke run records real SDL event delivery and then verifies its recording headlessly.
