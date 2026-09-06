//! DDS textures as this workspace reads, writes and weighs them: the header facts a surface reports, the pixels behind
//! them, the X-Ray converter's mip kernels, and which renderer path will load what.

pub(crate) mod encode;
pub(crate) mod file;
pub(crate) mod mip;
pub(crate) mod renderer;

pub use ddsfile::{D3DFormat, DxgiFormat};
pub use image::RgbaImage;
pub use image_dds::{ImageFormat, Quality};

pub use crate::encode::{DdsEncodeAttempt, DdsEncodeCandidate, DdsEncoding, DdsImageDifference};
pub use crate::file::{DdsFile, DdsFormat, DdsMetadata, DdsPng};
pub use crate::mip::{DdsMipChain, DdsMipFilter, DdsMipmaps};
pub use crate::renderer::{DdsFormatSupport, DdsRenderer};
