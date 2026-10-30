/// One end of a helper's wire as a view draws it: where it stands, its colour with whether its object is selected, when
/// it fades and how strongly it shows hidden, and the id a pick of it answers.
#[repr(C)]
#[derive(Clone, Copy, Debug, Default, PartialEq, bytemuck::Pod, bytemuck::Zeroable)]
pub struct HelperVertex {
  pub position: [f32; 3],
  /// sRGB, then one where its object is selected.
  pub color: [f32; 4],
  /// The distance it has faded out by, none for a selected object's, then how strongly it shows behind what is drawn.
  pub fade: [f32; 2],
  pub object: u32,
}

impl HelperVertex {
  /// Bytes a vertex takes.
  pub const SIZE: u64 = size_of::<Self>() as u64;

  /// The vertex attributes, by location: position, colour, fade, object.
  pub const ATTRIBUTES: [wgpu::VertexAttribute; 4] =
    wgpu::vertex_attr_array![0 => Float32x3, 1 => Float32x4, 2 => Float32x2, 3 => Uint32];

  /// A segment's two ends read as one instance: the first end's position and object, then the second end's position.
  pub const SEGMENT_ATTRIBUTES: [wgpu::VertexAttribute; 3] = [
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
    wgpu::VertexAttribute {
      format: wgpu::VertexFormat::Float32x3,
      offset: Self::SIZE,
      shader_location: 2,
    },
  ];
}
