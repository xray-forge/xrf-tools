use std::time::Instant;

use crate::host::world_fed_view::WorldFedView;

/// Where a host that runs its own state, as the game's simulation does, gives a viewport its camera and its clock each
/// frame, in place of the world's own controllers. It is read on the render thread once a viewport's frame, so it must
/// answer without waiting on the host's work.
pub trait WorldViewFeed: Send + Sync + 'static {
  /// The view to draw at `now`, the render frame's instant.
  fn read(&self, now: Instant) -> WorldFedView;
}
