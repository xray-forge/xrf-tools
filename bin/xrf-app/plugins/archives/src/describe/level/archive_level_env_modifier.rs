use serde::Serialize;
use xrf_level::EnvModifier;

/// One local weather override of a level, as the viewer reads it.
#[cfg_attr(feature = "typescript-bindings", derive(specta::Type))]
#[derive(Clone, Debug, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ArchiveLevelEnvModifier {
  /// How far the override reaches, falling off linearly to that edge.
  pub radius: f32,
  /// How much of itself it mixes in at the centre.
  pub power: f32,
  pub far_plane: f32,
  pub fog_density: f32,
  /// Which of its values the engine mixes in, named. A file below version `0x0016` carries no flag word and the
  /// engine mixes in all of them, which is what `use_flags.one()` does before the read.
  pub used_parameters: Vec<String>,
  /// Whether the file said which values to use, rather than the reader assuming all of them.
  pub declares_parameters: bool,
}

impl ArchiveLevelEnvModifier {
  /// Every modifier of a level, in the order the file lists them.
  pub fn of_all(modifiers: &[EnvModifier]) -> Vec<Self> {
    modifiers.iter().map(Self::of).collect()
  }

  /// One modifier, taken over what it overrides.
  fn of(modifier: &EnvModifier) -> Self {
    Self {
      radius: modifier.radius,
      power: modifier.power,
      far_plane: modifier.far_plane,
      fog_density: modifier.fog_density,
      used_parameters: modifier
        .get_used_parameters()
        .into_iter()
        .map(ToOwned::to_owned)
        .collect(),
      declares_parameters: modifier.use_flags.is_some(),
    }
  }
}
