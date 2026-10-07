use serde::{Deserialize, Serialize};
use xrf_renderer::RenderViewportEvent;
use xrf_world::WorldViewportEvent;

/// What a native viewport tells its page, from the renderer drawing it or the world it shows; each tells its own kinds.
#[cfg_attr(feature = "typescript-bindings", derive(specta::Type))]
#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
#[serde(untagged)]
pub enum ViewportEvent {
  Render(RenderViewportEvent),
  World(WorldViewportEvent),
}
