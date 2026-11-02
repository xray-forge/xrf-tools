#![doc = include_str!("../README.md")]

mod albedo;
mod alignment;
mod build;
mod bump;
mod crop;
mod description;
mod encoding;
mod equipment;
mod image_file;

pub mod job_phases;

pub use crate::albedo::{AlbedoVerdict, TextureAlbedo};
pub use crate::alignment::{BumpAlignment, BumpAlignmentVerdict};
pub use crate::build::{
  BuildTextureAdvancedChain, BuildTextureBumpPair, BuildTextureLayout, BuildTextureMipmaps, BuildTextureNormalMap,
  BuildTextureOmission, BuildTextureOptions, BuildTextureProcessor, BuildTextureRecipe, BuildTextureResult,
};
pub use crate::bump::{GenerateBumpGloss, GenerateBumpOptions, GenerateBumpProcessor, GenerateBumpResult};
pub use crate::crop::{CropTextureOptions, CropTextureProcessor, CropTextureResult};
pub use crate::description::{
  DescriptionSpriteOptions, DescriptionSpritePacker, DescriptionSpriteResult, DescriptionSpriteUnpacker,
  TextureFileDescriptor, TextureSpriteDescriptor,
};
pub use crate::encoding::{TextureEncodingFormat, TextureEncodingQuality};
pub use crate::equipment::{
  EquipmentGridOverlap, EquipmentGridVerifier, EquipmentSlotClaim, EquipmentSlotOccupant, EquipmentSpritePackOptions,
  EquipmentSpritePackResult, EquipmentSpritePacker, EquipmentSpriteUnpackOptions, EquipmentSpriteUnpacker,
  InventorySpriteDescriptor,
};
pub use crate::image_file::{ImageFileReader, ImageShape};
