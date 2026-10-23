#![doc = include_str!("../README.md")]

pub(crate) mod encode;
pub(crate) mod file;
pub(crate) mod mip;
pub(crate) mod renderer;

pub use ddsfile::{D3DFormat, DxgiFormat};
pub use image::{Rgba, RgbaImage};
pub use image_dds::{ImageFormat, Quality};

pub use crate::encode::{DDS_BLOCK_SIZE, DdsEncodeAttempt, DdsEncodeCandidate, DdsEncoding, DdsImageDifference};
pub use crate::file::{DdsFile, DdsFormat, DdsMetadata, DdsPng};
pub use crate::mip::{DdsMipChain, DdsMipFilter, DdsMipmaps};
pub use crate::renderer::{DdsFormatSupport, DdsRenderer};
