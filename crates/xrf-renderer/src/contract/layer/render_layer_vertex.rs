use bytemuck::{Pod, Zeroable};

/// A vertex of a layer mesh, as the GPU reads it: where it stands in device pixels from the window's top left, where it
/// samples its texture, and its colour, eight bits a channel, premultiplied, display encoded (the window's swapchain is
/// not sRGB, so what a layer writes is shown as written).
#[repr(C)]
#[derive(Clone, Copy, Debug, Default, PartialEq, Pod, Zeroable)]
pub struct RenderLayerVertex {
  pub position: [f32; 2],
  pub uv: [f32; 2],
  pub color: [u8; 4],
}
