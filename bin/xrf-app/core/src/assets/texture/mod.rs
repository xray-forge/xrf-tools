//! What a texture asset is: its header's shape, its descriptor, and its pixels decoded for a page.

mod asset_texture_descriptor;
mod asset_texture_image;
mod asset_texture_shape;

pub use asset_texture_descriptor::AssetTextureDescriptor;
pub use asset_texture_image::AssetTextureImage;
pub use asset_texture_shape::AssetTextureShape;
