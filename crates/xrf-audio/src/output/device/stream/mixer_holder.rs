use std::time::Duration;

use rtrb::Producer;

use crate::mixer::audio_mixer::AudioMixer;

/// The mixer inside a device's callback: when the stream drops the callback, the mixer goes back to the output
/// thread, so the next stream plays the same voices.
pub(crate) struct MixerHolder {
  mixer: Option<AudioMixer>,
  returned: Producer<AudioMixer>,
}

impl MixerHolder {
  pub(crate) fn new(mixer: AudioMixer, returned: Producer<AudioMixer>) -> Self {
    Self {
      mixer: Some(mixer),
      returned,
    }
  }

  /// Renders the device's buffer `out`, which plays `latency` after it is handed over, anchoring the mixer's clock at
  /// its first frame.
  pub(crate) fn render(&mut self, out: &mut [f32], channels: usize, latency: Duration) {
    match &mut self.mixer {
      Some(mixer) => {
        mixer.get_meter().anchor_at(latency);
        mixer.render(out, channels);
      }
      None => out.fill(0.0),
    }
  }
}

impl Drop for MixerHolder {
  fn drop(&mut self) {
    if let Some(mixer) = self.mixer.take() {
      // The ring has room for this one mixer, which comes back once.
      let _ = self.returned.push(mixer);
    }
  }
}
