/// One helper's icon as a view draws it: where it stands, its cell of the atlas, its colour with how its object is
/// singled out, when it fades and how strongly it shows hidden, and the id a pick of it answers.
#[repr(C)]
#[derive(Clone, Copy, Debug, Default, PartialEq, bytemuck::Pod, bytemuck::Zeroable)]
pub struct HelperIconInstance {
  pub position: [f32; 3],
  /// Its cell, `RenderHelperIcon::get_index`.
  pub icon: u32,
  /// sRGB, then how its object is singled out, as `HelperVertex::color` holds it.
  pub color: [f32; 4],
  /// As `HelperVertex::fade` holds it.
  pub fade: [f32; 2],
  pub object: u32,
  pub _pad: u32,
}

impl HelperIconInstance {
  /// Bytes an instance takes.
  pub const SIZE: u64 = size_of::<Self>() as u64;

  /// The instance attributes, by location: position, cell, colour, fade, object.
  pub const ATTRIBUTES: [wgpu::VertexAttribute; 5] =
    wgpu::vertex_attr_array![0 => Float32x3, 1 => Uint32, 2 => Float32x4, 3 => Float32x2, 4 => Uint32];

  /// What a pick reads of an instance: its position and its object.
  pub const PICK_ATTRIBUTES: [wgpu::VertexAttribute; 2] = [
    wgpu::VertexAttribute {
      format: wgpu::VertexFormat::Float32x3,
      offset: 0,
      shader_location: 0,
    },
    wgpu::VertexAttribute {
      format: wgpu::VertexFormat::Uint32,
      offset: std::mem::offset_of!(Self, object) as u64,
      shader_location: 1,
    },
  ];
}
