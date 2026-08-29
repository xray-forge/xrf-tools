use std::fmt::{Display, Formatter, Result as FormatResult};
use std::path::PathBuf;

use xrf_utils::{format_path, to_portable_path_string};

use crate::path::XrayLogicalPath;

/// Separator between a volume and the entry inside it.
///
/// Not a path separator: an archived entry has no host path, and joining the two the usual way would produce something a
/// reader could mistake for one and try to open.
const ENTRY_SEPARATOR: &str = "::";

/// Where one side of a collision physically sits.
///
/// A loose file is addressed by its host path. An archived entry is addressed by its volume plus the name that volume's
/// header authored, and the authored name is kept because it is exactly what case folding destroys — a person cannot fix
/// the archive without knowing which of the two spellings to remove.
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
///
/// An authoring error rather than shadowing: shadowing is what happens *between* mounts, where a loose file legitimately
/// overrides an archived one. Inside one source there is no priority to appeal to, so one file simply cannot be reached.
///
/// Reported rather than fatal, because a tool must be able to open a project and say what is wrong with it — an editor
/// cannot refuse to load a mod because one texture is authored twice. A consumer that treats a project as invalid decides
/// that for itself. The engine is not stricter: `CLocatorAPI::Register` folds a name to lower case before its lookup and
/// overwrites on a hit, so it resolves a collision silently rather than refusing the archive
/// (`xray-16/src/xrCore/LocatorAPI.cpp`).
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct XrayPathCollision {
  /// Engine identity both files normalize to.
  pub logical_path: XrayLogicalPath,
  /// File the source resolves.
  pub kept: XrayCollisionSite,
  /// File no lookup can reach, because `kept` already claims its identity.
  pub unreachable: XrayCollisionSite,
}
