use std::fmt::Display;

/// A test returning early because the host lacks what it checks: a device, a capability, a privilege. It says so on
/// stderr, which the test harness captures like any test output and shows with `--nocapture` or when the test fails.
pub struct TestSkip;

impl TestSkip {
  /// Says why the test skips; the test returns after it.
  #[expect(
    clippy::print_stderr,
    reason = "a skip's note is test output, which the test harness captures from stderr"
  )]
  pub fn report(reason: impl Display) {
    eprintln!("Skipped: {reason}");
  }
}
