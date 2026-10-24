/// The phases a job names the work it is doing by, where the work it was asked for is not yet under way or not the
/// only part of it.
pub struct JobPhase;

impl JobPhase {
  /// Getting ready: opening and reading what the work runs over.
  pub const PREPARE: &'static str = "prepare";

  /// Reading what the job converts.
  pub const READ: &'static str = "read";

  /// Writing what the job produced.
  pub const WRITE: &'static str = "write";
}
