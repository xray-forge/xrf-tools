use std::sync::Arc;

use crate::visuals::SpawnVisual;

/// A spawned visual read for the renderer, by the name the spawn asked for it under.
pub(crate) struct NamedSpawnVisual<'a> {
  pub name: &'a str,
  pub visual: Arc<SpawnVisual>,
}
