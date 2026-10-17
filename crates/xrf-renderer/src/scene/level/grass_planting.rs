use xrf_renderer_core::GraphBuffer;

use crate::pass::grass_build_parameters::GrassBuildParameters;
use crate::pass::grass_draw_parameters::GrassDrawParameters;
use crate::pass::grass_level_parameters::GrassLevelParameters;
use crate::pass::grass_sort_parameters::GrassSortParameters;

/// The grass's planting as one frame's graph binds it: the level's and the view's groups the planting, cull and cards
/// read, the sort's dispatch, what the draw reads, and the draw arguments and counts the cull writes.
#[derive(Clone, Copy)]
pub struct GrassPlanting {
  pub level: GrassLevelParameters,
  pub build: GrassBuildParameters,
  pub sort: GrassSortParameters,
  pub draw: GrassDrawParameters,
  pub args: GraphBuffer,
  pub counts: GraphBuffer,
}
