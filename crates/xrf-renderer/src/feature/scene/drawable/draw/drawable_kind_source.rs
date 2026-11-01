use xrf_renderer_core::GraphDeclaration;

use crate::feature::scene::drawable::draw::drawable_kind_readied::DrawableKindReadied;
use crate::feature::scene::drawable::drawable_kind_traced::DrawableKindTraced;
use crate::feature::scene::drawable::drawable_trace_instance::DrawableTraceInstance;
use crate::feature::scene::grass::cards::grass_card_handles::GrassCardHandles;
use crate::feature::scene::grass::grass_view::GrassView;
use crate::feature::scene::grass::level::grass_source::GrassSource;
use crate::feature::scene::grass::trace::grass_trace_source::GrassTraceSource;
use crate::pass::level_passes::LevelPasses;
use crate::scene::level::level_scene::LevelScene;
use crate::scene::level::view::record::view_record::ViewRecord;

/// A drawable kind beside the static scene as a view finds it every frame, before the frame readies it, a variant a
/// kind: what the scene group draws each kind into the G-buffer through, and what the view's trace takes of it.
#[derive(Clone, Copy)]
pub enum DrawableKindSource<'a> {
  /// The view's grass over the level's.
  Grass {
    view: &'a GrassView,
    source: GrassSource<'a>,
  },
}

impl<'a> DrawableKindSource<'a> {
  /// Every drawable kind beside the static scene a view draws and traces, as it finds them every frame from each kind's
  /// view of it and the `scene`'s level.
  pub fn list(grass: &'a GrassView, scene: &'a LevelScene, passes: LevelPasses<'a>) -> [Self; 1] {
    [Self::Grass {
      view: grass,
      source: GrassSource {
        pass: &passes.scene.grass,
        level: scene.grass.get_level(),
      },
    }]
  }

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

  /// The kind's instance in the view's own top level, where the view traces it this frame: the grass's cards.
  pub fn get_trace_instance(&self) -> Option<DrawableTraceInstance<'a>> {
    match *self {
      Self::Grass { view, .. } => view.get_trace_instance(),
    }
  }

  /// Records the kind's part of the view's scene trace, which the trace binds whether or not the kind readied a frame:
  /// from what the frame `readied` of it, `empty` bound in place of what it holds none of.
  pub fn add_trace(
    &self,
    declaration: GraphDeclaration<'_, 'a>,
    record: ViewRecord<'a>,
    readied: &[DrawableKindReadied<'a>],
    empty: &'a wgpu::Buffer,
  ) -> DrawableKindTraced {
    match *self {
      Self::Grass { view, source } => {
        let cards: Option<GrassCardHandles> = readied.iter().find_map(|kind| match kind {
          DrawableKindReadied::Grass { cards, .. } => *cards,
        });

        DrawableKindTraced::Grass(view.add_trace(
          declaration,
          record,
          GrassTraceSource {
            level: source.level,
            cards,
            empty,
          },
        ))
      }
    }
  }
}
