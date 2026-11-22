use std::sync::Arc;
use std::time::Duration;

use xrf_math::Vector3d;
use xrf_math::glam::Vec3;
use xrf_renderer::{LoaderReceiver, RenderWorkers, take_answer};
use xrf_visual::{HemiEstimate, HemiEstimator};

use crate::host::world_level_source::WorldLevelSource;
use crate::level::loading::loader_once::LoaderOnce;

/// `get_luminocity_hemi() < 0.05`: the hemi under which the actor stands indoors (`CGamePersistent::WeathersUpdate`).
const INDOOR_HEMI: f32 = 0.05;

/// How long between estimates; the engine casts five of its sky's rays a frame (`ps_r2_dhemi_count`).
const ESTIMATE_INTERVAL: Duration = Duration::from_millis(200);

/// `ps_r2_lt_smooth`: how much of the way to the estimate the smoothed hemi moves a second.
const SMOOTHING: f32 = 1.0;

/// How lit the camera stands, as `CROS_impl` keeps the actor's hemi: estimated on the workers every so often where the
/// camera stands, which stands in for the actor's sphere, and smoothed towards each estimate (`update_smooth`).
///
/// Its estimator comes with the level ([`WorldLevelSource::read_hemi`]), read on the workers, apart from the particles,
/// so a level whose particles do not read still knows where the camera stands indoors.
pub(crate) struct CameraHemi {
  /// The estimator on its way from the level, none once taken.
  reading: LoaderOnce<Arc<HemiEstimator>>,
  estimator: Option<Arc<HemiEstimator>>,
  /// The estimate asked for and not taken yet.
  pending: Option<LoaderReceiver<HemiEstimate>>,
  /// The last estimate, `hemi_value` and `hemi_cube`.
  value: Option<HemiEstimate>,
  /// `hemi_smooth`, one half until the first estimate replaces it.
  smooth: f32,
  /// `hemi_cube_smooth`, its faces as `HemiCube` orders them.
  cube_smooth: [f32; 6],
  /// When the next estimate is due, and when the hemi was last smoothed, on the engine's clock.
  due: Duration,
  smoothed_at: Option<Duration>,
  workers: RenderWorkers,
}

impl CameraHemi {
  /// Starts reading the level's estimator from `source` on the workers; until it comes, nothing is estimated.
  pub(crate) fn read(source: &Arc<dyn WorldLevelSource>, workers: &RenderWorkers) -> Self {
    let reader: Arc<dyn WorldLevelSource> = Arc::clone(source);

    Self {
      reading: LoaderOnce::spawn(workers, "camera hemi estimator", move || reader.read_hemi()),
      estimator: None,
      pending: None,
      value: None,
      smooth: 0.5,
      cube_smooth: [0.5; 6],
      due: Duration::ZERO,
      smoothed_at: None,
      workers: workers.clone(),
    }
  }

  /// Takes a finished estimate, asks for the next one once due, and smooths the hemi on to `now`, the engine's clock.
  /// `eye` is where the camera stands, in engine space.
  pub(crate) fn advance(&mut self, eye: Vec3, now: Duration) {
    if let Some(estimator) = self.reading.take() {
      self.estimator = Some(estimator);
    }

    // A worker that never answered leaves none pending, so the next is asked for.
    if let Some(value) = take_answer(&mut self.pending, "camera hemi") {
      // The first estimate is taken as it is, as the first update takes it.
      if self.value.is_none() {
        self.smooth = value.sky;
        self.cube_smooth = value.cube.faces;
      }

      self.value = Some(value);
    }

    if self.pending.is_none()
      && now >= self.due
      && let Some(estimator) = &self.estimator
    {
      let (sender, receiver) = LoaderReceiver::channel();
      let estimator: Arc<HemiEstimator> = Arc::clone(estimator);

      self.workers.spawn(move || {
        let _ = sender.send(estimator.estimate(&Vector3d::from(eye), 0.0));
      });
      self.pending = Some(receiver);
      self.due = now + ESTIMATE_INTERVAL;
    }

    if let (Some(value), Some(at)) = (self.value, self.smoothed_at) {
      let share: f32 = (now.saturating_sub(at).as_secs_f32() * SMOOTHING).clamp(0.0, 1.0);

      self.smooth += (value.sky - self.smooth) * share;

      for (smooth, face) in self.cube_smooth.iter_mut().zip(value.cube.faces) {
        *smooth += (face - *smooth) * share;
      }
    }

    self.smoothed_at = Some(now);
  }

  /// The smoothed hemi, none where the level has no estimator.
  pub(crate) fn get_smooth(&self) -> Option<f32> {
    self.estimator.is_some().then_some(self.smooth)
  }

  /// The brightest face of the smoothed cube but the one facing down, which the rain's loop is as loud as
  /// (`CEffect_Rain::OnFrame`); none where the level has no estimator.
  pub(crate) fn get_open_sky(&self) -> Option<f32> {
    let [px, py, pz, nx, _, nz] = self.cube_smooth;

    self.estimator.is_some().then(|| px.max(py).max(pz).max(nx).max(nz))
  }

  /// Whether the camera stands indoors, which a level without an estimator never does. The engine estimates the
  /// actor's hemi in its first frame; until the estimator and its first estimate are back the camera counts as indoors,
  /// so nothing starts where it is not yet known.
  pub(crate) fn is_indoors(&self) -> bool {
    self.reading.is_pending() || (self.estimator.is_some() && (self.value.is_none() || self.smooth < INDOOR_HEMI))
  }
}
