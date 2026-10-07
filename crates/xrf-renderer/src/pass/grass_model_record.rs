use glam::Vec4;
use xrf_renderer_core::ShaderStruct;

/// One detail model as the grass's passes read it: its least and most scale, radius and height, whether it waves, its
/// base texture's slot, its alpha reference, where its indices start in the grass's index arena, and how far its
/// vertices reach from its foot across the ground, which a traced hit covers the ground by.
#[repr(C)]
#[derive(Clone, Copy, Debug, Default, bytemuck::Pod, bytemuck::Zeroable, ShaderStruct)]
#[shader(name = "GrassModel")]
pub struct GrassModelRecord {
  pub shape: Vec4,
  pub is_waving: f32,
  pub texture: u32,
  pub alpha_reference: f32,
  pub index_count: u32,
  pub first_index: u32,
  pub spread: f32,
  pub pad: [u32; 2],
}
