//! The textures a surface binds beside its base: the detail, the terrain layers and a script's samplers.

mod xray_surface_detail;
mod xray_surface_sampler;
mod xray_surface_terrain;
mod xray_surface_terrain_layer;

pub use xray_surface_detail::XraySurfaceDetail;
pub use xray_surface_sampler::XraySurfaceSampler;
pub use xray_surface_terrain::XraySurfaceTerrain;
pub use xray_surface_terrain_layer::XraySurfaceTerrainLayer;
