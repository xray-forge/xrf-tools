use xrf_renderer_core::{FrameGraph, RasterContext, RasterPassBuilder};

use crate::feature::lighting::shadows::cascade_target::CascadeTarget;
use crate::feature::scene::drawable::drawable_kind::DrawableKind;
use crate::feature::scene::grass::cards::grass_shadow_casters::GrassShadowCasters;
use crate::pass::view::view_binding::ViewBinding;

/// A drawable kind beside the static scene as a frame's record reaches it, which the consumers drawing every kind alike
/// dispatch through, a match a kind.
#[derive(Clone, Copy)]
pub(crate) enum DrawableKindRef<'a> {
  /// The grass's cards, where they cast shadows this frame.
  Grass(GrassShadowCasters<'a>),
}

impl<'a> DrawableKindRef<'a> {
  pub(crate) fn get_kind(&self) -> DrawableKind {
    match self {
      Self::Grass(_) => DrawableKind::Grass,
    }
  }

  /// Declares its draw of what moves every frame into a cascade of the sun's maps, over what that holds.
  pub(crate) fn add_sun_pass(&self, graph: &mut FrameGraph<'a>, target: CascadeTarget<'a>) {
    match self {
      Self::Grass(casters) => casters.add_pass(graph, target),
    }
  }

  /// Declares on `builder` what its shadow casters' draw reads, for a pass drawing them into shadow maps.
  pub(crate) fn declare<'g>(&self, builder: RasterPassBuilder<'g, 'a>) -> RasterPassBuilder<'g, 'a> {
    match self {
      Self::Grass(casters) => casters.declare(builder),
    }
  }

  /// Draws its shadow casters into the map `context`'s pass draws into, seen by `view`, within the region already set.
  pub(crate) fn record_shadow(&self, context: &mut RasterContext<'_>, view: &ViewBinding) {
    match self {
      Self::Grass(casters) => casters.record(context, view),
    }
  }
}
