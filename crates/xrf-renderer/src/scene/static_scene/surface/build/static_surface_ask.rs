use xrf_material::XraySurfaceDescriptor;
use xrf_visual::SectorSurface;

use crate::scene::static_scene::surface::static_surface_key::StaticSurfaceKey;

/// A surface asked of the static scene's rows: the key its row is kept by, the surface, and the descriptor its shader
/// entry is dressed by, where its shader table has one.
#[derive(Clone)]
pub struct StaticSurfaceAsk<'a> {
  pub key: StaticSurfaceKey,
  pub surface: &'a SectorSurface,
  pub descriptor: Option<&'a XraySurfaceDescriptor>,
}
