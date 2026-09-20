use serde::{Deserialize, Serialize};

/// Parsed `shader_name/texture,texture,...` level shader table entry.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LevelShaderReference {
  pub shader: String,
  pub textures: Vec<String>,
}
