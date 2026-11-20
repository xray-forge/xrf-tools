use serde::Serialize;
use xrf_editable_level::EditableWayLink;

/// A link from one way point to another of its way.
#[cfg_attr(feature = "typescript-bindings", derive(specta::Type))]
#[derive(Clone, Copy, Debug, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct SceneWayLink {
  /// The point it leads to, by its place in the way's points.
  pub to: u32,
  pub probability: f32,
}

impl From<&EditableWayLink> for SceneWayLink {
  fn from(link: &EditableWayLink) -> Self {
    Self {
      to: link.to as u32,
      probability: link.probability,
    }
  }
}
