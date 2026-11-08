use glam::Vec4;
use xrf_renderer_core::ShaderStruct;
use xrf_visual::DetailsModel;

/// One detail model as the grass's passes read it: its least and most scale, radius and height, whether it waves, its
/// base texture's slot, its alpha reference, where its indices start in the grass's index arena, and how far its
/// vertices reach from its foot across the ground, which a traced hit covers the ground by.
#[repr(C)]
#[derive(Clone, Copy, Debug, Default, bytemuck::Pod, bytemuck::Zeroable, ShaderStruct)]
#[shader(name = "GrassModel")]
pub(crate) struct GrassModelRecord {
  pub shape: Vec4,
  pub is_waving: f32,
  pub texture: u32,
  pub alpha_reference: f32,
  pub index_count: u32,
  pub first_index: u32,
  pub spread: f32,
  pub pad: [u32; 2],
}

impl GrassModelRecord {
  /// `model`'s record, drawn with the texture in slot `texture`, cut at `reference` of 255, over the index range
  /// `(first_index, index_count)` of the grass's arena, reaching `spread` across the ground.
  pub(crate) fn of(
    model: &DetailsModel,
    texture: u32,
    reference: u8,
    (first_index, index_count): (u32, u32),
    spread: f32,
  ) -> Self {
    Self {
      shape: Vec4::new(model.min_scale, model.max_scale, model.radius, model.height),
      is_waving: if model.is_waving { 1.0 } else { 0.0 },
      texture,
      alpha_reference: f32::from(reference) / 255.0,
      index_count,
      first_index,
      spread,
      pad: [0; 2],
    }
  }
}
