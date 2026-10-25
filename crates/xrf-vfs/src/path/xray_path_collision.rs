use serde::Serialize;

use crate::path::{XrayCollisionSite, XrayLogicalPath};

/// Two files in one source claiming the same engine identity.
#[cfg_attr(feature = "typescript-bindings", derive(specta::Type))]
#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct XrayPathCollision {
  /// Engine identity both files normalize to.
  pub logical_path: XrayLogicalPath,
  /// File the source resolves.
  #[cfg_attr(feature = "typescript-bindings", specta(type = String))]
  pub kept: XrayCollisionSite,
  /// File no lookup can reach, because `kept` already claims its identity.
  #[cfg_attr(feature = "typescript-bindings", specta(type = String))]
  pub unreachable: XrayCollisionSite,
}

#[cfg(test)]
mod tests {
  use std::path::PathBuf;

  use serde_json::json;

  use super::{XrayCollisionSite, XrayPathCollision};
  use crate::path::XrayLogicalPath;

  /// The shape every surface reports, pinned here because it is the crate's answer rather than any one command's.
  #[test]
  fn reports_both_sites_as_portable_strings() {
    let collision: XrayPathCollision = XrayPathCollision {
      logical_path: XrayLogicalPath::new("textures\\a.dds").expect("a valid logical path"),
      kept: XrayCollisionSite::Archived {
        volume: PathBuf::from("game").join("db").join("textures.db0"),
        name: String::from("textures\\a.dds"),
      },
      unreachable: XrayCollisionSite::Loose(PathBuf::from("game").join("gamedata").join("textures").join("A.DDS")),
    };

    assert_eq!(
      serde_json::to_value(&collision).expect("a serializable collision"),
      json!({
        "logicalPath": "textures\\a.dds",
        "kept": "game/db/textures.db0::textures/a.dds",
        "unreachable": "game/gamedata/textures/A.DDS",
      })
    );
  }
}
