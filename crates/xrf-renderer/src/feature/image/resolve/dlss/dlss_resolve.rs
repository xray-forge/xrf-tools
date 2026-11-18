use xrf_dlss::DlssUpscaler;

use crate::feature::image::resolve::dlss::ngx_evaluation::NgxEvaluation;
use crate::frame::targets::frame_texture::FrameTexture;

/// A view's DLSS upscaler and what it writes.
pub(crate) struct DlssResolve {
  pub upscaler: DlssUpscaler,
  /// The frame it writes, at the viewport's size.
  pub output: FrameTexture,
  /// Whether the next frame forgets the ones before.
  pub is_reset: bool,
  pub evaluation: NgxEvaluation,
}

impl DlssResolve {
  /// What DLSS writes the upscaled frame as: colour of the scene's range, bound as storage.
  pub(crate) const FORMAT: wgpu::TextureFormat = wgpu::TextureFormat::Rgba16Float;

  /// The frame an NGX feature writes, `size` pixels, cleared as it is made.
  ///
  /// NGX writes it outside wgpu, so wgpu would otherwise take it as never written and clear it after NGX wrote it,
  /// showing every new view's first frame black.
  pub(crate) fn create_output(
    device: &wgpu::Device,
    queue: &wgpu::Queue,
    label: &str,
    size: (u32, u32),
  ) -> FrameTexture {
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
