use std::sync::mpsc::Sender;
use std::time::Duration;

use xrf_error::{XrfError, XrfResult};

use crate::capture::burst::burst_event::BurstEvent;
use crate::capture::burst::burst_shot::BurstShot;
use crate::contract::render_burst::RenderBurst;

/// Where a burst stands: drawing one of its frames, the lead-in's counted, at one of its samples; or waiting for the
/// last frames back.
#[derive(Clone, Copy, Debug, PartialEq)]
enum BurstStage {
  Drawing { frame: u32, sample: u32 },
  Reading,
}

/// A burst drawn in a viewport through its own view, at times of its own that every scene and the world follow while
/// it draws: the frames' clock goes back to its own time once the last frame is drawn.
pub struct BurstJob {
  burst: RenderBurst,
  stage: BurstStage,
  /// The frames copied out and on their way back.
  in_flight: Vec<BurstShot>,
  events: Sender<BurstEvent>,
  is_ended: bool,
}

impl BurstJob {
  /// Frames a burst reads back at most, and samples a reference draws each at.
  pub const MOST_FRAMES: u32 = 480;
  pub const MOST_SAMPLES: u32 = 256;

  /// Frames on their way back at most: the next waits, its clock held, rather than hold more of them.
  const MOST_IN_FLIGHT: usize = 8;

  /// A burst as asked.
  ///
  /// # Errors
  ///
  /// Returns an error for a burst that reads back no frame or more than `MOST_FRAMES`, a lead-in longer than that, a
  /// reference of no sample or more than `MOST_SAMPLES`, a step that is not a positive time, or a start that is not a
  /// time of the scene's.
  pub fn new(burst: RenderBurst, events: Sender<BurstEvent>) -> XrfResult<Self> {
    if !(1..=Self::MOST_FRAMES).contains(&burst.frames) || burst.lead_in > Self::MOST_FRAMES {
      return Err(XrfError::new_invalid_error(format!(
        "A burst reads back 1 to {} frames after a lead-in of at most as many, not {} after {}",
        Self::MOST_FRAMES,
        burst.frames,
        burst.lead_in
      )));
    }

    if burst
      .samples
      .is_some_and(|samples| !(1..=Self::MOST_SAMPLES).contains(&samples))
    {
      return Err(XrfError::new_invalid_error(format!(
        "A reference burst draws each frame at 1 to {} samples",
        Self::MOST_SAMPLES
      )));
    }

    if !(burst.step.is_finite() && burst.step > 0.0 && burst.at.is_finite() && burst.at >= 0.0) {
      return Err(XrfError::new_invalid_error(format!(
        "A burst steps by a positive time from one of the scene's, not {} from {}",
        burst.step, burst.at
      )));
    }

    Ok(Self {
      burst,
      stage: BurstStage::Drawing { frame: 0, sample: 0 },
      in_flight: Vec::new(),
      events,
      is_ended: false,
    })
  }

  /// Whether it draws a frame this turn: it waits, its clock held and its viewport drawing nothing, while
  /// `MOST_IN_FLIGHT` frames are on their way back.
  pub fn is_drawing(&self) -> bool {
    !self.is_ended && matches!(self.stage, BurstStage::Drawing { .. }) && self.in_flight.len() < Self::MOST_IN_FLIGHT
  }

  /// Whether this turn's frame is its first, which restarts what its view carries from frame to frame.
  pub fn is_starting(&self) -> bool {
    self.is_drawing() && self.stage == BurstStage::Drawing { frame: 0, sample: 0 }
  }

  /// The sample a reference draws this turn's frame at and its samples, the lead-in's at its first; none for a burst
  /// the view resolves.
  pub fn get_reference(&self) -> Option<(u32, u32)> {
    match (self.stage, self.burst.samples) {
      (BurstStage::Drawing { sample, .. }, Some(samples)) => Some((sample, samples)),
      _ => None,
    }
  }

  /// The scene's time its frame this turn is drawn at, since its level began opening: a step after the burst's start a
  /// frame, the same over a frame's samples and while it waits, as it moves on only with a frame drawn. None once it
  /// reads its last frames back, the frames' own clock taking over.
  pub fn get_time(&self) -> Option<Duration> {
    let BurstStage::Drawing { frame, .. } = self.stage else {
      return None;
    };

    Some(Duration::from_secs_f64(
      f64::from(self.burst.at) + f64::from(frame) * f64::from(self.burst.step),
    ))
  }

  /// Which of its frames read back this turn's is, with its sample and the scene's seconds, none for a lead-in's.
  pub fn get_kept_frame(&self) -> Option<(u32, Option<u32>, f32)> {
    let BurstStage::Drawing { frame, sample } = self.stage else {
      return None;
    };
    let index: u32 = frame.checked_sub(self.burst.lead_in)?;

    Some((
      index,
      self.burst.samples.map(|_| sample),
      self.get_time()?.as_secs_f32(),
    ))
  }

  /// Ends the frame drawn this turn, with `kept` the copy of it read back, and moves on to its next sample, its next
  /// frame or its last frames' return. A frame read back whose copy was not recorded stops it, as the frames it reads
  /// back would no longer follow one another.
  pub fn advance(&mut self, kept: Option<BurstShot>) {
    let BurstStage::Drawing { frame, sample } = self.stage else {
      return;
    };

    if self.get_kept_frame().is_some() {
      match kept.filter(BurstShot::is_recorded) {
        Some(shot) => {
          shot.request();
          self.in_flight.push(shot);
        }
        None => {
          return self.stop(XrfError::new_unexpected_error(format!(
            "Burst frame {} was drawn but not copied out",
            frame - self.burst.lead_in
          )));
        }
      }
    }

    let samples: u32 = match self.burst.samples {
      Some(samples) if frame >= self.burst.lead_in => samples,
      _ => 1,
    };

    self.stage = if sample + 1 < samples {
      BurstStage::Drawing {
        frame,
        sample: sample + 1,
      }
    } else if frame + 1 < self.burst.lead_in + self.burst.frames {
      BurstStage::Drawing {
        frame: frame + 1,
        sample: 0,
      }
    } else {
      BurstStage::Reading
    };
  }

  /// Tells the caller of the frames come back since, and of its end once every frame is back.
  pub fn poll(&mut self) {
    for shot in std::mem::take(&mut self.in_flight) {
      match shot.take() {
        Some(Ok(event)) => self.send(event),
        Some(Err(error)) => return self.stop(error),
        None => self.in_flight.push(shot),
      }
    }

    if self.stage == BurstStage::Reading && self.in_flight.is_empty() && !self.is_ended {
      self.is_ended = true;
      self.send(BurstEvent::Finished);
    }
  }

  /// Stops it, telling the caller why; nothing more is drawn or read back.
  pub fn stop(&mut self, error: XrfError) {
    if !self.is_ended {
      self.is_ended = true;
      self.in_flight.clear();
      let _ = self.events.send(BurstEvent::Stopped { error });
    }
  }

  /// Whether it finished or stopped, so its viewport draws on its own clock again.
  pub fn is_ended(&self) -> bool {
    self.is_ended
  }

  /// Tells the caller; a caller gone ends it, as nothing would hear what it draws.
  fn send(&mut self, event: BurstEvent) {
    if self.events.send(event).is_err() {
      self.is_ended = true;
      self.in_flight.clear();
    }
  }
}
