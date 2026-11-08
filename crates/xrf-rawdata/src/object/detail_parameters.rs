use serde::{Deserialize, Serialize};

/// What a detail model carries beside its geometry that an editable object does not hold: the Level Editor keeps them
/// with its `EDetail` entry for the object (`detail_object.part`), and `EDetail::Export` writes them into the `.dm`.
#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DetailParameters {
  /// `EDetail::m_Flags`: `DO_NO_WAVING` (1) keeps the wind from swaying the object.
  pub flags: u32,
  /// The range a planted instance's scale is drawn from.
  pub min_scale: f32,
  pub max_scale: f32,
}

impl Default for DetailParameters {
  /// `EDetail`'s, which every vanilla `.dm` carries.
  fn default() -> Self {
    Self {
      flags: 0,
      min_scale: 0.5,
      max_scale: 2.0,
    }
  }
}
