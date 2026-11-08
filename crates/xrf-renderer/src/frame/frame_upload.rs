use crate::frame::frame_encoder::FrameEncoder;

/// What a frame's uploads and copies go through: the device, its queue, and the frame's encoder.
pub(crate) struct FrameUpload<'a> {
  pub device: &'a wgpu::Device,
  pub queue: &'a wgpu::Queue,
  pub encoder: &'a mut FrameEncoder,
}
