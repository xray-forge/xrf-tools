use serde::{Deserialize, Serialize};

/// An object's update packet as the root spawn stores it, where it differs from the one a fresh entity read from its
/// spawn packet holds: the scene keeps the spawn packet alone, so this is what a root spawn rebuilt from it would lose.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LevelSceneUpdatePacket {
  pub object: String,
  /// The packet's bytes past its length and `M_UPDATE`, lowercase hex.
  pub bytes: String,
}
