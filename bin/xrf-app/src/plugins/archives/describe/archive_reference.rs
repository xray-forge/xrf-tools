use serde::Serialize;
use xrf_vfs::{XrayAssetRules, XrayAssetType, XrayLogicalPath};

use crate::plugins::archives::describe::archive_describe_source::ArchiveDescribeSource;

/// Whether the subject being browsed holds what a description named.
#[cfg_attr(feature = "typescript-bindings", derive(specta::Type))]
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub enum ArchiveReferenceStatus {
  Present,
  Absent,
  Unknown,
}

/// One file a description names, and what became of it.
#[cfg_attr(feature = "typescript-bindings", derive(specta::Type))]
#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ArchiveReference {
  /// The name as the file authored it: engine style, and without the extension the loader implies.
  pub name: String,
  /// The engine path the name was looked up as, or `None` for a name no logical path can be made of.
  pub path: Option<String>,
  /// The name the open subject lists the file under, which is what a surface selects in the tree.
  pub entry: Option<String>,
  pub status: ArchiveReferenceStatus,
}

impl ArchiveReference {
  /// Resolves a reference the way the engine addresses `asset_type`, against the subject being browsed.
  pub fn resolve(source: &ArchiveDescribeSource, asset_type: XrayAssetType, name: &str) -> Self {
    match to_engine_path(asset_type, name) {
      Some(path) => Self::of_engine_path(source, name.to_owned(), path),
      None => Self::unresolvable(name),
    }
  }

  /// Looks up an engine path already in hand, for a file addressed by where it sits rather than by a name.
  pub fn of_path(source: &ArchiveDescribeSource, path: &str) -> Self {
    Self::of_engine_path(source, path.to_owned(), path.to_owned())
  }

  /// A name no lookup could be made of, so none was attempted.
  pub fn unresolvable(name: &str) -> Self {
    Self {
      name: name.to_owned(),
      path: None,
      entry: None,
      status: ArchiveReferenceStatus::Unknown,
    }
  }

  fn of_engine_path(source: &ArchiveDescribeSource, name: String, path: String) -> Self {
    let entry: Option<String> = source.find_entry(&path);

    Self {
      name,
      status: if entry.is_some() {
        ArchiveReferenceStatus::Present
      } else {
        ArchiveReferenceStatus::Absent
      },
      path: Some(path),
      entry,
    }
  }
}

/// The engine identity a reference of `asset_type` names, or `None` when it cannot be made into one.
fn to_engine_path(asset_type: XrayAssetType, reference: &str) -> Option<String> {
  let rules: XrayAssetRules = asset_type.get_rules()?;

  XrayLogicalPath::new(&format!("{}\\{}", rules.directory, rules.to_logical_path(reference)))
    .ok()
    .map(|path| path.as_str().to_owned())
}

#[cfg(test)]
mod tests {
  use xrf_vfs::XrayAssetType;

  use super::to_engine_path;

  #[test]
  fn a_texture_reference_takes_its_directory_and_extension_from_the_kind() {
    assert_eq!(
      to_engine_path(XrayAssetType::Dds, "act\\act_arm_1_bump").as_deref(),
      Some("textures\\act\\act_arm_1_bump.dds")
    );
  }

  #[test]
  fn a_reference_is_normalized_to_the_identity_the_engine_registers() {
    assert_eq!(
      to_engine_path(XrayAssetType::Dds, "ACT/Act_Arm_1").as_deref(),
      Some("textures\\act\\act_arm_1.dds")
    );
  }

  #[test]
  fn a_name_that_is_not_a_path_makes_no_lookup() {
    // Engine text is authored by hand, and folding these into a path would report a reference nobody can resolve as
    // one that resolves somewhere.
    for name in ["act\\..\\..\\secret", "act\\\\arm", ".\\act\\arm"] {
      assert_eq!(to_engine_path(XrayAssetType::Dds, name), None, "'{name}' is not a path");
    }
  }
}
