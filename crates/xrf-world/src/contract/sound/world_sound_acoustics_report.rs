use serde::{Deserialize, Serialize};

/// How the host's acoustics shape a scene's sound: whether they run or the scene plays dry and why, the walls they
/// trace, and the reverb they hear around the listener.
#[cfg_attr(feature = "typescript-bindings", derive(specta::Type))]
#[derive(Clone, Debug, Default, Deserialize, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct WorldSoundAcousticsReport {
  /// Why the scene plays dry, placed by distance and panning alone; none while the acoustics run.
  pub failure: Option<String>,
  /// The triangles the acoustics trace, none while the level's walls are built or where there are none.
  pub triangles: Option<u32>,
  /// Why the walls shape nothing though the acoustics run.
  pub geometry_failure: Option<String>,
  /// The listener's reverb, each band's decay time in seconds (below 800 Hz, to 8 kHz, above).
  pub reverb_times: [f32; 3],
  /// How long the walls took to read and build, in milliseconds.
  pub build_ms: f32,
  /// How long the last simulation of the sounds' ways through the walls took, in milliseconds.
  pub direct_ms: f32,
  /// How long the last simulation of the reverb took, in milliseconds.
  pub reflections_ms: f32,
}
