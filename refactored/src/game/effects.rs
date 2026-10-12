use super::Adventure;
use super::model::SoundEffect;

impl Adventure {
    // Duration advances before voice selection; victory also animates the retained room color.
    pub(super) fn play_game_sound(&mut self) {
        if self.world.sound_remaining == 0 {
            self.sound.silence();
            return;
        }
        self.world.sound_remaining = self.world.sound_remaining.wrapping_sub(1);
        let remaining = self.world.sound_remaining;
        let (waveform, frequency, volume) = match self.world.sound_effect {
            SoundEffect::Victory => {
                self.display.appearance.foreground = remaining;
                (remaining, remaining >> 3, remaining >> 1)
            }
            SoundEffect::Roar => (
                if remaining & 1 != 0 { 3 } else { 8 },
                (remaining >> 2).wrapping_add(28),
                remaining,
            ),
            SoundEffect::Swallow => (6, remaining ^ 15, (remaining >> 1).wrapping_add(8)),
            SoundEffect::DragonSlain => (4, remaining ^ 31, remaining),
            SoundEffect::Drop => (6, remaining ^ 3, 5),
            SoundEffect::Pickup => (6, remaining, 5),
        };
        self.sound.voice(waveform, frequency, volume);
    }
}
