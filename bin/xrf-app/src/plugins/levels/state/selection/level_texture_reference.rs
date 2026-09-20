use serde::Serialize;

/// One texture a level's shader table names, and what it came to.
#[cfg_attr(feature = "typescript-bindings", derive(specta::Type))]
#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct LevelTextureReference {
  /// The reference as the shader table spells it, which is what a surface names.
  pub reference: String,
  /// What it resolved to, or `None` for a reference the roots hold nothing for.
  pub logical_path: Option<String>,
}
