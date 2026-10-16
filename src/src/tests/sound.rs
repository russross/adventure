use super::*;

fn tone(mode: u8, frequency: u8, volume: u8) -> Sound {
    let mut sound = Sound::default();
    sound.voice(mode, frequency, volume);
    sound
}

// Known native-clock periods distinguish tonal dividers from short and long
// noise sequences. A warmup removes the power-on transient from each circuit.
#[test]
fn tone_and_noise_periods_match_tia_modes() {
    for (mode, period) in [
        (1, 15),
        (2, 465),
        (3, 465),
        (4, 2),
        (5, 2),
        (6, 31),
        (7, 31),
        (8, 511),
        (9, 31),
        (10, 31),
        (12, 6),
        (13, 6),
        (14, 93),
        (15, 93),
    ] {
        let control = tone(mode, 0, 15).channels[0];
        let mut oscillator = Oscillator::default();
        for _ in 0..2048 {
            oscillator.tick(control);
        }
        let bits: Vec<bool> = (0..period * 3)
            .map(|_| {
                oscillator.tick(control);
                oscillator.level(control) != 0.0
            })
            .collect();
        assert_eq!(&bits[..period], &bits[period..period * 2], "mode {mode}");
        assert_eq!(&bits[..period], &bits[period * 2..], "mode {mode}");
        assert!(bits.iter().any(|bit| *bit) && bits.iter().any(|bit| !bit));
        for shorter in 1..period {
            if period % shorter == 0 {
                assert!(
                    (shorter..period).any(|i| bits[i] != bits[i % shorter]),
                    "mode {mode} unexpectedly repeats after {shorter}"
                );
            }
        }
    }
}

#[test]
fn frequency_divider_and_asymmetric_tone_keep_native_pitch() {
    for frequency in [0, 1, 7, 31] {
        let control = tone(4, frequency, 15).channels[0];
        let mut oscillator = Oscillator::default();
        let mut transitions = Vec::new();
        let mut previous = false;
        for clock in 0..512 {
            oscillator.tick(control);
            let bit = oscillator.level(control) != 0.0;
            if bit != previous {
                transitions.push(clock);
            }
            previous = bit;
        }
        assert!(
            transitions
                .windows(2)
                .all(|pair| pair[1] - pair[0] == usize::from(frequency) + 1)
        );
    }
    let control = tone(6, 0, 15).channels[0];
    let mut oscillator = Oscillator::default();
    for _ in 0..128 {
        oscillator.tick(control);
    }
    let highs = (0..31)
        .filter(|_| {
            oscillator.tick(control);
            oscillator.level(control) != 0.0
        })
        .count();
    assert!([13, 18].contains(&highs));
}

// PCM generation must be invariant under frame/buffer division, including
// waveform and frequency changes. Neither silence nor writes reset phase.
#[test]
fn pcm_is_continuous_across_buffers_and_control_changes() {
    let rate = NonZeroU32::new(44_100).unwrap();
    let mut whole = Synthesizer::new(rate);
    let mut split = Synthesizer::new(rate);
    let mut sound = tone(8, 7, 15);
    for mode in 0..16 {
        sound.voice(mode, mode * 2, sound.channels[0].volume);
        let mut expected = [0.0; 997];
        let mut actual = [0.0; 997];
        whole.render(&sound, &mut expected);
        for chunk in actual.chunks_mut(73) {
            split.render(&sound, chunk);
        }
        assert_eq!(expected, actual, "mode {mode}");
        assert!(
            actual
                .iter()
                .all(|sample| sample.is_finite() && sample.abs() < 1.0)
        );
    }
}

#[test]
fn silence_dc_removal_volume_and_channel_mixing() {
    let rate = NonZeroU32::new(48_000).unwrap();
    let mut silent = Synthesizer::new(rate);
    let mut samples = [1.0; 4800];
    silent.render(&Sound::default(), &mut samples);
    assert!(samples.iter().all(|sample| *sample == 0.0));

    let mut low = Synthesizer::new(rate);
    let mut high = Synthesizer::new(rate);
    let mut mixed = Synthesizer::new(rate);
    let quiet = tone(4, 7, 5);
    let loud = tone(4, 7, 15);
    let mut both = tone(4, 7, 5);
    both.channels[1] = both.channels[0];
    let mut a = [0.0; 4800];
    let mut b = [0.0; 4800];
    let mut c = [0.0; 4800];
    low.render(&quiet, &mut a);
    high.render(&loud, &mut b);
    mixed.render(&both, &mut c);
    assert!(a.iter().zip(b).all(|(a, b)| (a * 3.0 - b).abs() < 0.00001));
    assert!(a.iter().zip(c).all(|(a, c)| (a * 2.0 - c).abs() < 0.00001));
    high.render(&Sound::default(), &mut samples);
    assert!(samples[4000..].iter().all(|sample| sample.abs() < 0.00001));

    // Constant-volume modes produce a transient, then settle to inaudible DC.
    for mode in [0, 11] {
        let mut synth = Synthesizer::new(rate);
        synth.render(&tone(mode, 0, 15), &mut samples);
        assert!(
            samples[4000..].iter().all(|sample| sample.abs() < 0.00001),
            "mode {mode}"
        );
    }
}

// Cached envelopes use the reference circuit continuously within each effect,
// including fractional sample boundaries at rates not divisible by sixty.
#[test]
fn cached_effects_match_continuous_synthesis_and_release_to_zero() {
    for rate in [48_000, 44_101] {
        let rate = NonZeroU32::new(rate).unwrap();
        let bank = SoundBank::new(rate);
        for effect in SoundEffect::ALL {
            for cadence in [EffectCadence::EveryPicture, EffectCadence::EveryTurn] {
                let clip = bank.clip(EffectStep {
                    effect,
                    remaining: 0,
                    cadence,
                });
                let mut synth = Synthesizer::new(rate);
                let mut sound = Sound::default();
                for (step, remaining) in (0..effect.duration()).rev().enumerate() {
                    sound.channels[0] = effect.channel(remaining);
                    let start = clip.offsets[usize::from(remaining)];
                    let end = (step + 1) * cadence.pictures() * rate.get() as usize / 60;
                    let mut expected = vec![0.0; end - start];
                    synth.render(&sound, &mut expected);
                    assert_eq!(
                        &clip.samples[start..end],
                        expected,
                        "{effect:?} {cadence:?}"
                    );
                }
                assert!(clip.samples.iter().any(|sample| sample.abs() > 0.001));
                assert!(
                    clip.samples
                        .iter()
                        .all(|sample| sample.is_finite() && sample.abs() < 1.0)
                );
                assert_eq!(clip.samples.last(), Some(&0.0));
            }
        }
    }
}

// All replacement pairs include restarting the same effect. Interrupted ramps
// must start at the actual last output rather than the previous clip's amplitude.
#[test]
fn replacements_restarts_and_silence_have_continuous_ramps() {
    let mut player = EffectPlayer::new(NonZeroU32::new(48_000).unwrap());
    let mut sound = Sound::default();
    for first in SoundEffect::ALL {
        for second in SoundEffect::ALL {
            sound.effect(first, first.duration() - 1, EffectCadence::EveryTurn);
            player.update(&sound, false);
            player.render(&mut [0.0; 137]);
            let previous = player.last_output;
            sound.effect(second, second.duration() - 1, EffectCadence::EveryTurn);
            player.update(&sound, false);
            let mut ramp = vec![0.0; player.transition_samples];
            player.render(&mut ramp);
            assert_eq!(ramp[0], previous, "{first:?} -> {second:?}");
            let clip = player.bank.clip(sound.step.unwrap());
            assert_eq!(ramp.last(), clip.samples.get(player.transition_samples - 1));

            // Silence arriving midway through a restart finishes at exact zero.
            sound.effect(second, second.duration() - 1, EffectCadence::EveryTurn);
            player.update(&sound, false);
            player.render(&mut [0.0; 17]);
            let previous = player.last_output;
            sound.silence();
            player.update(&sound, false);
            player.render(&mut ramp);
            assert_eq!(ramp[0], previous);
            assert_eq!(ramp.last(), Some(&0.0));
            assert!(player.idle());
        }
    }
}

#[test]
fn envelope_steps_continue_and_mute_resumes_current_step() {
    let mut player = EffectPlayer::new(NonZeroU32::new(48_000).unwrap());
    let mut sound = Sound::default();
    sound.effect(SoundEffect::Roar, 15, EffectCadence::EveryTurn);
    player.update(&sound, false);
    player.render(&mut [0.0; 2400]);
    sound.effect(SoundEffect::Roar, 14, EffectCadence::EveryTurn);
    player.update(&sound, false);
    assert_eq!(player.cursor.unwrap().position, 2400);
    assert_eq!(player.transition_left, 0);
    // Presenting a retained command must not restart the clip or its transition.
    player.render(&mut [0.0; 101]);
    player.update(&sound, false);
    assert_eq!(player.cursor.unwrap().position, 2501);

    player.update(&sound, true);
    player.render(&mut [0.0; 96]);
    assert!(player.idle());
    sound.effect(SoundEffect::Roar, 10, EffectCadence::EveryTurn);
    player.update(&sound, true);
    assert!(player.idle());
    player.update(&sound, false);
    assert_eq!(player.cursor.unwrap().position, 5 * 2400);
    assert_eq!(player.transition_left, 96);

    // A faster scheduler changes the clip and seeks the current envelope step.
    sound.effect(SoundEffect::Roar, 9, EffectCadence::EveryPicture);
    player.update(&sound, false);
    assert_eq!(player.cursor.unwrap().position, 6 * 800);
    let mut output = vec![1.0; 48_000];
    player.render(&mut output);
    assert!(player.idle());
    assert!(output[10_000..].iter().all(|sample| *sample == 0.0));
}
