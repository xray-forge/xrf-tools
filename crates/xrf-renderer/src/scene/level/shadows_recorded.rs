use xrf_renderer_core::{GraphTexture, UniformBinding};

use crate::pass::wet_uniform::WetUniform;
use crate::scene::level::cover_textures::CoverTextures;

/// What a view's shadows phase hands the lighting: the rain's cover where it rains or the surfaces are wet, the wet
/// surfaces' uniform, and the level's surface as the wetting reads it (its lowest heights, its puddles and its water),
/// dry stand-ins where none is drawn.
#[derive(Clone, Copy)]
pub struct ShadowsRecorded {
  pub covers: Option<CoverTextures>,
  pub wet: Option<UniformBinding<WetUniform>>,
  pub surface: [GraphTexture; 3],
}
