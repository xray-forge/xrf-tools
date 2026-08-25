use xrf_error::XrfResult;

use crate::path::normalize;
use crate::{XrayMount, XrayMountId, XraySourceKind};

/// Which mounts a lookup scope admits.
#[derive(Clone, Debug, Default, Hash, PartialEq, Eq)]
pub(crate) enum XrayMountSelection {
  /// Every mount, in mount order.
  #[default]
  All,
  /// Only the named mounts, still visited in mount order rather than in the order given.
  Only(Vec<XrayMountId>),
  /// Only mounts that can be written to, which excludes every archive.
  Writable,
  /// Only mounts of one kind.
  OfKind(XraySourceKind),
}

/// Where one [`crate::XrayVfs`] operation is allowed to look: which mounts, and which logical subtree of them.
///
/// Both halves narrow the same search. `all()` is every mount and the whole tree; `with_prefix("configs")` keeps the mounts
/// and restricts the paths; `writable()` keeps the tree and restricts the mounts. Passing one of these instead of
/// hand-filtering results is what lets a config project and an asset lookup share one VFS — they differ only in scope.
#[derive(Clone, Debug, Default, Hash, PartialEq, Eq)]
pub struct XrayLookupScope {
  selection: XrayMountSelection,
  prefix: Option<String>,
}

impl XrayLookupScope {
  /// Selects every mount in priority order.
  pub fn all() -> Self {
    Self::default()
  }

  /// Selects only writable mounts.
  pub fn writable() -> Self {
    Self {
      selection: XrayMountSelection::Writable,
      ..Self::default()
    }
  }

  /// Selects the named mounts while preserving VFS priority order.
  pub fn only(mounts: impl IntoIterator<Item = XrayMountId>) -> Self {
    Self {
      selection: XrayMountSelection::Only(mounts.into_iter().collect()),
      ..Self::default()
    }
  }

  /// Selects mounts of one storage kind.
  pub fn of_kind(kind: XraySourceKind) -> Self {
    Self {
      selection: XrayMountSelection::OfKind(kind),
      ..Self::default()
    }
  }

  /// Restricts this scope to a normalized logical subtree such as `configs` or `textures\wpn`.
  ///
  /// # Errors
  ///
  /// Returns an error when `prefix` is not a valid X-Ray logical path.
  pub fn with_prefix(mut self, prefix: &str) -> XrfResult<Self> {
    self.prefix = Some(normalize(prefix)?.into_owned());

    Ok(self)
  }

  /// Restricts this scope to a subtree only when there is one to restrict to.
  ///
  /// Absence is a state this scope already holds, but [`Self::with_prefix`] cannot express it: an
  /// empty string is not a logical path and is refused. Callers whose prefix is configurable —
  /// a layout that may name none, a command flag left unset — would otherwise each invent their own
  /// spelling of "no prefix means the whole root", and three of them did.
  ///
  /// # Errors
  ///
  /// Returns an error when a prefix is given and is not a valid X-Ray logical path.
  pub fn with_optional_prefix(self, prefix: Option<&str>) -> XrfResult<Self> {
    match prefix.filter(|prefix| !prefix.is_empty()) {
      Some(prefix) => self.with_prefix(prefix),
      None => Ok(self),
    }
  }

  pub(crate) fn get_selection(&self) -> &XrayMountSelection {
    &self.selection
  }

  /// Returns the normalized logical subtree restriction, if any.
  pub fn get_prefix(&self) -> Option<&str> {
    self.prefix.as_deref()
  }

  /// Checks whether a mount matches this scope's selection.
  pub fn includes(&self, mount: &XrayMount) -> bool {
    match &self.selection {
      XrayMountSelection::All => true,
      XrayMountSelection::Only(ids) => ids.contains(&mount.get_id()),
      XrayMountSelection::Writable => mount.is_writable(),
      XrayMountSelection::OfKind(kind) => mount.get_kind() == *kind,
    }
  }
}

#[cfg(test)]
mod tests {
  use super::XrayMountSelection;
  use crate::{XrayLookupScope, XraySourceKind};

  #[test]
  fn defaults_to_everything() {
    assert_eq!(XrayLookupScope::all().get_selection(), &XrayMountSelection::All);
    assert_eq!(XrayLookupScope::all().get_prefix(), None);
  }

  #[test]
  fn normalizes_a_prefix_the_way_a_logical_path_is_normalized() {
    let scope: XrayLookupScope = XrayLookupScope::all()
      .with_prefix("Configs/Weapons")
      .expect("prefix is valid");

    assert_eq!(scope.get_prefix(), Some("configs\\weapons"));
  }

  #[test]
  fn rejects_an_ambiguous_prefix() {
    assert!(XrayLookupScope::all().with_prefix("configs/../textures").is_err());
  }

  #[test]
  fn selects_by_kind() {
    assert_eq!(
      XrayLookupScope::of_kind(XraySourceKind::Archive).get_selection(),
      &XrayMountSelection::OfKind(XraySourceKind::Archive)
    );
  }
}
