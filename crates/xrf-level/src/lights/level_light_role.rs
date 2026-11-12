use serde::{Deserialize, Serialize};

/// What a list of `build.lights` lights, by the chunk xrLC writes it to (`base_lighting`, read by
/// `xrLC_Light/global_calculation_data.cpp:66`): the static lights' colour, the sky's hemisphere, or the sun.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum LevelLightRole {
  Rgb,
  Hemi,
  Sun,
}

impl LevelLightRole {
  /// Every role, in chunk order.
  pub const ALL: [Self; 3] = [Self::Rgb, Self::Hemi, Self::Sun];

  /// The chunk the role's lights are written to.
  pub const fn get_chunk_id(self) -> u32 {
    match self {
      Self::Rgb => 0,
      Self::Hemi => 1,
      Self::Sun => 2,
    }
  }
}
