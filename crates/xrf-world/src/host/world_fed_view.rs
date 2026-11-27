use crate::host::world_fed_camera::WorldFedCamera;
use crate::host::world_fed_clock::WorldFedClock;

/// What a [`crate::WorldViewFeed`] answers for a frame: the camera, and the game's clock where the host runs the day,
/// which the viewport's weather then follows instead of its own.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct WorldFedView {
  pub camera: WorldFedCamera,
  pub clock: Option<WorldFedClock>,
}
