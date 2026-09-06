//! X-Ray texture recipes: the sprite sheets the interface is drawn from, the bump pair a surface binds, and the
//! operations that build each of them.

mod build;
mod bump;
mod crop;
mod description;
mod equipment;
mod image_file;

pub mod job_phases;

pub use crate::build::{
  BuildTextureOmission, BuildTextureOptions, BuildTextureProcessor, BuildTextureRecipe, BuildTextureResult,
};
pub use crate::bump::{GenerateBumpGloss, GenerateBumpOptions, GenerateBumpProcessor, GenerateBumpResult};
pub use crate::crop::{CropTextureOptions, CropTextureProcessor, CropTextureResult};
pub use crate::description::{
  PackDescriptionOptions, PackDescriptionProcessor, TextureFileDescriptor, TextureSpriteDescriptor,
  UnpackDescriptionProcessor,
};
pub use crate::equipment::{
  EquipmentGridOverlap, InventorySpriteDescriptor, PackEquipmentOptions, PackEquipmentProcessor, PackEquipmentResult,
  UnpackEquipmentOptions, UnpackEquipmentProcessor, VerifyEquipmentGridProcessor,
};
