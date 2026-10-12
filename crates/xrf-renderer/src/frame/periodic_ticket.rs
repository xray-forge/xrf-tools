/// A job offered to a frame's `PeriodicScheduler`, which asks whether it was granted.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct PeriodicTicket {
  pub(crate) frame: u64,
  pub(crate) index: usize,
}
