use xrf_utils::HostRelativePath;

/// One entry a tree write lands: what it is read from, and where it goes below the destination root.
pub(crate) struct TreeEntry<T> {
  /// Whatever the writer reads the payload out of: a volume descriptor, or a mounted world entry.
  pub(crate) source: T,
  /// Host-relative path below the destination root, already crossed out of engine spelling.
  pub(crate) relative: HostRelativePath,
  /// A directory record, which carries no payload and only needs its directory to exist.
  pub(crate) is_directory: bool,
}
