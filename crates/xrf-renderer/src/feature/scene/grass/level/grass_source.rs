use crate::feature::scene::grass::grass_pass::GrassPass;
use crate::feature::scene::grass::level::grass_level::GrassLevel;

/// What a view's grass is planted and drawn from: the grass pass, and the level's grass, none before it streams in.
#[derive(Clone, Copy)]
pub struct GrassSource<'a> {
  pub pass: &'a GrassPass,
  pub level: Option<&'a GrassLevel>,
}
