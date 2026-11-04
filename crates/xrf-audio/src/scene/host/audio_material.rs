/// How a surface treats the sound reaching it, in three bands (below 800 Hz, 800 Hz to 8 kHz, above 8 kHz), as Steam
/// Audio's materials and the game's material acoustics (`SGameMtl::MtlAcoustics`) both hold it.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct AudioMaterial {
  /// The share of sound energy each band loses where it reflects, from 0 to 1.
  pub absorption: [f32; 3],
  /// The share of reflected sound scattered rather than mirrored, from 0 to 1.
  pub scattering: f32,
  /// The share of each band passing through, from 0 to 1.
  pub transmission: [f32; 3],
}

impl Default for AudioMaterial {
  /// Steam Audio's generic material.
  fn default() -> Self {
    Self {
      absorption: [0.10, 0.20, 0.30],
      scattering: 0.05,
      transmission: [0.100, 0.050, 0.030],
    }
  }
}
