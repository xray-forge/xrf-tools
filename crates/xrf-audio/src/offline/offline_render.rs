use crate::mixer::audio_mixer::AudioMixer;
use crate::mixer::audio_mixer_meter::AudioMixerMeter;

/// Renders a mixer without a device, as many frames as asked and as fast as the machine allows: what tests listen to.
pub struct OfflineRender {
  mixer: AudioMixer,
}

impl OfflineRender {
  pub fn new(mut mixer: AudioMixer, output_rate: u32) -> Self {
    mixer.set_output_rate(output_rate);

    Self { mixer }
  }

  pub fn get_output_rate(&self) -> u32 {
    self.mixer.get_output_rate()
  }

  /// The next `frames` stereo frames, interleaved.
  pub fn render(&mut self, frames: usize) -> Vec<f32> {
    let mut out: Vec<f32> = vec![0.0; frames * 2];

    self.render_into(&mut out);
    out
  }

  /// Fills `out` with the next stereo frames, interleaved, allocating nothing.
  pub fn render_into(&mut self, out: &mut [f32]) {
    self.mixer.render(out, 2);
  }

  /// The mixer's clock and load.
  pub fn get_meter(&self) -> &AudioMixerMeter {
    self.mixer.get_meter()
  }

  pub fn into_mixer(self) -> AudioMixer {
    self.mixer
  }
}
