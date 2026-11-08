use std::time::{Duration, Instant};

/// A start that failed: why, when, and how long until it is tried again, which doubles each time it fails for the same
/// reason, up to `LONGEST`, so a cause that does not go away costs a try a minute rather than one every few seconds.
#[derive(Debug)]
pub(crate) struct RetryBackoff {
  reason: String,
  failed: Instant,
  wait: Duration,
}

impl RetryBackoff {
  /// The wait after a first failure, or one for another reason.
  pub(crate) const FIRST: Duration = Duration::from_secs(2);
  /// The longest wait between tries.
  pub(crate) const LONGEST: Duration = Duration::from_secs(64);

  pub(crate) fn new(reason: String) -> Self {
    Self {
      reason,
      failed: Instant::now(),
      wait: Self::FIRST,
    }
  }

  /// Notes another failure: the wait doubled for the same reason, the first again for another; whether the reason is
  /// new, which is when it is worth saying.
  pub(crate) fn fail(&mut self, reason: String) -> bool {
    let is_new: bool = reason != self.reason;

    self.wait = if is_new {
      Self::FIRST
    } else {
      (self.wait * 2).min(Self::LONGEST)
    };
    self.reason = reason;
    self.failed = Instant::now();

    is_new
  }

  /// Whether the wait since the last failure is over.
  pub(crate) fn is_due(&self) -> bool {
    self.failed.elapsed() >= self.wait
  }

  #[cfg(test)]
  pub(crate) fn get_wait(&self) -> Duration {
    self.wait
  }
}
