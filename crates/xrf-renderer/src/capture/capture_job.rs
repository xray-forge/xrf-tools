use std::sync::mpsc::Sender;
use std::time::Duration;

use xrf_error::XrfError;

use crate::capture::capture_event::CaptureEvent;
use crate::capture::capture_plan::CapturePlan;
use crate::capture::capture_shot::CaptureShot;
use crate::frame::frame_capture::FrameCapture;
use crate::pass::view_binding::ViewBinding;
use crate::scene::level::scene_view::SceneView;

/// Where a capture stands: metering its exposure, settling a shot, or waiting for the last shots back.
#[derive(Clone, Copy, Debug, PartialEq)]
enum CaptureStage {
  Metering { frames: u32 },
  Shot { index: usize, frames: u32 },
  Reading,
}

/// A level map or cubemap captured in a viewport in place of its own frames, through a view of the scene of its own.
pub struct CaptureJob {
  plan: CapturePlan,
  stage: CaptureStage,
  /// Its view of the scene and its camera on the GPU, made once a GPU is there.
  view: Option<(SceneView, ViewBinding)>,
  /// The shots copied out and on their way back, by their place in the plan.
  in_flight: Vec<(usize, FrameCapture)>,
  events: Sender<CaptureEvent>,
  is_ended: bool,
}

impl CaptureJob {
  /// The time its view's frames are drawn apart: a frame at sixty a second, however long each took, so its metering
  /// settles alike however fast it draws.
  const FRAME_STEP: Duration = Duration::from_micros(16_667);

  pub fn new(plan: CapturePlan, events: Sender<CaptureEvent>) -> Self {
    let mut job: Self = Self {
      plan,
      stage: CaptureStage::Metering { frames: 0 },
      view: None,
      in_flight: Vec::new(),
      events,
      is_ended: false,
    };

    job.send(CaptureEvent::Progress {
      drawn: 0,
      total: job.plan.shots.len() as u32,
    });

    job
  }

  /// The shot this frame draws: a metering view, a shot settling, or none while the last come back.
  pub fn get_drawn_shot(&self) -> Option<CaptureShot> {
    match self.stage {
      CaptureStage::Metering { frames } => self.plan.get_meter(frames),
      CaptureStage::Shot { index, .. } => self.plan.shots.get(index).copied(),
      CaptureStage::Reading => None,
    }
  }

  /// Whether this frame meters the exposure rather than drawing a shot.
  pub fn is_metering(&self) -> bool {
    matches!(self.stage, CaptureStage::Metering { .. })
  }

  /// Whether this frame's shot is the one copied out: the last of its settling frames.
  pub fn is_kept_frame(&self) -> bool {
    match self.stage {
      CaptureStage::Shot { frames, .. } => frames + 1 >= self.plan.settle_frames.max(1),
      _ => false,
    }
  }

  /// Its view and its camera's binding, made with `make` the first time, its clock stepped by its frames.
  pub fn get_view_mut(&mut self, make: impl FnOnce() -> (SceneView, ViewBinding)) -> (&mut SceneView, &ViewBinding) {
    let (view, binding): &mut (SceneView, ViewBinding) = self.view.get_or_insert_with(|| {
      let (mut view, binding): (SceneView, ViewBinding) = make();

      view.step_clock(Some(Self::FRAME_STEP));

      (view, binding)
    });

    (view, binding)
  }

  /// Its view and its camera's binding, none before they are made.
  pub fn get_view(&self) -> Option<(&SceneView, &ViewBinding)> {
    self.view.as_ref().map(|(view, binding)| (view, binding))
  }

  /// Its view, none before it is made.
  pub fn get_scene_view_mut(&mut self) -> Option<&mut SceneView> {
    self.view.as_mut().map(|(view, _)| view)
  }

  /// Ends the frame its view drew and moves on, with `kept` the shot copied out of it; a shot whose copy was not
  /// recorded is drawn again.
  pub fn advance(&mut self, kept: Option<FrameCapture>) {
    if let Some((view, _)) = self.view.as_mut() {
      view.end_frame();
    }

    self.stage = match self.stage {
      CaptureStage::Metering { frames } if frames + 1 >= self.plan.meter_frames => {
        if let Some((view, _)) = self.view.as_mut() {
          view.freeze_exposure(true);
        }

        CaptureStage::Shot { index: 0, frames: 0 }
      }
      CaptureStage::Metering { frames } => CaptureStage::Metering { frames: frames + 1 },
      CaptureStage::Shot { index, frames } => match kept.filter(FrameCapture::is_recorded) {
        Some(capture) => {
          capture.request();
          self.in_flight.push((index, capture));
          self.send(CaptureEvent::Progress {
            drawn: index as u32 + 1,
            total: self.plan.shots.len() as u32,
          });

          if index + 1 < self.plan.shots.len() {
            CaptureStage::Shot {
              index: index + 1,
              frames: 0,
            }
          } else {
            CaptureStage::Reading
          }
        }
        None => CaptureStage::Shot {
          index,
          frames: frames + 1,
        },
      },
      CaptureStage::Reading => CaptureStage::Reading,
    };
  }

  /// Tells the caller of the shots come back since, and of its end once every shot is back.
  pub fn poll(&mut self) {
    for (index, capture) in std::mem::take(&mut self.in_flight) {
      match capture.take() {
        Some(Ok(capture)) => match self.plan.shots.get(index).copied() {
          Some(shot) => self.send(CaptureEvent::Shot { shot, capture }),
          None => {
            return self.stop(XrfError::new_unexpected_error(format!(
              "A capture read back shot {index} it never planned"
            )));
          }
        },
        Some(Err(error)) => return self.stop(error),
        None => self.in_flight.push((index, capture)),
      }
    }

    if self.stage == CaptureStage::Reading && self.in_flight.is_empty() && !self.is_ended {
      self.is_ended = true;
      self.send(CaptureEvent::Finished);
    }
  }

  /// Stops it, telling the caller why; nothing more is drawn or read back.
  pub fn stop(&mut self, error: XrfError) {
    if !self.is_ended {
      self.is_ended = true;
      self.in_flight.clear();
      let _ = self.events.send(CaptureEvent::Stopped { error });
    }
  }

  /// Whether it finished or stopped, so its viewport draws its own frames again.
  pub fn is_ended(&self) -> bool {
    self.is_ended
  }

  /// Tells the caller; a caller gone ends it, as nothing would hear what it draws.
  fn send(&mut self, event: CaptureEvent) {
    if self.events.send(event).is_err() {
      self.is_ended = true;
      self.in_flight.clear();
    }
  }
}
