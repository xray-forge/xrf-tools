use xrf_renderer::{RenderModelSkeleton, StaticModel, StaticModelPlace};

/// One model a loader thread packed for the scene: the visual it was read from, every object standing as it, and the
/// skeleton it is posed by where the renderer poses it.
pub struct SpawnLoad {
  pub model: StaticModel,
  pub name: String,
  pub places: Vec<StaticModelPlace>,
  pub skeleton: Option<RenderModelSkeleton>,
}
