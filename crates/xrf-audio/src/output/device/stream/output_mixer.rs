use xrf_collections::rtrb::Consumer;

use crate::mixer::audio_mixer::AudioMixer;

/// Where the output thread's one mixer is: with the thread, lent to a stream's callback, or on its way back from a
/// callback the device has not let go of yet, through the ring the callback hands it back by. One handed back late is
/// taken up on a later look, however late, so audio plays again once it comes.
pub(crate) struct OutputMixer {
  /// The mixer while the thread holds it.
  mixer: Option<AudioMixer>,
  /// Where a stream's callback hands it back, while the device keeps it past the wait.
  returning: Option<Consumer<AudioMixer>>,
}

impl OutputMixer {
  pub(crate) fn new(mixer: AudioMixer) -> Self {
    Self {
      mixer: Some(mixer),
      returning: None,
    }
  }

  /// The mixer while the thread holds it, to render with no device.
  pub(crate) fn get_mut(&mut self) -> Option<&mut AudioMixer> {
    self.mixer.as_mut()
  }

  /// Lends the mixer to a stream, none while a stream or a device still holds it.
  pub(crate) fn lend(&mut self) -> Option<AudioMixer> {
    self.mixer.take()
  }

  /// Takes back what a stream answered as it closed or failed to open: the mixer, or the ring it comes back through
  /// later. Answers whether the thread holds it now.
  pub(crate) fn take_back(&mut self, returned: Result<AudioMixer, Consumer<AudioMixer>>) -> bool {
    match returned {
      Ok(mixer) => self.mixer = Some(mixer),
      Err(returning) => self.returning = Some(returning),
    }

    self.mixer.is_some()
  }

  /// Takes up a mixer a device handed back late; answers whether one came.
  pub(crate) fn take_up_late(&mut self) -> bool {
    let Some(mixer) = self.returning.as_mut().and_then(|returning| returning.pop().ok()) else {
      return false;
    };

    self.mixer = Some(mixer);
    self.returning = None;

    true
  }
}
