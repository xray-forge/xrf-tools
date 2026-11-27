#![doc = include_str!("../README.md")]

mod albedo;
mod alignment;
mod build;
mod bump;
mod crop;
mod encoding;
mod image_file;
mod output;

pub mod job_phases;

pub use crate::albedo::{AlbedoVerdict, TextureAlbedo};
pub use crate::alignment::{BumpAlignment, BumpAlignmentLevel, BumpAlignmentVerdict};
pub use crate::build::{
  BuildTextureAdvancedChain, BuildTextureBumpPair, BuildTextureFade, BuildTextureLayout, BuildTextureMipmaps,
  BuildTextureNormalMap, BuildTextureOmission, BuildTextureOptions, BuildTextureProcessor, BuildTextureRecipe,
  BuildTextureResult,
};
pub use crate::bump::{
  GenerateBumpFormat, GenerateBumpGloss, GenerateBumpOptions, GenerateBumpProcessor, GenerateBumpResult,
};
pub use crate::crop::{CropTextureOptions, CropTextureProcessor, CropTextureResult};
pub use crate::encoding::{TextureEncodedDescriptor, TextureEncodingFormat, TextureEncodingQuality};
pub use crate::image_file::{ImageFileReader, ImageFileReshape, ImageFileWriter, ImageFit, ImageShape};
pub use crate::output::{TextureDiskWriter, TextureFileWriter};
