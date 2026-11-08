use crate::feature::scene::grass::cards::grass_card_handles::GrassCardHandles;
use crate::feature::scene::grass::level::grass_level::GrassLevel;

/// What the grass's part of a view's trace is recorded from: the level's grass, none where it holds none; the cards the
/// frame stood, none where it stood none; and a buffer of nothing bound in place of either.
#[derive(Clone, Copy)]
pub(crate) struct GrassTraceSource<'a> {
  pub level: Option<&'a GrassLevel>,
  pub cards: Option<GrassCardHandles>,
  pub empty: &'a wgpu::Buffer,
}
