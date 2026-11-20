use serde::{Deserialize, Serialize};
use xrf_level::{BuildAimapFile, BuildCformFile, BuildPrjFile, LevelLightsFile};

use crate::ai::ai_map_covers::AiMapCovers;
use crate::detail::detail_step::DetailStep;

/// One step of a level's compile from the files `level build` writes, in run order: the geometry (xrLC's geometry
/// phases), its light (xrLC's light phases, baked in the geometry pass since the lightmap pages decide the visuals),
/// the details' light (xrDO_Light) and the AI map (xrAI's `-f`).
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

  /// The step that must run in the same compile: the geometry for the light, which cannot be laid over geometry
  /// already written.
  pub(crate) fn get_companion(self) -> Option<Self> {
    match self {
      Self::Light => Some(Self::Geometry),
      Self::Geometry | Self::Details | Self::Ai => None,
    }
  }

  /// The level directory's files the step needs before it runs: `build.prj`, plus what the geometry step leaves for
  /// the details and full AI covers. An AI map with no covers reads its nodes alone.
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

  /// Whether the step reads `build.prj`'s textures and so needs a `.thm` for each: all but an AI map with no covers.
  pub(crate) fn is_reading_textures(self, ai_covers: AiMapCovers) -> bool {
    matches!(self, Self::Geometry | Self::Light) || self.is_tracing(ai_covers)
  }

  /// Whether the step traces rays through the ray model, faces classed by `shaders_xrlc.xr`: the light, the details,
  /// and an AI map with full covers.
  pub(crate) fn is_tracing(self, ai_covers: AiMapCovers) -> bool {
    match self {
      Self::Geometry => false,
      Self::Light | Self::Details => true,
      Self::Ai => ai_covers == AiMapCovers::Full,
    }
  }

  /// Whether the step reads `shaders_xrlc.xr`: the geometry for its face flags, and every tracing step
  /// ([`Self::is_tracing`]).
  pub(crate) fn is_reading_library(self, ai_covers: AiMapCovers) -> bool {
    self == Self::Geometry || self.is_tracing(ai_covers)
  }
}
