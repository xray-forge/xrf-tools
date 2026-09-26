use std::fmt::{Display, Formatter, Result as FormatResult};
use std::path::PathBuf;

use serde::{Serialize, Serializer};
use xrf_utils::{format_path, to_portable_path_string};

use crate::path::XrayLogicalPath;

/// Separator between a volume and the entry inside it.
const ENTRY_SEPARATOR: &str = "::";

/// Where one side of a collision physically sits.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum XrayCollisionSite {
  /// A file on the host filesystem.
  Loose(PathBuf),
  /// An entry of an archive volume's name table, named as authored.
  Archived {
    /// The volume file holding the entry.
    volume: PathBuf,
    /// Entry name as the volume's header records it, case included.
    name: String,
  },
}

impl XrayCollisionSite {
  /// Portable rendering for a report field, matching what [`xrf_utils::to_portable_path_string`] answers for a path.
  pub fn to_portable_string(&self) -> String {
    match self {
      Self::Loose(path) => to_portable_path_string(path),
      Self::Archived { volume, name } => {
        format!(
          "{}{ENTRY_SEPARATOR}{}",
          to_portable_path_string(volume),
          name.replace('\\', "/")
        )
      }
    }
  }
}

/// Serializes as the portable string, so one rendering reaches every consumer.
impl Serialize for XrayCollisionSite {
  fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
    serializer.serialize_str(&self.to_portable_string())
  }
}

/// Renders for a person, so the host half goes through the one formatting boundary rather than `Path::display`.
impl Display for XrayCollisionSite {
  fn fmt(&self, formatter: &mut Formatter<'_>) -> FormatResult {
    match self {
      Self::Loose(path) => write!(formatter, "{}", format_path(path)),
      Self::Archived { volume, name } => {
        write!(formatter, "{}{ENTRY_SEPARATOR}{name}", format_path(volume))
      }
    }
  }
}

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

  use serde_json::{Value, json};

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

  /// A separator is the one thing a report cannot inherit from the host, or the same run reads differently per platform.
  #[test]
  fn portable_strings_do_not_carry_the_host_separator() {
    let site: XrayCollisionSite = XrayCollisionSite::Loose(PathBuf::from("gamedata").join("textures").join("a.dds"));
    let rendered: Value = serde_json::to_value(&site).expect("a serializable site");

    assert_eq!(rendered, json!("gamedata/textures/a.dds"));
  }
}
