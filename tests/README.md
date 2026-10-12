Stella rendering reference
==========================

`stella-game-one.pixels` contains 160×193 indexed pixels captured from the original cartridge in Stella 7.0 with Game 1 started and no joystick movement. The cartridge SHA-256 is `b0326b45e5cab066b2d03f1e683708d82de3ffe60b2e968bf88c5494237f0a96`.

Stella's debugger script advances twelve frames, presses Reset for six frames using `swchb $3e`, releases it with `swchb $3f`, then advances six frames and executes `saveSnap`. The 320×228 snapshot has two horizontal display pixels per TIA pixel. The fixture takes every second horizontal pixel, removes thirteen top border scanlines, and maps black to 0, gray to 8, and yellow to 26. RGB palette differences therefore do not affect the geometry comparison. The final kernel scanline is excluded because the functional ports deliberately paint that partially blanked ROM line in full.

The starting player occupies x=78–81, the Gold Key occupies x=31–38, and the gate occupies x=76–82. The key's first row is scanline 83 and the player's first row is scanline 147. Both Rust versions compare their entire 193-line raster against this independently rendered fixture.

Position regression cases also use debugger snapshots of the original `PositionDrawable` routine with a one-bit normal sprite. Recorded `(game x, screen x)` pairs include `(0,159)`, `(1,0)`, `(32,31)`, `(80,79)`, `(159,158)`, `(164,3)`, `(165,156)`, `(179,10)`, `(225,156)`, `(239,10)`, `(240,11)`, and `(255,26)`. These cases cover screen wrapping and the reset landing in horizontal blanking or a following scanline.
