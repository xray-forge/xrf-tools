use std::time::{Duration, Instant};

/// When a limited frame rate starts and presents its frames. Each is presented at a due time kept in step at the
/// interval, waiting out what is left of it once its work is done, so how long the work took does not move when it is
/// shown; and each is started a lead ahead of its due, the recent slowest start-to-ready time and a margin, so the wait
/// before it presents stays short.
#[derive(Debug, Default)]
pub struct FramePacer {
  /// When the frame drawn now is due to present, none without a limit.
  due: Option<Instant>,
  /// How far ahead of its due a frame starts.
  lead: Duration,
}

impl FramePacer {
  /// What a frame starts ahead of its slowest recent start-to-ready time by, which also takes in a start woken late.
  pub const MARGIN: Duration = Duration::from_millis(1);
  /// What the lead keeps of itself a frame, so a slow frame's lead lasts a second or two at the rates a limit offers.
  pub const LEAD_KEPT: f32 = 0.995;
  /// What is left of the wait before presenting that is spun out rather than slept, a sleep waking up to this late.
  pub const SPUN: Duration = Duration::from_micros(1500);

  /// When the next frame starts, a lead ahead of its due: none without a limit or before the first frame.
  pub fn get_start(&self, interval: Option<Duration>) -> Option<Instant> {
    let due: Instant = self.due? + interval?;

    Some(due.checked_sub(self.lead).unwrap_or(due))
  }

  /// When the frame drawn now is due to present, none without a limit.
  pub fn get_due(&self) -> Option<Instant> {
    self.due
  }

  pub fn get_lead(&self) -> Duration {
    self.lead
  }

  /// Begins a frame at `now` under a limit's `interval`: due a step after the last, kept in step so a frame started a
  /// little late does not push every later one, and never before now, so one started past its due presents at once.
  pub fn begin(&mut self, now: Instant, interval: Option<Duration>) {
    self.due = interval.map(|interval| self.due.map_or(now, |due| due + interval).max(now));
  }

  /// Notes that a frame was ready to present `took` after it started, which later frames start that far ahead of their
  /// due, or what a slower recent frame took.
  pub fn note_ready(&mut self, took: Duration) {
    self.lead = (took + Self::MARGIN).max(self.lead.mul_f32(Self::LEAD_KEPT));
  }

  /// Waits until the frame drawn now is due to present: slept, then spun for the last `SPUN`, as a sleep wakes late,
  /// yielding while it spins so another thread ready to run is not starved.
  pub fn wait_for_due(&self) {
    let Some(due) = self.due else {
      return;
    };

    loop {
      let left: Duration = due.saturating_duration_since(Instant::now());

      if left.is_zero() {
        return;
      }

      match left.checked_sub(Self::SPUN) {
        Some(sleep) if !sleep.is_zero() => std::thread::sleep(sleep),
        _ => std::thread::yield_now(),
      }
    }
  }
}
