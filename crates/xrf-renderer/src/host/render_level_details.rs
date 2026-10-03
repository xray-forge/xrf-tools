use xrf_material::XraySurfaceDescriptor;
use xrf_visual::DetailsPackage;

/// A level's grass as its loader reads it: the slots, bins and models packed, and how each model is drawn.
#[derive(Debug)]
pub struct RenderLevelDetails {
  pub package: DetailsPackage,
  /// How each model is drawn, in their order; its base texture is read through
  /// [`crate::RenderAssetSource::read_texture`] by the name the model gives.
  pub surfaces: Vec<XraySurfaceDescriptor>,
}
