use bytemuck::{Pod, Zeroable};

/// What the upscale passes read, as `frame/upscale.wgsl`'s `Upscale` lays it out.
#[repr(C)]
#[derive(Clone, Copy, Debug, Default, Pod, Zeroable)]
pub struct UpscaleUniform {
  pub output_size: [f32; 2],
  pub sharpness: f32,
  pub pad: f32,
}
