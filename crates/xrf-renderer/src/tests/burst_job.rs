use std::sync::mpsc::{Receiver, channel};
use std::time::Duration;

use crate::capture::burst::burst_event::BurstEvent;
use crate::capture::burst::burst_job::BurstJob;
use crate::contract::render_burst::RenderBurst;

const BURST: RenderBurst = RenderBurst {
  at: 600.0,
  step: 0.25,
  lead_in: 2,
  frames: 3,
  samples: None,
};

fn start(burst: RenderBurst) -> (BurstJob, Receiver<BurstEvent>) {
  let (events, receiver) = channel();

  (BurstJob::new(burst, events).expect("the burst is valid"), receiver)
}

#[test]
fn refuses_a_burst_that_reads_nothing_back_or_steps_by_no_time() {
  for burst in [
    RenderBurst { frames: 0, ..BURST },
    RenderBurst {
      frames: BurstJob::MOST_FRAMES + 1,
      ..BURST
    },
    RenderBurst { step: 0.0, ..BURST },
    RenderBurst { at: -1.0, ..BURST },
    RenderBurst {
      samples: Some(0),
      ..BURST
    },
  ] {
    assert!(BurstJob::new(burst, channel().0).is_err(), "{burst:?}");
  }
}

// Its lead-in restarts the view and reads nothing back; each frame after is a step on, whatever it took to draw.
#[test]
fn draws_its_lead_in_then_its_frames_a_step_apart() {
  let (mut job, _events) = start(BURST);

  assert!(job.is_starting());
  assert_eq!(job.get_time(), Some(Duration::from_secs(600)));
  assert_eq!(job.get_kept_frame(), None);

  job.advance(None);

  assert!(!job.is_starting());
  assert_eq!(job.get_time(), Some(Duration::from_secs_f64(600.25)));

  job.advance(None);

  assert_eq!(job.get_kept_frame(), Some((0, None, 600.5)));
  assert_eq!(job.get_reference(), None);
}

// A frame read back but never copied out ends it: the frames after would no longer follow one another.
#[test]
fn stops_when_a_frame_read_back_was_not_copied() {
  let (mut job, events) = start(RenderBurst { lead_in: 0, ..BURST });

  job.advance(None);

  assert!(job.is_ended());
  assert!(matches!(events.try_recv(), Ok(BurstEvent::Stopped { .. })));
}

// A reference draws each frame at every sample, its time held over them; its lead-in at its first sample alone.
#[test]
fn a_reference_holds_its_time_over_a_frames_samples() {
  let (mut job, _events) = start(RenderBurst {
    lead_in: 1,
    samples: Some(4),
    ..BURST
  });

  assert_eq!(job.get_reference(), Some((0, 4)));

  job.advance(None);

  assert_eq!(job.get_reference(), Some((0, 4)));
  assert_eq!(job.get_kept_frame(), Some((0, Some(0), 600.25)));
  assert!(job.is_drawing());
}
