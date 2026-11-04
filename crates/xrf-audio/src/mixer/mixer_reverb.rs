//! The listener's reverb on the mixer: Steam Audio's where the build has it; a type with no values otherwise, so the
//! mixer's code is the same in both builds and a dry build never holds one.

#[cfg(feature = "steam-audio")]
pub(crate) use crate::acoustics::steam::mixer_reverb::MixerReverb;

#[cfg(not(feature = "steam-audio"))]
pub(crate) use dry::MixerReverb;

#[cfg(not(feature = "steam-audio"))]
mod dry {
  use crate::acoustics::audio_acoustics::AudioAcoustics;
  use crate::acoustics::effects::ear_blocks::EarBlocks;

  pub(crate) enum MixerReverb {}

  impl MixerReverb {
    pub(crate) fn for_acoustics(_: &AudioAcoustics, _: u32) -> Option<Box<Self>> {
      None
    }

    pub(crate) fn apply(&mut self, _: [f32; 3], _: &[f32]) -> EarBlocks<'_> {
      match *self {}
    }
  }
}
