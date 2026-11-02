use crate::progress::JobProgress;

/// Delivers a job's progress snapshots to whoever is watching.
///
/// One method, so a new sink is written once and cannot answer some snapshots while silently dropping others. Coalescing
/// is not a sink's concern: by the time a snapshot arrives here the handle has already decided it is worth sending, and
/// a sink that throttled again would make the rate a function of which sink happened to be attached.
///
/// Called from whichever thread tripped the emission gate, which for a parallel operation is an arbitrary worker. An
/// implementation that blocks blocks that worker.
pub trait ProgressSink: Send + Sync {
  fn report(&self, progress: &JobProgress);
}
