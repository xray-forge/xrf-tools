use std::time::{Duration, Instant};

use crate::frame::frame_pacer::FramePacer;

const INTERVAL: Option<Duration> = Some(Duration::from_millis(8));

#[test]
fn keeps_frames_due_in_step_and_presents_a_late_one_at_once() {
  let start: Instant = Instant::now();
  let mut pacer: FramePacer = FramePacer::default();

  pacer.begin(start, INTERVAL);
  assert_eq!(pacer.get_due(), Some(start));

  // Started a little early, as the lead asks: still due a step after the last.
  pacer.begin(start + Duration::from_millis(6), INTERVAL);
  assert_eq!(pacer.get_due(), Some(start + Duration::from_millis(8)));

  // Started past its due: due at once, the steps after kept from there.
  pacer.begin(start + Duration::from_millis(20), INTERVAL);
  assert_eq!(pacer.get_due(), Some(start + Duration::from_millis(20)));
}

#[test]
fn starts_the_next_frame_its_slowest_recent_work_ahead_of_its_due() {
  let start: Instant = Instant::now();
  let mut pacer: FramePacer = FramePacer::default();

  pacer.begin(start, INTERVAL);
  pacer.note_ready(Duration::from_millis(3));

  let lead: Duration = Duration::from_millis(3) + FramePacer::MARGIN;

  assert_eq!(pacer.get_lead(), lead);
  assert_eq!(pacer.get_start(INTERVAL), Some(start + Duration::from_millis(8) - lead));

  // A quicker frame after a slow one keeps most of the slow one's lead.
  pacer.note_ready(Duration::from_millis(1));
  assert_eq!(pacer.get_lead(), lead.mul_f32(FramePacer::LEAD_KEPT));
}

#[test]
fn neither_waits_nor_leads_without_a_limit() {
  let mut pacer: FramePacer = FramePacer::default();

  pacer.begin(Instant::now(), None);
  pacer.note_ready(Duration::from_millis(3));

  assert_eq!(pacer.get_due(), None);
  assert_eq!(pacer.get_start(None), None);
}
