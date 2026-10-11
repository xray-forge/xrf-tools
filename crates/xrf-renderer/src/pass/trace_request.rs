use glam::{UVec2, Vec2, Vec4};
use xrf_renderer_core::ShaderStruct;

/// A reflection ray the screen left unanswered on its turn, which the scene pass traces against the level's geometry:
/// where it starts in renderer space and how far it reaches, its cone, its direction (octahedral, two snorm halves), its
/// traced pixel (`x` in the low half, `y` in the high), and what the screen answered: that answer's colour already
/// weighed by its share, then the share (four halves), and its length where it holds most; and the reason its ends
/// take on a hit.
#[repr(C)]
#[derive(Clone, Copy, Debug, Default, bytemuck::Pod, bytemuck::Zeroable, ShaderStruct)]
#[shader(name = "TraceRequest")]
pub struct TraceRequest {
  pub origin: Vec4,
  pub cone: Vec2,
  pub toward: u32,
  pub pixel: u32,
  pub screen: UVec2,
  pub screen_length: f32,
  /// The reason the pixel's ends take where the ray meets something.
  pub hit_end: u32,
}

impl TraceRequest {
  /// Words of the request list's head: requests made, then the scene pass's dispatch (workgroups across, down, deep).
  pub const HEAD: [u32; 4] = [0, 0, 1, 1];

  /// Requests a workgroup of the scene pass traces.
  pub const WORKGROUP: u32 = 64;
}
