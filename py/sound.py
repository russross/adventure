"""Game effect envelopes and the TIA tone/noise circuit."""

from array import array
from dataclasses import dataclass
from enum import IntEnum
from math import exp, tau

SAMPLE_RATE = 48000


class Effect(IntEnum):
    Victory = 0
    Roar = 1
    Swallow = 2
    DragonSlain = 3
    Drop = 4
    Pickup = 5


def voice(effect: Effect, remaining: int) -> tuple[int, int, int]:
    if effect == Effect.Victory:
        waveform, frequency, volume = remaining, remaining >> 3, remaining >> 1
    elif effect == Effect.Roar:
        waveform, frequency, volume = 3 if remaining & 1 else 8, (remaining >> 2) + 28, remaining
    elif effect == Effect.Swallow:
        waveform, frequency, volume = 6, remaining ^ 15, (remaining >> 1) + 8
    elif effect == Effect.DragonSlain:
        waveform, frequency, volume = 4, remaining ^ 31, remaining
    elif effect == Effect.Drop:
        waveform, frequency, volume = 6, remaining ^ 3, 5
    else:
        waveform, frequency, volume = 6, remaining, 5
    return waveform & 15, frequency & 31, volume & 15


@dataclass
class Sound:
    selected: Effect = Effect.Victory
    remaining: int = 0
    active: Effect | None = None
    active_remaining: int = 0
    every_picture: bool = False
    revision: int = 0

    def start(self, effect: Effect, duration: int) -> None:
        self.selected = effect
        self.remaining = duration

    # Events choose an envelope; only the turn boundary writes the current voice.
    def advance(self, won: bool) -> None:
        self.revision += 1
        if self.remaining == 0:
            self.active = None
            return
        self.remaining -= 1
        self.active = self.selected
        self.active_remaining = self.remaining
        self.every_picture = won


@dataclass
class Oscillator:
    divider: int = 0
    pulse: int = 0
    noise: int = 0
    enabled: bool = False
    noise_feedback: bool = False
    noise_bit: bool = False
    hold: bool = False

    # Two clock phases retain feedback latches across changes to frequency or mode.
    def tick(self, mode: int, frequency: int) -> None:
        if self.enabled:
            self.noise_bit = bool(self.noise & 1)
            self.hold = (
                self.noise & 30 != 2
                if mode & 3 == 2
                else not self.noise_bit
                if mode & 3 == 3
                else False
            )
            if mode & 3 == 0:
                self.noise_feedback = (
                    bool((self.pulse ^ self.noise) & 1)
                    or self.noise == 0
                    and self.pulse == 10
                    or mode & 12 == 0
                )
            else:
                self.noise_feedback = bool(((self.noise >> 2) ^ self.noise) & 1) or self.noise == 0
        self.enabled = self.divider == frequency
        self.divider = 0 if self.enabled or self.divider == 31 else self.divider + 1
        if self.enabled:
            if mode >> 2 == 0:
                feedback = (
                    bool(((self.pulse >> 1) ^ self.pulse) & 1)
                    and self.pulse != 10
                    and mode & 3 != 0
                )
            elif mode >> 2 == 1:
                feedback = self.pulse & 8 == 0
            elif mode >> 2 == 2:
                feedback = not self.noise_bit
            else:
                feedback = self.pulse & 2 == 0 and self.pulse & 14 != 0
            self.noise = (self.noise >> 1) | (int(self.noise_feedback) << 4)
            if not self.hold:
                self.pulse = (~(self.pulse >> 1) & 7) | (int(feedback) << 3)


class Synthesizer:
    def __init__(self) -> None:
        self.oscillator = Oscillator()
        self.clock_phase = 0
        self.previous_input = 0.0
        self.previous_output = 0.0
        self.decay = exp(-tau * 20 / SAMPLE_RATE)

    # Integrate the native clock over output samples, then remove the DC component.
    def render(self, channel: tuple[int, int, int], count: int) -> array:
        mode, frequency, volume = channel
        samples = array("h")
        clock_period = SAMPLE_RATE * 114
        for _ in range(count):
            remaining, integral = 3579545, 0
            while remaining:
                span = min(remaining, clock_period - self.clock_phase)
                integral += (self.oscillator.pulse & 1) * volume * span
                remaining -= span
                self.clock_phase += span
                if self.clock_phase == clock_period:
                    self.clock_phase = 0
                    self.oscillator.tick(mode, frequency)
            value = integral / 3579545 / 30 * 0.6
            output = value - self.previous_input + self.decay * self.previous_output
            self.previous_input, self.previous_output = value, output
            samples.append(round(max(-1.0, min(1.0, output)) * 32767))
        return samples


# Each cached effect starts independently and keeps phase through its whole envelope.
def effect_samples(effect: Effect, every_picture: bool) -> tuple[bytes, dict[int, int]]:
    duration = (
        255 if effect == Effect.Victory else 4 if effect in (Effect.Drop, Effect.Pickup) else 16
    )
    cadence = 1 if every_picture else 3
    synthesizer = Synthesizer()
    samples = array("h")
    offsets: dict[int, int] = {}
    for remaining in range(duration - 1, -1, -1):
        offsets[remaining] = len(samples) * samples.itemsize
        samples.extend(synthesizer.render(voice(effect, remaining), SAMPLE_RATE // 60 * cadence))
    # A short tapered release removes the final discontinuity without prolonging the envelope.
    release = synthesizer.render((0, 0, 0), SAMPLE_RATE // 40)
    for index, value in enumerate(release):
        release[index] = round(value * (1 - (index + 1) / len(release)))
    samples.extend(release)
    return samples.tobytes(), offsets
