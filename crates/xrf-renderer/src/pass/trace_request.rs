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
  /// Words of the request list's head a frame starts from: requests made, then the scene pass's dispatch (workgroups
  /// across, down, deep), one workgroup more than the requests take so the scene pass always keeps the wait.
  pub const HEAD: [u32; 4] = [0, 1, 1, 1];
  /// Words the head holds: `HEAD`'s, then the wait its pixels' turns are stretched by, which a frame keeps from the
  /// last.
  pub const HEAD_WORDS: usize = Self::HEAD.len() + 1;
  /// Where the scene pass's dispatch starts in the head: after the count of requests made.
  pub const HEAD_DISPATCH_OFFSET: u64 = size_of::<u32>() as u64;
  /// Bytes an indirect dispatch takes.
  pub const DISPATCH_BYTES: u64 = size_of::<wgpu::util::DispatchIndirectArgs>() as u64;

  /// Requests a workgroup of the scene pass traces.
  pub const WORKGROUP: u32 = 64;
}
