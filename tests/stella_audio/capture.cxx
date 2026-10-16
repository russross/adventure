// Drive the original Stella channel through uninterrupted control and silence transitions.
#include "AudioChannel.hxx"
#include <iostream>

int main() {
    AudioChannel channel;
    channel.reset();
    const auto capture = [&channel](unsigned mode, unsigned frequency, unsigned volume,
                                   unsigned clocks) {
        channel.audc(mode);
        channel.audf(frequency);
        channel.audv(volume);
        std::cout << mode << ' ' << frequency << ' ' << volume << ' ';
        for (unsigned clock = 0; clock < clocks; ++clock) {
            // The Rust oscillator exposes one complete pair of native clock phases per tick.
            channel.phase0();
            channel.phase1();
            std::cout << "0123456789abcdef"[channel.actualVolume()];
        }
        std::cout << '\n';
    };

    // Exercise every waveform, silence, changes while silent, and resumption without resetting.
    for (unsigned mode = 0; mode < 16; ++mode) {
        capture(mode, mode * 2, 15, 997);
        capture(mode, mode * 2, 0, 509);
        capture(15 - mode, 31 - mode, 0, 233);
        capture(15 - mode, 31 - mode, 9, 1141);
    }

    // Selected Adventure envelope writes include pickup, drop, roar, slain, swallow, and victory.
    // Unequal clock counts make phase retention observable across interruptions and restarts.
    for (unsigned remaining = 15; remaining > 0; --remaining) {
        capture(6, remaining & 31, 5, 521);
        capture(6, (remaining ^ 3) & 31, 5, 523);
        capture(remaining & 1 ? 3 : 8, ((remaining >> 2) + 28) & 31, remaining, 527);
        capture(4, remaining ^ 31, remaining, 529);
        capture(6, remaining ^ 15, ((remaining >> 1) + 8) & 15, 541);
        capture(remaining & 15, remaining >> 3, remaining >> 1, 547);
    }
    return std::cout ? 0 : 1;
}
