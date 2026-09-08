use serde::Serialize;

use crate::patch::compare::{ArchivePatchClass, ArchivePatchSide};

/// One entry a comparison classified, named by the identity both sides fold it to.
#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ArchivePatchChange {
  /// Engine identity: lower-case, backslash-separated, as `CLocatorAPI::Register` folds both spellings to.
  pub name: String,
  pub class: ArchivePatchClass,
  pub base: Option<ArchivePatchSide>,
  pub target: Option<ArchivePatchSide>,
}

impl ArchivePatchChange {
  /// An entry only the target holds.
  pub(crate) fn added(name: &str, target: ArchivePatchSide) -> Self {
    Self {
      name: name.to_owned(),
      class: ArchivePatchClass::Added,
      base: None,
      target: Some(target),
    }
  }

  /// An entry both hold, with differing payloads.
  pub(crate) fn modified(name: &str, base: ArchivePatchSide, target: ArchivePatchSide) -> Self {
    Self {
      name: name.to_owned(),
      class: ArchivePatchClass::Modified,
      base: Some(base),
      target: Some(target),
    }
  }

  /// An entry only the base holds.
  pub(crate) fn removed(name: &str, base: ArchivePatchSide) -> Self {
    Self {
      name: name.to_owned(),
      class: ArchivePatchClass::Removed,
      base: Some(base),
      target: None,
    }
  }
}
