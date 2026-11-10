use glam::UVec2;
use xrf_renderer_core::ShaderStruct;

/// A reflection ray the screen left unanswered on its turn, which the scene pass traces against the level's geometry,
/// in ten words, a list as large as the frame being one of the frame's largest buffers: where it starts in renderer
/// space; how far it reaches, then the length of what the screen answered where it holds most (two halves); what the
/// screen answered, its colour already weighed by its share, then the share (four halves); its cone's width where it
/// starts and its spread (two halves); its direction (octahedral, two snorm halves); its traced pixel (`x` in the low
/// half, `y` in the high); and the reason its ends take on a hit.
#[repr(C)]
#[derive(Clone, Copy, Debug, Default, bytemuck::Pod, bytemuck::Zeroable, ShaderStruct)]
#[shader(name = "TraceRequest")]
pub(crate) struct TraceRequest {
  pub origin: [f32; 3],
  pub reach: u32,
  pub screen: UVec2,
  pub cone: u32,
  pub toward: u32,
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
