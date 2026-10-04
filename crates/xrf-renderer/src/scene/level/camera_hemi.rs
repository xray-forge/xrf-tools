use std::sync::Arc;
use std::sync::mpsc::{Receiver, channel};

use glam::Vec3;
use xrf_math::Vector3d;
use xrf_visual::HemiEstimator;

use crate::thread::render_workers::RenderWorkers;

/// `get_luminocity_hemi() < 0.05`: the hemi under which the actor stands indoors (`CGamePersistent::WeathersUpdate`).
const INDOOR_HEMI: f32 = 0.05;

/// Milliseconds between estimates; the engine casts five of its sky's rays a frame (`ps_r2_dhemi_count`).
const ESTIMATE_INTERVAL: u64 = 200;

/// `ps_r2_lt_smooth`: how much of the way to the estimate the smoothed hemi moves a second.
const SMOOTHING: f32 = 1.0;

/// How lit the camera stands, as `CROS_impl` keeps the actor's hemi: estimated on the workers every so often where the
/// camera stands, which stands in for the actor's sphere, and smoothed towards each estimate (`update_smooth`).
pub struct CameraHemi {
  estimator: Option<Arc<HemiEstimator>>,
  pending: Option<Receiver<f32>>,
  /// The last estimate, `hemi_value`.
  value: Option<f32>,
  /// `hemi_smooth`, one half until the first estimate replaces it.
  smooth: f32,
  /// When the next estimate is due, and when the hemi was last smoothed, in milliseconds.
  due: u64,
  smoothed_at: Option<u64>,
  workers: RenderWorkers,
}

impl CameraHemi {
  pub fn new(estimator: Option<Arc<HemiEstimator>>, workers: &RenderWorkers) -> Self {
    Self {
      estimator,
      pending: None,
      value: None,
      smooth: 0.5,
      due: 0,
      smoothed_at: None,
      workers: workers.clone(),
    }
  }

  /// Takes a finished estimate, asks for the next one once due, and smooths the hemi on to `now`. `eye` is where the
  /// camera stands, in engine space.
  pub fn advance(&mut self, eye: Vec3, now: u64) {
    if let Some(value) = self.pending.as_ref().and_then(|pending| pending.try_recv().ok()) {
      // The first estimate is taken as it is, as the first update takes it.
      if self.value.is_none() {
        self.smooth = value;
      }

      self.value = Some(value);
      self.pending = None;
    }

    if self.pending.is_none()
      && now >= self.due
      && let Some(estimator) = &self.estimator
    {
      let (sender, receiver) = channel();
      let estimator: Arc<HemiEstimator> = Arc::clone(estimator);

      self.workers.spawn(move || {
        let _ = sender.send(estimator.estimate(&Vector3d::new(eye.x, eye.y, eye.z), 0.0).sky);
      });
      self.pending = Some(receiver);
      self.due = now + ESTIMATE_INTERVAL;
    }

    if let (Some(value), Some(at)) = (self.value, self.smoothed_at) {
      let share: f32 = ((now.saturating_sub(at)) as f32 / 1000.0 * SMOOTHING).clamp(0.0, 1.0);

      self.smooth += (value - self.smooth) * share;
    }

    self.smoothed_at = Some(now);
  }

  /// Whether the camera stands indoors, which a level without an estimator never does.
  pub fn is_indoors(&self) -> bool {
    self.smooth < INDOOR_HEMI
  }
}
