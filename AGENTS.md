Check in to git automatically with 1-line commit messages at logical milestones. This is a project override for the general no-git rule. If there are uncommited changes, always ask if they should be included in the next checkin; do not automatically exclude them.


Project goals
=============

Port Adventure's observable game behavior from the Atari 2600 to Rust with SDL. Preserve frame-by-frame pictures, pixel-accurate contacts, sound, input consumption, update ordering, and all player-visible quirks. Correct known fidelity discrepancies rather than silently treating them as acceptable approximations.

Express the game rather than reproduce its host machine. Use named game concepts, explicit state transitions, ordinary data structures, and whole-picture and whole-sprite operations. Keep byte wrapping, display selection, retained state, and scheduling rules where they affect behavior; isolate their hardware explanation and calibration from the gameplay rules. Clearly distinguish fidelity requirements from implementation choices.

Prepare geometry once for independent collision detection and rendering. Collision results must not depend on painting or visible color priority. Preserve the exact points where contacts are produced and consumed, including contacts from retained display state. Replacement graphics must be separable from the original collision geometry.

Render incrementally: retain completed output and recompute affected regions rather than regenerate every changed picture. Preserve hidden-geometry changes even when visible pixels remain identical. Measure before adding more elaborate optimizations.

Advance sound commands and sample generation per picture while retaining oscillator, divider, noise, fractional-clock, and filter state across silence, effects, and mute. Separate faithful sample generation from host delivery policies.

Keep the game understandable without Atari knowledge and reusable for studying its design, ports to other languages and systems, and alternative graphics. Prefer a small set of explicit modules and dependencies over a general-purpose engine.

Validate changes against independent execution of the original cartridge at semantic and lower-level boundaries. Preserve immutable reference fixtures and supplement equivalence tests with readable examples of game rules. Do not change an oracle merely to agree with the port; establish corrections independently. Keep NOTES.md current with complete operation flows, retained-state invariants, and validation limits.
