use std::collections::HashMap;
use std::sync::Arc;
use std::sync::mpsc::TryRecvError;
use std::time::Instant;

use glam::{Mat4, Vec3};
use xrf_anm::AnmFile;
use xrf_renderer::{LoaderReceiver, RenderLevelSource, RenderWorkers, to_vec3};
use xrf_visual::VisualTransform;

/// One object motion asked for: on its way from a loader thread, read, or not to be had.
enum MotionState {
  Reading(LoaderReceiver<Option<AnmFile>>),
  Read(AnmFile),
  Missing,
}

/// The object motions a level's moving zones follow, each read once on a loader thread the first time something it
/// carries asks, and played looping from when the level began opening (`CObjectAnimator::Play(true)` at spawn), one
/// clock for every zone, so a zone's idle effect and its idle light move together.
pub struct LevelObjectMotions {
  source: Arc<dyn RenderLevelSource>,
  workers: RenderWorkers,
  motions: HashMap<String, MotionState>,
  started: Instant,
  /// Seconds since the level began opening, at this frame and at the one before.
  now: f32,
  previous: f32,
}

impl LevelObjectMotions {
  pub fn new(source: &Arc<dyn RenderLevelSource>, workers: &RenderWorkers) -> Self {
    Self {
      source: Arc::clone(source),
      workers: workers.clone(),
      motions: HashMap::new(),
      started: Instant::now(),
      now: 0.0,
      previous: 0.0,
    }
  }

  /// Moves the clock on to this frame.
  pub fn prepare(&mut self) {
    self.prepare_at(self.started.elapsed().as_secs_f32());
  }

  /// Where a motion has its object this frame, in engine space, and how fast it moved since the frame before
  /// (`CCustomZone::OnMove`); none while the motion is read, or where it cannot be, which leaves the object standing.
  pub fn get_pose(&mut self, name: &str) -> Option<(Mat4, Vec3)> {
    let (now, previous) = (self.now, self.previous);
    let motion: &AnmFile = self.get(name)?;
    let (position, rotation) = motion.evaluate(motion.get_looped_time(now));
    let (before, _) = motion.evaluate(motion.get_looped_time(previous));
    let transform: Mat4 =
      Mat4::from_cols_array(&VisualTransform::of_motion(&position, &rotation).mirrored().to_matrix());
    let elapsed: f32 = now - previous;
    let velocity: Vec3 = if elapsed > 0.0 {
      (to_vec3(&position) - to_vec3(&before)) / elapsed
    } else {
      Vec3::ZERO
    };

    Some((transform, velocity))
  }

  fn prepare_at(&mut self, now: f32) {
    self.previous = self.now;
    self.now = now;
  }

  /// The motion by its name once it is read, asking for it the first time.
  fn get(&mut self, name: &str) -> Option<&AnmFile> {
    let (source, workers) = (&self.source, &self.workers);
    let state: &mut MotionState = self.motions.entry(name.to_owned()).or_insert_with(|| {
      let (sender, receiver) = LoaderReceiver::channel();
      let (source, name) = (Arc::clone(source), name.to_owned());

      workers.spawn(move || {
        let motion: Option<AnmFile> = source
          .read_object_motion(&name)
          .inspect_err(|error| log::warn!("Object motion '{name}' cannot be played: {error}"))
          .ok();

        let _ = sender.send(motion);
      });

      MotionState::Reading(receiver)
    });

    if let MotionState::Reading(receiver) = state {
      *state = match receiver.try_recv() {
        Ok(Some(motion)) => MotionState::Read(motion),
        Ok(None) | Err(TryRecvError::Disconnected) => MotionState::Missing,
        Err(TryRecvError::Empty) => return None,
      };
    }

    match state {
      MotionState::Read(motion) => Some(motion),
      _ => None,
    }
  }
}

#[cfg(test)]
mod tests {
  use std::sync::Arc;
  use std::time::{Duration, Instant};

  use glam::{Mat4, Vec3};
  use xrf_animation_envelope::{AnimationEnvelope, AnimationKey};
  use xrf_anm::AnmFile;
  use xrf_environment::WeatherDescriptor;
  use xrf_error::{XrfError, XrfResult};
  use xrf_material::XraySurfaceDescriptor;
  use xrf_renderer::{
    RenderAssetSource, RenderLevelDetails, RenderLevelParticles, RenderLevelSource, RenderLevelSpawn,
    RenderLevelWeather, RenderSpawnModels,
  };
  use xrf_visual::{LightsDescription, SectorPackage};

  use crate::level::level_object_motions::{LevelObjectMotions, MotionState};
  use crate::tests::test_workers::create_workers;

  /// A level with nothing but one motion, `line.anm`: two seconds carrying its object 3 metres a second along x.
  struct MotionSource;

  impl RenderAssetSource for MotionSource {
    fn get_texture_scope(&self) -> String {
      String::from("test")
    }

    fn read_texture(&self, _: &str) -> XrfResult<Option<Vec<u8>>> {
      Ok(None)
    }
  }

  impl RenderLevelSource for MotionSource {
    fn get_sector_count(&self) -> u32 {
      0
    }

    fn pack_sector(&self, _: u32) -> XrfResult<SectorPackage> {
      Err(none())
    }

    fn get_surfaces(&self) -> &[XraySurfaceDescriptor] {
      &[]
    }

    fn read_lights(&self) -> XrfResult<LightsDescription> {
      Err(none())
    }

    fn read_weather(&self) -> XrfResult<RenderLevelWeather> {
      Err(none())
    }

    fn read_weather_cycle(&self, _: &str) -> XrfResult<Vec<WeatherDescriptor>> {
      Err(none())
    }

    fn read_spawn(&self) -> XrfResult<RenderLevelSpawn> {
      Err(none())
    }

    fn read_spawn_models(&self, _: &[String]) -> XrfResult<RenderSpawnModels> {
      Err(none())
    }

    fn read_details(&self) -> XrfResult<Option<RenderLevelDetails>> {
      Ok(None)
    }

    fn read_particles(&self) -> XrfResult<Option<RenderLevelParticles>> {
      Ok(None)
    }

    fn read_object_motion(&self, name: &str) -> XrfResult<AnmFile> {
      if name != "line.anm" {
        return Err(none());
      }

      let key = |time: f32, value: f32| AnimationKey {
        value,
        time,
        shape: 4,
        interpolation: None,
      };
      let still = || AnimationEnvelope {
        behavior: (1, 1),
        keys: Vec::new(),
      };

      Ok(AnmFile {
        name: String::new(),
        frame_start: 0,
        frame_end: 60,
        fps: 30.0,
        version: AnmFile::CURRENT_VERSION,
        channels: vec![
          AnimationEnvelope {
            behavior: (1, 1),
            keys: vec![
              key(0.0, 0.0),
              AnimationKey {
                shape: 3,
                ..key(2.0, 6.0)
              },
            ],
          },
          still(),
          still(),
          still(),
          still(),
          still(),
        ],
      })
    }
  }

  fn none() -> XrfError {
    XrfError::new_not_found_error("nothing")
  }

  /// Asks for a motion until its loader has answered, a few seconds at most.
  fn read(motions: &mut LevelObjectMotions, name: &str) -> Option<(Mat4, Vec3)> {
    let started: Instant = Instant::now();

    loop {
      if let Some(pose) = motions.get_pose(name) {
        return Some(pose);
      }

      if started.elapsed() > Duration::from_secs(5) {
        return None;
      }

      std::thread::sleep(Duration::from_millis(5));
    }
  }

  // At 1.5 s the object stands 4.5 m along x, and since the frame at 1 s it moved 1.5 m in half a second.
  #[test]
  fn moves_an_object_along_its_motion_with_its_velocity() {
    let source: Arc<dyn RenderLevelSource> = Arc::new(MotionSource);
    let mut motions: LevelObjectMotions = LevelObjectMotions::new(&source, &create_workers());

    motions.prepare_at(1.0);
    motions.prepare_at(1.5);

    let (transform, velocity) = read(&mut motions, "line.anm").expect("the motion to be read");

    assert!(transform.w_axis.truncate().abs_diff_eq(Vec3::new(4.5, 0.0, 0.0), 1e-4));
    assert!(velocity.abs_diff_eq(Vec3::new(3.0, 0.0, 0.0), 1e-3));
  }

  #[test]
  fn leaves_an_object_standing_without_its_motion() {
    let source: Arc<dyn RenderLevelSource> = Arc::new(MotionSource);
    let mut motions: LevelObjectMotions = LevelObjectMotions::new(&source, &create_workers());

    motions.prepare_at(1.0);

    let started: Instant = Instant::now();

    while motions.get_pose("gone.anm").is_none()
      && matches!(motions.motions.get("gone.anm"), Some(MotionState::Reading(_)))
      && started.elapsed() < Duration::from_secs(5)
    {
      std::thread::sleep(Duration::from_millis(5));
    }

    assert!(matches!(motions.motions.get("gone.anm"), Some(MotionState::Missing)));
    assert!(motions.get_pose("gone.anm").is_none());
  }
}
