use std::sync::mpsc::Sender;

use xrf_error::{XrfError, XrfResult};

use crate::contract::render_capture::RenderCapture;
use crate::contract::render_rect::RenderRect;

/// Where a capture's reply goes.
pub type CaptureReply = Sender<XrfResult<RenderCapture>>;

/// Copies a viewport's rectangle out of the frame just drawn and reads it back, for a capture asked of the renderer
/// rather than of the screen, which another window may cover.
pub fn capture_frame(
  device: &wgpu::Device,
  queue: &wgpu::Queue,
  frame: &wgpu::Texture,
  rect: RenderRect,
) -> XrfResult<RenderCapture> {
  let format: wgpu::TextureFormat = frame.format();

  if !matches!(
    format,
    wgpu::TextureFormat::Bgra8Unorm | wgpu::TextureFormat::Rgba8Unorm
  ) {
    return Err(XrfError::new_not_implemented_error(format!(
      "Capturing a {format:?} frame"
    )));
  }

  let row: u32 = (rect.width * 4).next_multiple_of(wgpu::COPY_BYTES_PER_ROW_ALIGNMENT);
  let buffer: wgpu::Buffer = device.create_buffer(&wgpu::BufferDescriptor {
    label: Some("capture"),
    size: (row * rect.height) as u64,
    usage: wgpu::BufferUsages::COPY_DST | wgpu::BufferUsages::MAP_READ,
    mapped_at_creation: false,
  });
  let mut encoder: wgpu::CommandEncoder = device.create_command_encoder(&Default::default());

  encoder.copy_texture_to_buffer(
    wgpu::TexelCopyTextureInfo {
      texture: frame,
      mip_level: 0,
      origin: wgpu::Origin3d {
        x: rect.x as u32,
        y: rect.y as u32,
        z: 0,
      },
      aspect: wgpu::TextureAspect::All,
    },
    wgpu::TexelCopyBufferInfo {
      buffer: &buffer,
      layout: wgpu::TexelCopyBufferLayout {
        offset: 0,
        bytes_per_row: Some(row),
        rows_per_image: Some(rect.height),
      },
    },
    wgpu::Extent3d {
      width: rect.width,
      height: rect.height,
      depth_or_array_layers: 1,
    },
  );
  queue.submit([encoder.finish()]);
  buffer.slice(..).map_async(wgpu::MapMode::Read, |_| ());
  device
    .poll(wgpu::PollType::wait_indefinitely())
    .map_err(|error| XrfError::new_unexpected_error(format!("Capture readback: {error}")))?;

  let mapped = buffer
    .slice(..)
    .get_mapped_range()
    .map_err(|error| XrfError::new_unexpected_error(format!("Capture readback: {error}")))?;
  let mut pixels: Vec<u8> = Vec::with_capacity((rect.width * rect.height * 4) as usize);

  for line in mapped.chunks_exact(row as usize) {
    for texel in line[..(rect.width * 4) as usize].as_chunks::<4>().0 {
      match format {
        wgpu::TextureFormat::Bgra8Unorm => pixels.extend_from_slice(&[texel[2], texel[1], texel[0], 255]),
        _ => pixels.extend_from_slice(&[texel[0], texel[1], texel[2], 255]),
      }
    }
  }

  Ok(RenderCapture {
    width: rect.width,
    height: rect.height,
    pixels,
  })
}
