use xrf_renderer_core::FrameGraph;

use crate::feature::lighting::shadows::cascade_target::CascadeTarget;
use crate::feature::scene::drawable::drawable_kind::DrawableKind;
use crate::feature::scene::grass::cards::grass_shadow_casters::GrassShadowCasters;

/// A drawable kind beside the static scene as a frame's record reaches it, which the consumers drawing every kind alike
/// dispatch through, a match a kind.
#[derive(Clone, Copy)]
pub enum DrawableKindRef<'a> {
  /// The grass's cards, where they cast the sun's shadow this frame.
  Grass(GrassShadowCasters<'a>),
}

impl<'a> DrawableKindRef<'a> {
  pub fn get_kind(&self) -> DrawableKind {
    match self {
      Self::Grass(_) => DrawableKind::Grass,
    }
  }

  /// Declares its draw of what moves every frame into a cascade of the sun's maps, over what that holds.
  pub fn add_sun_pass(&self, graph: &mut FrameGraph<'a>, target: CascadeTarget<'a>) {
    match self {
      Self::Grass(casters) => casters.add_pass(graph, target),
    }
  }
}
