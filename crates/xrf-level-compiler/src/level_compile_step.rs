use serde::{Deserialize, Serialize};

/// One step of a level's compile from the files `level build` writes, in the order they run: the geometry (xrLC's
/// part, lightmaps included), the details' light (xrDO_Light's) and the AI map (xrAI's `-f`). Each runs the SDK's own
/// compiler until a compiler of ours takes its place.
#[cfg_attr(feature = "typescript-bindings", derive(specta::Type))]
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum LevelCompileStep {
  Geometry,
  Details,
  Ai,
}

impl LevelCompileStep {
  /// Every step, in the order a compile runs them.
  pub const ALL: [Self; 3] = [Self::Geometry, Self::Details, Self::Ai];

  /// The step's name on a command line, `geometry`, `details` or `ai`.
  pub fn get_name(self) -> &'static str {
    match self {
      Self::Geometry => "geometry",
      Self::Details => "details",
      Self::Ai => "ai",
    }
  }

  /// The step a command line names, by [`Self::get_name`]; none for another name.
  pub fn from_name(name: &str) -> Option<Self> {
    Self::ALL.into_iter().find(|step| step.get_name() == name)
  }

  /// The files of the level's directory the step reads, which must be there before it runs: the build project, and
  /// for the details and the AI map what the geometry's compile leaves (the collision form, and the lights the details
  /// are lit by).
  pub fn get_inputs(self) -> &'static [&'static str] {
    match self {
      Self::Geometry => &["build.prj"],
      Self::Details => &["build.prj", "build.details", "build.cform", "build.lights"],
      Self::Ai => &["build.prj", "build.aimap", "build.cform"],
    }
  }
}
