use serde::Serialize;

/// One part file a tool has beside its scene, whether the editor loads it or skips it.
#[cfg_attr(feature = "typescript-bindings", derive(specta::Type))]
#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ScenePartFileDescription {
  /// `<tool>.part`, or a numbered one after it.
  pub name: String,
  /// Its size in bytes.
  pub size: u64,
  /// The GUID of the scene it was saved for, `g0:g1`; none for a file that does not read as a part.
  pub guid: Option<String>,
}
