#![doc = include_str!("../README.md")]

mod build;
mod bump;
mod crop;
mod description;
mod encoding;
mod equipment;
mod image_file;

pub mod job_phases;

pub use crate::build::{
  BuildTextureOmission, BuildTextureOptions, BuildTextureProcessor, BuildTextureRecipe, BuildTextureResult,
};
pub use crate::bump::{GenerateBumpGloss, GenerateBumpOptions, GenerateBumpProcessor, GenerateBumpResult};
pub use crate::crop::{CropTextureOptions, CropTextureProcessor, CropTextureResult};
pub use crate::description::{
  PackDescriptionProcessor, SpriteDescriptionOptions, SpriteDescriptionResult, TextureFileDescriptor,
  TextureSpriteDescriptor, UnpackDescriptionProcessor,
};
pub use crate::encoding::{TextureEncodingFormat, TextureEncodingQuality};
pub use crate::equipment::{
  EquipmentGridOverlap, EquipmentSlotClaim, EquipmentSlotOccupant, InventorySpriteDescriptor, PackEquipmentOptions,
  PackEquipmentProcessor, PackEquipmentResult, UnpackEquipmentOptions, UnpackEquipmentProcessor,
  VerifyEquipmentGridProcessor,
};
pub use crate::image_file::{ImageShape, read_image_as_rgba};
