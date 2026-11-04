//! What places a voice through the acoustics: Steam Audio's effects where the build has them; a type with no values
//! otherwise, so the voice's code is the same in both builds and a dry build never holds one.

#[cfg(feature = "steam-audio")]
pub(crate) use crate::acoustics::steam::voice_effects::VoiceEffects as VoiceSpatial;

#[cfg(not(feature = "steam-audio"))]
pub(crate) use dry::VoiceSpatial;

#[cfg(not(feature = "steam-audio"))]
mod dry {
  use crate::acoustics::effects::placed_block::PlacedBlock;
  use crate::voice::shared::voice_spatial_parameters::VoiceSpatialParameters;

  #[derive(Debug)]
  pub(crate) enum VoiceSpatial {}

  impl VoiceSpatial {
    pub(crate) fn apply(&mut self, _: &VoiceSpatialParameters, _: &[f32]) -> PlacedBlock<'_> {
      match *self {}
    }
  }
}
