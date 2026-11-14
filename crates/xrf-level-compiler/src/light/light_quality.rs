use serde::{Deserialize, Serialize};

/// How finely a level is lit: by `build.prj`'s own settings, or by a cheaper preset that regenerates what it changes.
#[cfg_attr(feature = "typescript-bindings", derive(specta::Type))]
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum LightQuality {
  /// `build.prj`'s own: its sky and sun lights as the editor generated them, its sample counts.
  #[default]
  Scene,
  /// The cheap estimate: the sky and the sun regenerated at quality 1 (26 and 9 lights), each multi-use model lit from
  /// one sample.
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
  pub fn get_sky_quality(self) -> Option<u8> {
    match self {
      Self::Scene => None,
      Self::Draft => Some(1),
    }
  }

  /// Whether the multi-use models are lit from one sample, as xrLC lights them at its draft quality: always at
  /// [`Self::Draft`], else where `build.prj` asks for a draft (`is_project_draft`).
  pub fn is_draft(self, is_project_draft: bool) -> bool {
    self == Self::Draft || is_project_draft
  }
}
