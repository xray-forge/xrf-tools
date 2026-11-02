#![doc = include_str!("../README.md")]

mod material;
mod surface;
mod texture;

#[cfg(any(test, feature = "fixtures"))]
pub mod fixtures;

#[cfg(test)]
mod tests;

pub use crate::material::{
  XrayBumpFallback, XrayBumpMode, XrayBumpNaming, XrayBumpOutcome, XrayDetailUsage, XrayMaterialBump,
  XrayMaterialBumpInput, XrayMaterialDeclaration, XrayMaterialDescriptor, XrayMaterialDetail, XrayMaterialResolver,
};
pub use crate::surface::{
  XraySurfaceDeclaration, XraySurfaceDescriptor, XraySurfaceDetail, XraySurfaceDraw, XraySurfaceResolver,
  XraySurfaceSampler, XraySurfaceTerrain, XraySurfaceTerrainLayer,
};
pub use crate::texture::{XrayReducedTextures, XrayTextureScope};
