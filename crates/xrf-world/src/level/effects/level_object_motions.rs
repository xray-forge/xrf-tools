use std::sync::Arc;
use std::time::Duration;

use glam::{Mat4, Vec3};
use xrf_anm::AnmFile;
use xrf_renderer::{RenderLevelSource, RenderWorkers};
use xrf_visual::VisualTransform;

use crate::level::loading::loader_slot::LoaderSlot;
use crate::level::loading::loader_slots::LoaderSlots;

/// The object motions a level's moving zones follow, each read once on a loader thread the first time something it
/// carries asks, and played looping from when the level began opening (`CObjectAnimator::Play(true)` at spawn), one
/// clock for every zone, so a zone's idle effect and its idle light move together.
pub struct LevelObjectMotions {
  source: Arc<dyn RenderLevelSource>,
  workers: RenderWorkers,
  motions: LoaderSlots<AnmFile>,
  /// The level's clock at this frame and at the one before.
  now: Duration,
  previous: Duration,
}

impl LevelObjectMotions {
  pub fn new(source: &Arc<dyn RenderLevelSource>, workers: &RenderWorkers) -> Self {
    Self {
      source: Arc::clone(source),
      workers: workers.clone(),
      motions: LoaderSlots::default(),
      now: Duration::ZERO,
      previous: Duration::ZERO,
    }
  }

  /// Reads from `source`, a next source of the same level, from now on: where `is_reread`, every motion read before is
  /// let go and read again from it the next time something asks, else each stays as it was read.
  pub fn follow(&mut self, source: &Arc<dyn RenderLevelSource>, is_reread: bool) {
    self.source = Arc::clone(source);

    if is_reread {
      self.motions = LoaderSlots::default();
    }
  }

  /// Moves on to `now`, the level's clock.
  pub fn prepare(&mut self, now: Duration) {
    self.previous = self.now;
    self.now = now;
  }

  /// Where a motion has its object this frame, in engine space, and how fast it moved since the frame before
  /// (`CCustomZone::OnMove`); none while the motion is read, or where it cannot be, which leaves the object standing.
  pub fn get_pose(&mut self, name: &str) -> Option<(Mat4, Vec3)> {
    let (now, previous) = (self.now, self.previous);
    let motion: &AnmFile = self.get(name)?;
    let (looped, looped_before): (f32, f32) = (motion.get_looped_time(now), motion.get_looped_time(previous));
    let (position, rotation) = motion.evaluate(looped);
    let transform: Mat4 =
      Mat4::from_cols_array(&VisualTransform::of_motion(&position, &rotation).mirrored().to_matrix());
    let elapsed: f32 = now.saturating_sub(previous).as_secs_f32();
    // Across the loop's seam the two times lie at its two ends: the motion is measured over this frame's stretch
    // ahead instead, rather than over the whole path back to its start.
    let (from, to) = if looped_before > looped {
      (position, motion.evaluate(looped + elapsed).0)
    } else {
      (motion.evaluate(looped_before).0, position)
    };
    let velocity: Vec3 = if elapsed > 0.0 {
      (Vec3::from(to.to_array()) - Vec3::from(from.to_array())) / elapsed
    } else {
      Vec3::ZERO
    };

    Some((transform, velocity))
  }

  /// The motion by its name once it is read, asking for it the first time.
  /// Whether the motion `name` is still on its way from its loader.
  #[cfg(test)]
  pub(crate) fn is_pending(&self, name: &str) -> bool {
    self.motions.is_pending(name)
  }

  fn get(&mut self, name: &str) -> Option<&AnmFile> {
    let (source, workers) = (&self.source, &self.workers);

    self.motions.get(name, || {
      let (source, name) = (Arc::clone(source), name.to_owned());

      LoaderSlot::spawn(workers, "object motion", move || {
        source
          .read_object_motion(&name)
          .inspect_err(|error| log::warn!("Object motion '{name}' cannot be played: {error}"))
          .ok()
      })
    })
  }
}
