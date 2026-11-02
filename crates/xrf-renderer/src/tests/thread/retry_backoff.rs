use std::time::Duration;

use crate::thread::retry_backoff::RetryBackoff;

#[test]
fn doubles_its_wait_for_the_same_reason_up_to_the_longest_and_starts_over_for_another() {
  let mut backoff: RetryBackoff = RetryBackoff::new("no adapter".into());

  assert_eq!(backoff.get_wait(), RetryBackoff::FIRST);
  assert!(!backoff.is_due());
  assert!(!backoff.fail("no adapter".into()));
  assert_eq!(backoff.get_wait(), RetryBackoff::FIRST * 2);

  for _ in 0..10 {
    backoff.fail("no adapter".into());
  }

  assert_eq!(backoff.get_wait(), RetryBackoff::LONGEST);
  assert!(backoff.fail("a pass does not compile".into()));
  assert_eq!(backoff.get_wait(), Duration::from_secs(2));
}
