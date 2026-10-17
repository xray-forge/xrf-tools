use serde::Serialize;

/// The profile a list was narrowed to, with what an NPC of it can be.
#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct DialogListProfile {
  pub id: String,
  /// The specific characters the profile resolves to: the pinned one, or every one it may pick by class.
  pub characters: Vec<String>,
}
