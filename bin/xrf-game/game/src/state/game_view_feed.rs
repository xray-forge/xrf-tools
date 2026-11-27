use std::sync::Arc;
use std::time::{Duration, Instant};

use xrf_game_loop::{LoopFeed, LoopRead};
use xrf_game_movement::MovementCamera;
use xrf_math::glam::Vec3;
use xrf_world::{WorldFedCamera, WorldFedClock, WorldFedView, WorldViewFeed};

use crate::input::game_look::GameLook;
use crate::state::game_frame::GameFrame;

/// The game loop's frames as the world draws them, read at each render frame's start. The player's own view does not
/// wait a tick: it faces where presentation's look faces now, and stands where the latest tick left it, carried forward
/// to the frame's instant by how fast it moved. The game's day, which the viewport's weather follows, and what else the
/// frame draws blend one tick behind.
pub struct GameViewFeed {
  feed: Arc<LoopFeed<GameFrame>>,
  look: Arc<GameLook>,
  /// The lens the camera is drawn through; its pose is replaced each frame.
  lens: WorldFedCamera,
}

impl GameViewFeed {
  /// The furthest the view is carried past its tick: a stalled loop holds the view there rather than drifting on.
  const MOST_AHEAD: Duration = Duration::from_millis(50);

  pub fn new(feed: Arc<LoopFeed<GameFrame>>, look: Arc<GameLook>, lens: WorldFedCamera) -> Self {
    Self { feed, look, lens }
  }

  /// The player's camera at `now`: the latest tick's position carried forward, facing as the look faces.
  pub fn get_camera(&self, read: &LoopRead<GameFrame>, now: Instant) -> MovementCamera {
    let ahead: Duration = now.saturating_duration_since(read.latest_at).min(Self::MOST_AHEAD);
    let position: Vec3 = read.latest.state.camera.position + read.latest.state.velocity * ahead.as_secs_f32();

    MovementCamera::facing(position, self.look.get())
  }
}

impl WorldViewFeed for GameViewFeed {
  /// Before the loop's first frame, the lens camera as given and the viewport's own clock.
  fn read(&self, now: Instant) -> WorldFedView {
    let Some(read): Option<LoopRead<GameFrame>> = self.feed.read(now) else {
      return WorldFedView {
        camera: self.lens,
        clock: None,
      };
    };
    let camera: MovementCamera = self.get_camera(&read, now);

    WorldFedView {
      camera: WorldFedCamera {
        position: camera.position,
        forward: camera.get_forward(),
        up: camera.get_up(),
        ..self.lens
      },
      clock: Some(WorldFedClock {
        time: read.previous.state.blend_time_of_day(&read.latest.state, read.alpha),
        rate: read.latest.state.time_factor,
      }),
    }
  }
}
