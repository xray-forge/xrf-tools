use std::sync::Arc;

use crate::parts::LevelSpawnVisual;

/// A spawned visual read for the renderer, by the name the spawn asked for it under.
pub struct NamedSpawnVisual<'a> {
  pub name: &'a str,
  pub visual: Arc<LevelSpawnVisual>,
}
