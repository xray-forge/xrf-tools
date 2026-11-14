use serde::{Deserialize, Serialize};
use xrf_level::{BuildAimapFile, BuildCformFile, BuildPrjFile, LevelLightsFile};

use crate::ai::ai_map_covers::AiMapCovers;
use crate::detail::detail_step::DetailStep;

/// One step of a level's compile from the files `level build` writes, in the order they run: the geometry (xrLC's
/// geometry phases), its light (xrLC's light phases, baked within the geometry step's pass, since the lightmap pages
/// decide the visuals), the details' light (xrDO_Light's, by `DetailLighter`) and the AI map (xrAI's `-f`, by
/// `AiMapCompiler`).
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

  /// The step that must run in the same compile for this one to: the geometry for its light, which cannot be laid
  /// over geometry already written; none for the others.
  pub(crate) fn get_companion(self) -> Option<Self> {
    match self {
      Self::Light => Some(Self::Geometry),
      Self::Geometry | Self::Details | Self::Ai => None,
    }
  }

  /// The files of the level's directory the step reads, which must be there before it runs, the AI map's with
  /// `ai_covers`: the build project, and for the details and full covers what the geometry's compile leaves (the
  /// collision form, and the lights the details are lit by). An AI map with no covers reads its nodes alone.
  pub(crate) fn get_inputs(self, ai_covers: AiMapCovers) -> &'static [&'static str] {
    match (self, ai_covers) {
      (Self::Geometry | Self::Light, _) => &[BuildPrjFile::FILE_NAME],
      (Self::Details, _) => &[
        BuildPrjFile::FILE_NAME,
        DetailStep::SOURCE,
        BuildCformFile::FILE_NAME,
        LevelLightsFile::FILE_NAME,
      ],
      (Self::Ai, AiMapCovers::Full) => &[
        BuildPrjFile::FILE_NAME,
        BuildAimapFile::FILE_NAME,
        BuildCformFile::FILE_NAME,
      ],
      (Self::Ai, AiMapCovers::None) => &[BuildAimapFile::FILE_NAME],
    }
  }

  /// Whether the step reads the textures `build.prj` names, and so needs a `.thm` for each, with `ai_covers`: every
  /// step but an AI map with no covers.
  pub(crate) fn is_reading_textures(self, ai_covers: AiMapCovers) -> bool {
    matches!(self, Self::Geometry | Self::Light) || self.is_tracing(ai_covers)
  }

  /// Whether a compiler of ours traces the step's rays through the ray model, with `ai_covers`, reading how each face
  /// meets a ray from the game data's `shaders_xrlc.xr`: the light, the details, and an AI map with full covers.
  pub(crate) fn is_tracing(self, ai_covers: AiMapCovers) -> bool {
    match self {
      Self::Geometry => false,
      Self::Light | Self::Details => true,
      Self::Ai => ai_covers == AiMapCovers::Full,
    }
  }

  /// Whether the step reads the game data's compiler shader library, `shaders_xrlc.xr`, with `ai_covers`: the geometry
  /// for each face's flags, and every step tracing the ray model ([`Self::is_tracing`]).
  pub(crate) fn is_reading_library(self, ai_covers: AiMapCovers) -> bool {
    self == Self::Geometry || self.is_tracing(ai_covers)
  }
}
