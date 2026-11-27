use xrf_world::{WorldEventSink, WorldViewportEvent};

/// What the world publishes of the game's viewport, its camera's pose and its weather: nothing reads them yet, since
/// the game's state is the loop's and the developer layer comes later.
pub(crate) struct GameWorldSink;

impl WorldEventSink for GameWorldSink {
  fn send(&self, _: WorldViewportEvent) {}
}
