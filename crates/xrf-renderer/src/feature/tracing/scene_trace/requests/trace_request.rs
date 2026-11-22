use xrf_math::glam::UVec2;
use xrf_renderer_core::ShaderStruct;

/// A reflection ray the screen left unanswered, which the scene pass traces against the level's geometry.
///
/// Packed into ten words, as a frame-sized list is one of the frame's largest buffers.
#[repr(C)]
#[derive(Clone, Copy, Debug, Default, bytemuck::Pod, bytemuck::Zeroable, ShaderStruct)]
#[shader(name = "TraceRequest")]
pub(crate) struct TraceRequest {
  /// Where it starts, in renderer space.
  pub origin: [f32; 3],
  /// How far it reaches, then the length of what the screen answered where it holds most, as halves.
  pub reach: u32,
  /// What the screen answered, its colour already weighed by its share, then the share, as four halves.
  pub screen: UVec2,
  /// Its cone's width where it starts and its spread, as halves.
  pub cone: u32,
  /// Its direction, octahedral, as two snorm halves.
  pub toward: u32,
  /// Its traced pixel, `x` in the low half and `y` in the high.
  pub pixel: u32,
  /// The reason the pixel's ends take where the ray meets something.
  pub hit_end: u32,
}

impl TraceRequest {
  /// Words of the request list's head a frame starts from: requests made, then the scene pass's dispatch (workgroups
  /// across, down, deep), one workgroup more than the requests take so the scene pass always keeps the wait.
  pub(crate) const HEAD: [u32; 4] = [0, 1, 1, 1];
  /// Words the head holds: `HEAD`'s, then the wait its pixels' turns are stretched by, which a frame keeps from the
  /// last.
  pub(crate) const HEAD_WORDS: usize = Self::HEAD.len() + 1;
  /// Where the wait stands in the head.
  pub(crate) const HEAD_WAIT_OFFSET: u64 = (Self::HEAD.len() * size_of::<u32>()) as u64;
  /// Where the scene pass's dispatch starts in the head: after the count of requests made.
  pub(crate) const HEAD_DISPATCH_OFFSET: u64 = size_of::<u32>() as u64;
  /// Bytes an indirect dispatch takes.
  pub(crate) const DISPATCH_BYTES: u64 = size_of::<wgpu::util::DispatchIndirectArgs>() as u64;

  /// Requests a workgroup of the scene pass traces.
  pub(crate) const WORKGROUP: u32 = 64;
}
