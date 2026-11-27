use std::sync::Arc;
use std::time::Instant;

use xrf_game_loop::{LoopFeed, LoopFrame};
use xrf_world::{WorldFedCamera, WorldFedClock, WorldFedView, WorldViewFeed};

use crate::state::game_frame::GameFrame;

/// The game loop's frames as the world draws them: the camera blended at the render frame's instant, and the game's
/// day, which the viewport's weather follows.
pub struct GameViewFeed {
  feed: Arc<LoopFeed<GameFrame>>,
  /// The lens the camera is drawn through; its pose is replaced each frame.
  lens: WorldFedCamera,
}

impl GameViewFeed {
  pub fn new(feed: Arc<LoopFeed<GameFrame>>, lens: WorldFedCamera) -> Self {
    Self { feed, lens }
  }
}

impl WorldViewFeed for GameViewFeed {
  /// Before the loop's first frame, the lens camera as given and the viewport's own clock.
  fn read(&self, now: Instant) -> WorldFedView {
    let Some(frame): Option<LoopFrame<GameFrame>> = self.feed.read(now) else {
      return WorldFedView {
        camera: self.lens,
        clock: None,
      };
    };
    let camera = frame.state.camera;

    WorldFedView {
      camera: WorldFedCamera {
        position: camera.position,
        forward: camera.get_forward(),
        up: camera.get_up(),
        ..self.lens
      },
      clock: Some(WorldFedClock {
        time: frame.state.time_of_day,
        rate: frame.state.time_factor,
      }),
    }
  }
}
