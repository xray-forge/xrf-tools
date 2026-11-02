use std::sync::{Arc, Mutex};

use crate::host::render_bundle::RenderBundle;
use crate::host::render_cache_store::RenderCacheStore;
use crate::host::world::render_world::RenderWorld;

/// What the application hands the render thread beside its commands: the files it ships with, the world it draws, and
/// where it keeps the driver's compiled pipelines between runs, none where nothing keeps them.
pub struct RenderHosts {
  pub bundle: Arc<dyn RenderBundle>,
  pub world: Arc<Mutex<dyn RenderWorld>>,
  pub store: Option<Arc<dyn RenderCacheStore>>,
}
