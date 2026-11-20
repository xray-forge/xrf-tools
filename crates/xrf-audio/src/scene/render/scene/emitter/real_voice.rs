use crate::decode::stream::stream_control::StreamControl;
use crate::voice::audio_voice_id::AudioVoiceId;

/// The voice an emitter holds, its stream's control where it streams, and where it should be by now: reckoned from the
/// source frame it started at, at the rate it was last set to, its Doppler shift included, from the mixer frame that
/// rate was set at.
pub(crate) struct RealVoice {
  pub(crate) id: AudioVoiceId,
  pub(crate) stream: Option<StreamControl>,
  /// The source frame due at mixer frame `reckoned_at`, unwrapped past the loop seam.
  pub(crate) due: f64,
  pub(crate) reckoned_at: u64,
  /// The rate it plays at, its emitter's speed times its Doppler shift.
  pub(crate) rate: f32,
  /// Whether the acoustics place it.
  pub(crate) is_placed: bool,
}

impl RealVoice {
  /// The source frame due at `mixer_frame`, its sound at `sample_rate` played by a mixer at `mixer_rate`; a mixer frame
  /// before the one it starts at is its start.
  pub(crate) fn get_due(&self, mixer_frame: u64, sample_rate: u32, mixer_rate: u32) -> f64 {
    let played: u64 = mixer_frame.saturating_sub(self.reckoned_at);

    self.due + played as f64 * f64::from(sample_rate) * f64::from(self.rate) / f64::from(mixer_rate.max(1))
  }

  /// Takes `rate` from `mixer_frame` on, what was due by then kept.
  pub(crate) fn reckon(&mut self, mixer_frame: u64, sample_rate: u32, mixer_rate: u32, rate: f32) {
    if mixer_frame > self.reckoned_at {
      self.due = self.get_due(mixer_frame, sample_rate, mixer_rate);
      self.reckoned_at = mixer_frame;
    }

    self.rate = rate;
  }
}
