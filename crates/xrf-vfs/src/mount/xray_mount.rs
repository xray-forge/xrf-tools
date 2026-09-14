use std::borrow::Cow;

use serde::Serialize;
use xrf_error::XrfResult;

use crate::path::{is_component_prefix, join, normalize, normalize_base};
use crate::{XrayAssetSource, XraySourceKind};

/// Stable identity of a mount within one VFS.
///
/// Labels need not be unique, so scopes select mounts by this identifier.
#[cfg_attr(feature = "typescript-bindings", derive(specta::Type))]
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize)]
pub struct XrayMountId(pub(crate) usize);

/// One source mounted at a logical base.
///
/// The base maps source-relative paths into the engine namespace. An empty base mounts a complete root; a base such as
/// `configs\weapons` mounts only that logical subtree.
#[derive(Debug)]
pub struct XrayMount {
  id: XrayMountId,
  base: String,
  /// How the plan that produced this mount described it, absent for a source mounted by hand.
  origin: Option<String>,
  source: Box<dyn XrayAssetSource>,
}

impl XrayMount {
  /// Creates a mount at a logical base, using an empty base for a complete root.
  ///
  /// # Errors
  ///
  /// Returns an error when a non-empty base is not a valid X-Ray logical path.
  pub fn new(id: XrayMountId, base: &str, source: Box<dyn XrayAssetSource>) -> XrfResult<Self> {
    Ok(Self {
      base: normalize_base(base)?,
      id,
      origin: None,
      source,
    })
  }

  /// Stable identity of this mount within its VFS, for scoping and remounting.
  pub fn get_id(&self) -> XrayMountId {
    self.id
  }

  /// How the plan that produced this mount described it: an `fsgame.ltx` alias such as `$game_data$`, or the
  /// constructor that planned it, as `root` or `volumes` for a path a person named directly.
  ///
  /// `None` for a source mounted by hand. A planned path an earlier plan already opened keeps that plan's name for it,
  /// because reuse does not re-decide why the mount exists.
  pub fn get_origin(&self) -> Option<&str> {
    self.origin.as_deref()
  }

  /// Records how a plan described this mount, keeping whatever an earlier plan already named it.
  pub(crate) fn set_origin(&mut self, origin: &str) {
    self.origin.get_or_insert_with(|| origin.to_string());
  }

  /// Returns the normalized logical base assigned to this mount.
  pub fn get_base(&self) -> &str {
    &self.base
  }

  /// Returns whether the mount stores loose files or archive entries.
  pub fn get_kind(&self) -> XraySourceKind {
    self.source.get_kind()
  }

  /// Returns whether writes through this mount can update existing entries.
  pub fn is_writable(&self) -> bool {
    self.source.is_writable()
  }

  /// Returns the source label used in diagnostics.
  pub fn get_label(&self) -> &str {
    self.source.get_label()
  }

  /// Borrows the mounted source for source-specific inspection.
  pub fn get_source(&self) -> &dyn XrayAssetSource {
    self.source.as_ref()
  }

  /// Converts a logical path to a source-relative path, or returns `None` when it lies outside the mount's base.
  ///
  /// Always a borrow of the caller's path: a root mount's source path *is* the logical path, and a based mount's is a
  /// tail of it. This runs once per mount on every lookup.
  pub(crate) fn to_source_path<'a>(&self, logical_path: &'a str) -> Option<&'a str> {
    if self.base.is_empty() {
      return Some(logical_path);
    }

    if !is_component_prefix(logical_path, &self.base) {
      return None;
    }

    Some(logical_path[self.base.len()..].trim_start_matches('\\'))
  }

  /// Applies this mount's base to a source-relative path.
  ///
  /// Borrowed for a root mount, where the source path already *is* the logical path — enumeration calls this once per
  /// entry, so copying each one to say nothing was the bulk of its allocation.
  pub(crate) fn to_logical_path<'a>(&self, source_path: &'a str) -> XrfResult<Cow<'a, str>> {
    if self.base.is_empty() {
      return normalize(source_path);
    }

    Ok(Cow::Owned(join(&self.base, source_path)?))
  }

  /// Translates a scope prefix into the source's namespace.
  ///
  /// Returns `None` when the scope cannot overlap this mount, and `Some(None)` when every source entry qualifies.
  pub(crate) fn to_source_prefix(&self, logical_prefix: Option<&str>) -> Option<Option<String>> {
    let Some(prefix) = logical_prefix else {
      return Some(None);
    };

    if self.base.is_empty() {
      return Some(Some(prefix.to_string()));
    }

    // The prefix reaches into this mount, so narrow it to the part below the base.
    if let Some(inner) = self.to_source_path(prefix) {
      return Some(if inner.is_empty() {
        None
      } else {
        Some(inner.to_string())
      });
    }

    // The base sits inside the requested prefix, so the whole mount qualifies.
    if is_component_prefix(&self.base, prefix) {
      return Some(None);
    }

    None
  }
}
