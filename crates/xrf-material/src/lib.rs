#![doc = include_str!("../README.md")]

pub(crate) mod data;
pub(crate) mod resolve;

#[cfg(any(test, feature = "fixtures"))]
pub mod fixtures;

#[cfg(test)]
mod tests;

pub use crate::data::xray_bump_fallback::XrayBumpFallback;
pub use crate::data::xray_bump_mode::XrayBumpMode;
pub use crate::data::xray_bump_naming::XrayBumpNaming;
pub use crate::data::xray_bump_outcome::XrayBumpOutcome;
pub use crate::data::xray_detail_usage::XrayDetailUsage;
pub use crate::data::xray_material_bump::XrayMaterialBump;
pub use crate::data::xray_material_bump_input::XrayMaterialBumpInput;
pub use crate::data::xray_material_declaration::XrayMaterialDeclaration;
pub use crate::data::xray_material_descriptor::XrayMaterialDescriptor;
pub use crate::data::xray_material_detail::XrayMaterialDetail;
pub use crate::data::xray_surface_declaration::XraySurfaceDeclaration;
pub use crate::data::xray_surface_descriptor::XraySurfaceDescriptor;
pub use crate::data::xray_surface_detail::XraySurfaceDetail;
pub use crate::data::xray_surface_draw::XraySurfaceDraw;
pub use crate::data::xray_surface_sampler::XraySurfaceSampler;
pub use crate::data::xray_surface_terrain::XraySurfaceTerrain;
pub use crate::data::xray_surface_terrain_layer::XraySurfaceTerrainLayer;
pub use crate::resolve::xray_material_resolver::XrayMaterialResolver;
pub use crate::resolve::xray_reduced_textures::XrayReducedTextures;
pub use crate::resolve::xray_surface_resolver::XraySurfaceResolver;
pub use crate::resolve::xray_texture_scope::XrayTextureScope;
