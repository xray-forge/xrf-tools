use std::time::Duration;

/// How long [`ObjectVerifier`](crate::ObjectVerifier) took over each area it compared, the compile before them left
/// out.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct ObjectVerificationDurations {
  pub model: Duration,
  pub skeleton: Duration,
  pub motions: Duration,
  pub geometry: Duration,
}
