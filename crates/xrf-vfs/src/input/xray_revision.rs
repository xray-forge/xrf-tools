use serde::Serialize;

/// When an input last changed, as the project revision of the change: zero for what is as it was mounted, which every
/// archived entry always is. Written as the bare number.
#[cfg_attr(feature = "typescript-bindings", derive(specta::Type))]
#[derive(Clone, Copy, Debug, Default, Eq, Hash, Ord, PartialEq, PartialOrd, Serialize)]
#[serde(transparent)]
pub struct XrayRevision(#[cfg_attr(feature = "typescript-bindings", specta(type = u64))] u64);

impl XrayRevision {
  /// The revision of everything as it was mounted.
  pub const MOUNTED: Self = Self(0);

  pub fn get(self) -> u64 {
    self.0
  }

  /// The revision the change after this one makes.
  pub fn next(self) -> Self {
    Self(self.0 + 1)
  }

  /// The revision of the change numbered `revision`.
  pub(crate) const fn from_count(revision: u64) -> Self {
    Self(revision)
  }
}
