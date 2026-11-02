use crate::progress::JobProgress;
use crate::progress::ProgressSink;

/// Discards every snapshot.
///
/// What a job reporting to nobody holds, so an operation never branches on whether anyone is watching.
#[derive(Default)]
pub struct NoopSink;

impl ProgressSink for NoopSink {
  fn report(&self, _: &JobProgress) {}
}
