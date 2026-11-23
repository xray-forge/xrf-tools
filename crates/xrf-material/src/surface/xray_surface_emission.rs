use serde::Serialize;

/// The light a self-lit surface's special pass fills where it stands, which every light then adds to: a script whose
/// base pass says `emissive(true)` and whose `l_special` pass begins one of the engine's emissive programs.
#[cfg_attr(feature = "typescript-bindings", derive(specta::Type))]
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub enum XraySurfaceEmission {
  /// `accum_emissive.ps`, `models\selflight`'s: sixteen in every channel, its specular too, which washes its albedo
  /// white wherever the colour is not already one of its own.
  Bright,
  /// `accum_emissivel.ps`, `models\selflightl`'s: one in colour and none in specular, which shows its albedo as it is.
  Dim,
}

impl XraySurfaceEmission {
  /// The pixel program that fills a dim surface's light; any other a self-lit script names fills a bright one.
  const DIM_PROGRAM: &'static str = "accum_emissivel";

  /// What a self-lit script's special pass fills, by the pixel program it begins.
  pub(crate) fn of_program(pixel_program: &str) -> Self {
    if pixel_program.eq_ignore_ascii_case(Self::DIM_PROGRAM) {
      Self::Dim
    } else {
      Self::Bright
    }
  }

  /// The light it fills in each colour channel.
  pub const fn get_light(self) -> f32 {
    match self {
      Self::Bright => 16.0,
      Self::Dim => 1.0,
    }
  }
}
