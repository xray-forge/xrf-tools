use crate::scene::level::light_shadow_face::LightShadowFace;

/// A light's faces at one size: a spot's one, a point's six, with the planes they were drawn between.
#[derive(Clone, Debug)]
pub struct LightShadowSet {
  /// The square side every face was asked at.
  pub size: u32,
  pub near: f32,
  pub far: f32,
  pub faces: Vec<LightShadowFace>,
}

impl LightShadowSet {
  /// Whether every face is drawn, which a light waits for before it lights with them.
  pub fn is_drawn(&self) -> bool {
    self.faces.iter().all(|face| face.drawn.is_some())
  }
}
