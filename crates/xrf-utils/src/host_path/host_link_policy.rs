/// What a [`crate::HostDestination`] does with a link already present below its root.
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub enum HostLinkPolicy {
  /// Write through links, junctions and mod-manager overlays as the filesystem resolves them.
  #[default]
  Follow,
  /// Refuse to descend through or write over any link below the root; on Windows, any reparse point.
  Refuse,
}
