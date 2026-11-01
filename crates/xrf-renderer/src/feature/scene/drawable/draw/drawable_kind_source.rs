use xrf_renderer_core::GraphDeclaration;

use crate::feature::scene::drawable::draw::drawable_kind_readied::DrawableKindReadied;
use crate::feature::scene::grass::grass_view::GrassView;
use crate::feature::scene::grass::level::grass_source::GrassSource;

/// A drawable kind beside the static scene as a view's scene group finds it, before the frame readies it, a variant a
/// kind: what the group draws each kind into the G-buffer through.
#[derive(Clone, Copy)]
pub enum DrawableKindSource<'a> {
  /// The view's grass over the level's.
  Grass {
    view: &'a GrassView,
    source: GrassSource<'a>,
  },
}

impl<'a> DrawableKindSource<'a> {
  /// Readies the kind's frame ahead of the static scene's depth, as the grass plants its ring's stale cells; none where
  /// it draws nothing this frame.
  pub fn add_ready(self, declaration: GraphDeclaration<'_, 'a>) -> Option<DrawableKindReadied<'a>> {
    match self {
      Self::Grass { view, source } => {
        view
          .add_planting(declaration, source)
          .map(|planting| DrawableKindReadied::Grass {
            view,
            source,
            planting,
            cards: None,
          })
      }
    }
  }
}
