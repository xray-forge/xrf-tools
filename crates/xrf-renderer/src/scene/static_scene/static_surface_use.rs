use crate::scene::static_scene::static_class::StaticClass;

/// What a resolved surface is drawn as: its row among the scene's surfaces, its class, and the surface passes it takes
/// part in, a bit each.
#[derive(Clone, Copy, Debug)]
pub struct StaticSurfaceUse {
  pub surface: u32,
  pub class: StaticClass,
  pub passes: u32,
}
