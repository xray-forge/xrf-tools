/// A part kept with a session's snapshot: built from it the first time anything asks and let go with it, so a value
/// opened in a session carries what its features derive from it without a field for each.
pub(crate) trait SessionPart: 'static {
  /// What the part keeps, cloned out to each ask: commonly the part shared, or why it could not be built.
  type Value: Clone + Send + Sync + 'static;

  /// What a change event calls the part when what it was built from moved, so a subscriber knows what to refetch.
  const NAME: &'static str;

  /// Whether the part is built as a [`xrf_job::PoolOnceLock`] builds, on a thread of its own, so the workers of a pool
  /// may wait on it while its build fans out over Rayon, which deadlocks a plain lock.
  const IS_POOLED: bool = true;
}
