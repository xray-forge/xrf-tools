use xrf_renderer_core::{FrameGraph, GraphDeclaration, GraphTexture, UniformBinding};

use crate::contract::feature::tracing::render_reflection_settings::RenderReflectionSettings;
use crate::feature::scene::drawable::casters::drawable_kind_ref::DrawableKindRef;
use crate::feature::scene::grass::build::grass_planting::GrassPlanting;
use crate::feature::scene::grass::cards::grass_card_handles::GrassCardHandles;
use crate::feature::scene::grass::draw::grass_occlusion_parameters::GrassOcclusionParameters;
use crate::feature::scene::grass::grass_view::GrassView;
use crate::feature::scene::grass::level::grass_source::GrassSource;
use crate::feature::scene::statics::cull::static_occlusion_uniform::StaticOcclusionUniform;
use crate::frame::targets::view_target_handles::ViewTargetHandles;
use crate::pass::view::view_binding::ViewBinding;

/// A drawable kind as the frame readied it ahead of the static scene's depth, a variant a kind: what the scene group's
/// later phases cull, stand and draw, and the shadows draw its casters from.
#[derive(Clone, Copy)]
pub enum DrawableKindReadied<'a> {
  /// The grass planted this frame, and its cards once they stand.
  Grass {
    view: &'a GrassView,
    source: GrassSource<'a>,
    planting: GrassPlanting,
    cards: Option<GrassCardHandles>,
  },
}

impl<'a> DrawableKindReadied<'a> {
  /// Culls what the kind readied against the static scene's depth this frame: its `pyramid`, reduced through the view
  /// `occlusion` holds.
  pub fn add_cull(
    &self,
    graph: &mut FrameGraph<'a>,
    pyramid: GraphTexture,
    occlusion: UniformBinding<StaticOcclusionUniform>,
  ) {
    match *self {
      Self::Grass {
        view, source, planting, ..
      } => view.add_cull(graph, source, planting, GrassOcclusionParameters { pyramid, occlusion }),
    }
  }

  /// Stands the casters its shadows and its trace draw, where it casts any: the grass's cards.
  pub fn add_casters(&mut self, declaration: GraphDeclaration<'_, 'a>) {
    match self {
      Self::Grass {
        view,
        source,
        planting,
        cards,
      } => *cards = view.add_cards(declaration, *source, *planting),
    }
  }

  /// Whether the reflections walk their rays over its depth, so it draws before they keep it.
  pub fn is_reflected(&self, settings: &RenderReflectionSettings) -> bool {
    match self {
      Self::Grass { .. } => settings.is_grass_reflected,
    }
  }

  /// Declares its draw into the G-buffer `targets` hold, seen by `view`.
  pub fn add_draw(
    &self,
    graph: &mut FrameGraph<'a>,
    targets: ViewTargetHandles,
    view: &'a ViewBinding,
    textures: &'a wgpu::BindGroup,
  ) {
    match *self {
      Self::Grass {
        view: grass,
        source,
        planting,
        ..
      } => grass.add_draw(graph, source, targets, planting, view, textures),
    }
  }

  /// What it casts into the sun's cascades and the lamps' faces, where it casts this frame.
  pub fn get_shadow_casters(
    &self,
    declaration: GraphDeclaration<'_, 'a>,
    textures: &'a wgpu::BindGroup,
  ) -> Option<DrawableKindRef<'a>> {
    match *self {
      Self::Grass {
        view, source, cards, ..
      } => view
        .get_shadow_casters(declaration, source, textures, cards)
        .map(DrawableKindRef::Grass),
    }
  }
}
