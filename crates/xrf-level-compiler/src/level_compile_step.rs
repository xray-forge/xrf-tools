use serde::{Deserialize, Serialize};

use crate::ai::ai_map_covers::AiMapCovers;

/// One step of a level's compile from the files `level build` writes, in the order they run: the geometry (xrLC's
/// part, lightmaps included), the details' light (xrDO_Light's, done by [`crate::DetailLighter`]) and the AI map
/// (xrAI's `-f`, done by [`crate::AiMapCompiler`]). A step no compiler of ours does yet runs the SDK's own
/// ([`Self::is_sdk`]).
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

  /// Whether the SDK's own compiler runs the step, needing the SDK, rather than a compiler of ours.
  pub fn is_sdk(self) -> bool {
    self == Self::Geometry
  }

  /// The files of the level's directory the step reads, which must be there before it runs, the AI map's with
  /// `ai_covers`: the build project, and for the details and full covers what the geometry's compile leaves (the
  /// collision form, and the lights the details are lit by). An AI map with no covers reads its nodes alone.
  pub fn get_inputs(self, ai_covers: AiMapCovers) -> &'static [&'static str] {
    match (self, ai_covers) {
      (Self::Geometry, _) => &["build.prj"],
      (Self::Details, _) => &["build.prj", "build.details", "build.cform", "build.lights"],
      (Self::Ai, AiMapCovers::Full) => &["build.prj", "build.aimap", "build.cform"],
      (Self::Ai, AiMapCovers::None) => &["build.aimap"],
    }
  }

  /// Whether the step reads the textures `build.prj` names, and so needs a `.thm` for each, with `ai_covers`: every
  /// step but an AI map with no covers.
  pub fn is_reading_textures(self, ai_covers: AiMapCovers) -> bool {
    self == Self::Geometry || self.is_tracing(ai_covers)
  }

  /// Whether a compiler of ours traces the step's rays through the ray model, with `ai_covers`, reading how each face
  /// meets a ray from the game data's `shaders_xrlc.xr`: the details, and an AI map with full covers.
  pub fn is_tracing(self, ai_covers: AiMapCovers) -> bool {
    match self {
      Self::Geometry => false,
      Self::Details => true,
      Self::Ai => ai_covers == AiMapCovers::Full,
    }
  }
}
