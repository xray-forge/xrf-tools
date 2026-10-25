use serde::Serialize;

/// Stable identity of a mount within one VFS.
///
/// Labels need not be unique, so scopes select mounts by this identifier.
#[cfg_attr(feature = "typescript-bindings", derive(specta::Type))]
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize)]
pub struct XrayMountId(pub(crate) usize);
