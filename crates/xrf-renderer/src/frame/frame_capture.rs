use std::sync::mpsc::Sender;

use xrf_error::{XrfError, XrfResult};

use crate::contract::render_capture::RenderCapture;
use crate::contract::render_rect::RenderRect;
use crate::frame::gpu_readback::GpuReadback;

/// Where a capture's reply goes.
pub type CaptureReply = Sender<XrfResult<RenderCapture>>;

/// A viewport's rectangle of a presented frame on its way back, for a capture asked of the renderer rather than of the
/// screen, which another window may cover: copied out with the frame, read once a later frame's poll finds it mapped.
pub struct FrameCapture {
  readback: GpuReadback,
  rect: RenderRect,
  format: wgpu::TextureFormat,
  /// Bytes a row of the copy takes, aligned as copies need.
  row: u32,
  frame: u64,
}

impl FrameCapture {
  /// A capture of `rect` of a frame of `format`, the frame `frame`, waiting for its copy.
  ///
  /// # Errors
  ///
  /// Returns an error for a frame of a format a capture cannot read.
  pub fn new(device: &wgpu::Device, format: wgpu::TextureFormat, rect: RenderRect, frame: u64) -> XrfResult<Self> {
    if !matches!(
      format,
      wgpu::TextureFormat::Bgra8Unorm | wgpu::TextureFormat::Rgba8Unorm
    ) {
      return Err(XrfError::new_not_implemented_error(format!(
        "Capturing a {format:?} frame"
      )));
    }

    let row: u32 = (rect.width * 4).next_multiple_of(wgpu::COPY_BYTES_PER_ROW_ALIGNMENT);

    Ok(Self {
      readback: GpuReadback::new(device, "capture", (row * rect.height) as u64),
      rect,
      format,
      row,
      frame,
    })
  }

  /// Copies its rectangle out of `texture`, the frame drawn; asked for once the copy is submitted.
  pub fn encode(&self, encoder: &mut wgpu::CommandEncoder, texture: &wgpu::Texture) {
    let rect: RenderRect = self.rect;

    encoder.copy_texture_to_buffer(
      wgpu::TexelCopyTextureInfo {
        texture,
        mip_level: 0,
        origin: wgpu::Origin3d {
          x: rect.x as u32,
          y: rect.y as u32,
          z: 0,
        },
        aspect: wgpu::TextureAspect::All,
      },
      wgpu::TexelCopyBufferInfo {
        buffer: self.readback.get_buffer(),
        layout: wgpu::TexelCopyBufferLayout {
          offset: 0,
          bytes_per_row: Some(self.row),
          rows_per_image: Some(rect.height),
        },
      },
      wgpu::Extent3d {
        width: rect.width,
        height: rect.height,
        depth_or_array_layers: 1,
      },
    );
    self.readback.mark_recorded();
  }

  /// Asks for the copy back, after the frame copying it is submitted.
  pub fn request(&self) {
    self.readback.request();
  }

  /// The capture once it is back, as eight bit RGBA rows top to bottom; nothing while it is on its way.
  pub fn take(&self) -> Option<XrfResult<RenderCapture>> {
    let (width, height, row) = (self.rect.width, self.rect.height, self.row as usize);

    self.readback.take(|bytes| {
      let mut pixels: Vec<u8> = Vec::with_capacity((width * height * 4) as usize);

      for line in bytes.chunks_exact(row) {
        for texel in line[..(width * 4) as usize].as_chunks::<4>().0 {
          match self.format {
            wgpu::TextureFormat::Bgra8Unorm => pixels.extend_from_slice(&[texel[2], texel[1], texel[0], 255]),
            _ => pixels.extend_from_slice(&[texel[0], texel[1], texel[2], 255]),
          }
        }
      }

      RenderCapture {
        width,
        height,
        pixels,
        frame: self.frame,
      }
    })
  }
}
