use super::*;

#[test]
fn digital_output_matches_independent_stella_capture_through_all_transitions() {
    let mut oscillator = Oscillator::default();
    for (segment, line) in include_str!("../../tests/stella-audio.txt")
        .lines()
        .enumerate()
    {
        let mut fields = line.split_whitespace();
        let control = Channel {
            waveform: fields.next().unwrap().parse().unwrap(),
            frequency: fields.next().unwrap().parse().unwrap(),
            volume: fields.next().unwrap().parse().unwrap(),
        };
        let expected = fields.next().unwrap();
        assert!(fields.next().is_none());

        // One oscillator lives across every segment, including silent mode and frequency changes.
        for (clock, value) in expected.chars().enumerate() {
            oscillator.tick(control);
            assert_eq!(
                oscillator.level(control) as u32,
                value.to_digit(16).unwrap(),
                "Stella segment {segment}, native clock {clock}, controls {control:?}"
            );
        }
    }
}

#[test]
fn picture_sample_clock_retains_fractional_samples_and_phase_when_output_is_discarded() {
    let rate = NonZeroU32::new(44_101).unwrap();
    let mut pictures = PictureSynthesizer::new(rate);
    let mut reference = Synthesizer::new(rate);
    let mut generated = 0;
    for picture in 0..180 {
        let sound = tone(
            (picture % 16) as u8,
            (picture % 32) as u8,
            if (40..90).contains(&picture) { 0 } else { 9 },
        );
        let samples = pictures.render(&sound);
        generated += samples.len();
        let mut expected = vec![0.0; samples.len()];
        reference.render(&sound, &mut expected);
        assert_eq!(samples, expected);
        assert_eq!(generated, ((picture + 1) * 44_101) / 60);
    }
    assert_eq!(generated, 44_101 * 3);
}

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
