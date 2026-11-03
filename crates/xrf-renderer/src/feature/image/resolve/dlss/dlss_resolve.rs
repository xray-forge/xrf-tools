use xrf_dlss::DlssUpscaler;

use crate::frame::targets::frame_texture::FrameTexture;

/// DLSS's state: its upscaler, the frame it writes at the viewport's size, and whether the next frame forgets the ones
/// before.
pub struct DlssResolve {
  pub upscaler: DlssUpscaler,
  pub output: FrameTexture,
  pub is_reset: bool,
}

impl DlssResolve {
  /// What DLSS writes the upscaled frame as: colour of the scene's range, bound as storage.
  pub const FORMAT: wgpu::TextureFormat = wgpu::TextureFormat::Rgba16Float;

  /// The frame an NGX feature writes, `size` pixels, cleared as it is made. NGX writes it as storage and clears it with
  /// a transfer command, all outside wgpu: left unwritten, wgpu would take its first frame for one never written and
  /// clear it under the pass reading it, after NGX wrote it, so every new view's first frame would show black.
  pub fn create_output(device: &wgpu::Device, queue: &wgpu::Queue, label: &str, size: (u32, u32)) -> FrameTexture {
    let output: FrameTexture = FrameTexture::new(
      device,
      label,
      size,
      Self::FORMAT,
      wgpu::TextureUsages::STORAGE_BINDING | wgpu::TextureUsages::COPY_DST | wgpu::TextureUsages::RENDER_ATTACHMENT,
    );
    let mut encoder: wgpu::CommandEncoder =
      device.create_command_encoder(&wgpu::CommandEncoderDescriptor { label: Some(label) });

    encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
      label: Some(label),
      color_attachments: &[Some(wgpu::RenderPassColorAttachment {
        view: &output.view,
        depth_slice: None,
        resolve_target: None,
        ops: wgpu::Operations {
          load: wgpu::LoadOp::Clear(wgpu::Color::TRANSPARENT),
          store: wgpu::StoreOp::Store,
        },
      })],
      ..Default::default()
    });
    queue.submit([encoder.finish()]);

    output
  }
}
