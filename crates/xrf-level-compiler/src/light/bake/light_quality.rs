use serde::{Deserialize, Serialize};
use xrf_level::BuildPrjParams;

use crate::light::vertex::light_models::LightModels;

/// How finely a level is lit: by `build.prj`'s own settings, or by a cheaper preset that regenerates what it changes.
#[cfg_attr(feature = "typescript-bindings", derive(specta::Type))]
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum LightQuality {
  /// `build.prj`'s own: its sky and sun lights as the editor generated them, its sample counts.
  #[default]
  Scene,
  /// The cheap estimate: the sky and the sun regenerated at quality 1 (26 and 9 lights), the lightmaps 3 texels a metre
  /// of one sample each, each multi-use model lit from one sample.
  Draft,
}

impl LightQuality {
  /// Every choice, in the order a command line lists them.
  pub const ALL: [Self; 2] = [Self::Scene, Self::Draft];

  /// The choice's name on a command line, `scene` or `draft`.
  pub fn get_name(self) -> &'static str {
    match self {
      Self::Scene => "scene",
      Self::Draft => "draft",
    }
  }

  /// The choice a command line names, by [`Self::get_name`]; none for another name.
  pub fn from_name(name: &str) -> Option<Self> {
    Self::ALL.into_iter().find(|quality| quality.get_name() == name)
  }

  /// The quality the sky and the sun are regenerated at, `xrf_geometry::Sky`'s; none where `build.prj`'s are kept.
  pub(crate) fn get_sky_quality(self) -> Option<u8> {
    match self {
      Self::Scene => None,
      Self::Draft => Some(1),
    }
  }

  /// The lightmaps' texels a metre: 3 at [`Self::Draft`], else `build.prj`'s `project_density`.
  pub(crate) fn get_pixels_per_meter(self, project_density: f32) -> f32 {
    match self {
      Self::Scene => project_density,
      Self::Draft => 3.0,
    }
  }

  /// The samples a lightmap texel takes: 1 at [`Self::Draft`], else `build.prj`'s `project_samples`.
  pub(crate) fn get_jitter_samples(self, project_samples: u32) -> u32 {
    match self {
      Self::Scene => project_samples,
      Self::Draft => 1,
    }
  }

  /// The samples a multi-use model's vertex takes: xrLC's draft count, `LightModels::DRAFT_SAMPLES`, at
  /// [`Self::Draft`] or where `params` ask for a draft; else `LightModels::SAMPLES`.
  pub fn get_model_samples(self, params: &BuildPrjParams) -> u32 {
    if self == Self::Draft || params.is_draft() {
      LightModels::DRAFT_SAMPLES
    } else {
      LightModels::SAMPLES
    }
  }
}
