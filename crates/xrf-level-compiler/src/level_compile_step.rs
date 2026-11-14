use serde::{Deserialize, Serialize};

use crate::ai::ai_map_covers::AiMapCovers;

/// One step of a level's compile from the files `level build` writes, in the order they run: the geometry (xrLC's
/// part, done by [`crate::GeometryLevel`], or by the SDK's xrLC with its own light where the request names the SDK,
/// [`Self::is_sdk`]), its light (xrLC's baker, done with the geometry in one pass, since the lightmap pages decide the
/// visuals), the details' light (xrDO_Light's, done by [`crate::DetailLighter`]) and the AI map (xrAI's `-f`, done by
/// [`crate::AiMapCompiler`]).
#[cfg_attr(feature = "typescript-bindings", derive(specta::Type))]
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum LevelCompileStep {
  Geometry,
  Light,
  Details,
  Ai,
}

impl LevelCompileStep {
  /// Every step, in the order a compile runs them.
  pub const ALL: [Self; 4] = [Self::Geometry, Self::Light, Self::Details, Self::Ai];

  /// The step's name on a command line, `geometry`, `light`, `details` or `ai`.
  pub fn get_name(self) -> &'static str {
    match self {
      Self::Geometry => "geometry",
      Self::Light => "light",
      Self::Details => "details",
      Self::Ai => "ai",
    }
  }

  /// The step a command line names, by [`Self::get_name`]; none for another name.
  pub fn from_name(name: &str) -> Option<Self> {
    Self::ALL.into_iter().find(|step| step.get_name() == name)
  }

  /// Whether the SDK's own compiler can run the step, which it does where the request names the SDK. xrLC lights the
  /// geometry it compiles whatever the steps, so the light step is its geometry step's.
  pub fn is_sdk(self) -> bool {
    self == Self::Geometry
  }

  /// The step that must run in the same compile for this one to: the geometry for its light, which cannot be laid
  /// over geometry already written; none for the others.
  pub fn get_companion(self) -> Option<Self> {
    match self {
      Self::Light => Some(Self::Geometry),
      Self::Geometry | Self::Details | Self::Ai => None,
    }
  }

  /// The files of the level's directory the step reads, which must be there before it runs, the AI map's with
  /// `ai_covers`: the build project, and for the details and full covers what the geometry's compile leaves (the
  /// collision form, and the lights the details are lit by). An AI map with no covers reads its nodes alone.
  pub fn get_inputs(self, ai_covers: AiMapCovers) -> &'static [&'static str] {
    match (self, ai_covers) {
      (Self::Geometry | Self::Light, _) => &["build.prj"],
      (Self::Details, _) => &["build.prj", "build.details", "build.cform", "build.lights"],
      (Self::Ai, AiMapCovers::Full) => &["build.prj", "build.aimap", "build.cform"],
      (Self::Ai, AiMapCovers::None) => &["build.aimap"],
    }
  }

  /// Whether the step reads the textures `build.prj` names, and so needs a `.thm` for each, with `ai_covers`: every
  /// step but an AI map with no covers.
  pub fn is_reading_textures(self, ai_covers: AiMapCovers) -> bool {
    matches!(self, Self::Geometry | Self::Light) || self.is_tracing(ai_covers)
  }

  /// Whether a compiler of ours traces the step's rays through the ray model, with `ai_covers`, reading how each face
  /// meets a ray from the game data's `shaders_xrlc.xr`: the light, the details, and an AI map with full covers.
  pub fn is_tracing(self, ai_covers: AiMapCovers) -> bool {
    match self {
      Self::Geometry => false,
      Self::Light | Self::Details => true,
      Self::Ai => ai_covers == AiMapCovers::Full,
    }
  }
}
