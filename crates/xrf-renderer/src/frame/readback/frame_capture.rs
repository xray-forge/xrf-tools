use std::sync::mpsc::Sender;

use xrf_error::{XrfError, XrfResult};

use crate::contract::capture::render_capture::RenderCapture;
use crate::contract::capture::render_screenshot::RenderScreenshot;
use crate::contract::viewport::render_rect::RenderRect;
use crate::frame::readback::gpu_readback::GpuReadback;

/// Where a capture's reply goes.
pub(crate) type CaptureReply = Sender<XrfResult<RenderCapture>>;

/// A viewport's rectangle of a presented frame on its way back, for a capture asked of the renderer rather than of the
/// screen, which another window may cover: copied out with the frame, read once a later frame's poll finds it mapped.
/// A screenshot's is the viewport's finished frame whole, drawn apart from the window without its overlays, its level
/// object helpers over it where asked.
pub(crate) struct FrameCapture {
  readback: GpuReadback,
  rect: RenderRect,
  format: wgpu::TextureFormat,
  /// Bytes a row of the copy takes, aligned as copies need.
  row: u32,
  frame: u64,
  is_screenshot: bool,
  /// How a screenshot is drawn, the defaults for any other capture.
  screenshot: RenderScreenshot,
  /// A level map's tile's depth on its way back beside it, and the bytes a row of its copy takes: its coverage, opaque
  /// where anything was drawn, clear where the depth stayed at the far plane's zero.
  coverage: Option<(GpuReadback, u32)>,
}

impl FrameCapture {
  /// A capture of `rect` of a frame of `format`, the frame `frame`, waiting for its copy.
  ///
  /// # Errors
  ///
  /// Returns an error for a frame of a format a capture cannot read.
  pub(crate) fn new(
    device: &wgpu::Device,
    format: wgpu::TextureFormat,
    rect: RenderRect,
    frame: u64,
  ) -> XrfResult<Self> {
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
      readback: GpuReadback::new(device, "capture", u64::from(row) * u64::from(rect.height)),
      rect,
      format,
      row,
      frame,
      is_screenshot: false,
      screenshot: RenderScreenshot::default(),
      coverage: None,
    })
  }

  /// A screenshot of a viewport `size` pixels across, drawn in `format`, the frame `frame`, waiting for its copy.
  ///
  /// # Errors
  ///
  /// Returns an error for a frame of a format a capture cannot read.
  pub(crate) fn new_screenshot(
    device: &wgpu::Device,
    format: wgpu::TextureFormat,
    (width, height): (u32, u32),
    frame: u64,
  ) -> XrfResult<Self> {
    let rect: RenderRect = RenderRect {
      x: 0,
      y: 0,
      width,
      height,
    };

    Ok(Self {
      is_screenshot: true,
      ..Self::new(device, format, rect, frame)?
    })
  }

  /// The same, a shot `width` by `height` pixels across, its depth's coverage waiting for its copy too, as a map's tile's
  /// is.
  pub(crate) fn with_coverage(self, device: &wgpu::Device, (width, height): (u32, u32)) -> Self {
    let row: u32 = (width * 4).next_multiple_of(wgpu::COPY_BYTES_PER_ROW_ALIGNMENT);

    Self {
      coverage: Some((
        GpuReadback::new(device, "map tile coverage", u64::from(row) * u64::from(height)),
        row,
      )),
      ..self
    }
  }

  /// The same, drawn as `screenshot` asks.
  pub(crate) fn with_screenshot(self, screenshot: RenderScreenshot) -> Self {
    Self { screenshot, ..self }
  }

  /// Whether it is a screenshot, which its viewport's own pass copies out rather than the window's.
  pub(crate) fn is_screenshot(&self) -> bool {
    self.is_screenshot
  }

  /// Whether its view's level object helpers are drawn over it.
  pub(crate) fn is_helped(&self) -> bool {
    self.screenshot.is_helped
  }

  /// The pixels it copies across and down.
  pub(crate) fn get_size(&self) -> (u32, u32) {
    (self.rect.width, self.rect.height)
  }

  /// The format it reads.
  pub(crate) fn get_format(&self) -> wgpu::TextureFormat {
    self.format
  }

  /// Copies its rectangle out of `texture`, the frame drawn; asked for once the copy is submitted.
  pub(crate) fn encode(&self, encoder: &mut wgpu::CommandEncoder, texture: &wgpu::Texture) {
    let rect: RenderRect = self.rect;

    encoder.copy_texture_to_buffer(
      wgpu::TexelCopyTextureInfo {
        texture,
        mip_level: 0,
        origin: wgpu::Origin3d {
          x: rect.x.cast_unsigned(),
          y: rect.y.cast_unsigned(),
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

  /// Copies a level map's tile's depth out of `depth`, the frame's depth target, beside its picture.
  pub(crate) fn encode_coverage(&self, encoder: &mut wgpu::CommandEncoder, depth: &wgpu::Texture) {
    let Some((readback, row)) = &self.coverage else {
      return;
    };
    let (width, height): (u32, u32) = self.get_size();

    encoder.copy_texture_to_buffer(
      wgpu::TexelCopyTextureInfo {
        texture: depth,
        mip_level: 0,
        origin: wgpu::Origin3d::ZERO,
        aspect: wgpu::TextureAspect::DepthOnly,
      },
      wgpu::TexelCopyBufferInfo {
        buffer: readback.get_buffer(),
        layout: wgpu::TexelCopyBufferLayout {
          offset: 0,
          bytes_per_row: Some(*row),
          rows_per_image: Some(height),
        },
      },
      wgpu::Extent3d {
        width,
        height,
        depth_or_array_layers: 1,
      },
    );
    readback.mark_recorded();
  }

  /// Whether its copies were recorded with the frame, which a frame that drew nothing of it did not.
  pub(crate) fn is_recorded(&self) -> bool {
    self.readback.is_recorded()
      && self
        .coverage
        .as_ref()
        .is_none_or(|(readback, _)| readback.is_recorded())
  }

  /// Asks for the copy back, after the frame copying it is submitted.
  pub(crate) fn request(&self) {
    self.readback.request();

    if let Some((readback, _)) = &self.coverage {
      readback.request();
    }
  }

  /// The capture once it is back, as eight bit RGBA rows top to bottom, a map tile's alpha its coverage; nothing while
  /// it, or a map tile's depth, is on its way.
  pub(crate) fn take(&self) -> Option<XrfResult<RenderCapture>> {
    match &self.coverage {
      Some((readback, row)) if self.readback.is_back() && readback.is_back() => {
        let capture: XrfResult<RenderCapture> = self.take_picture()?;
        let depths: XrfResult<Vec<bool>> = readback.take(|bytes| Self::read_covered(bytes, *row, self.get_size()))?;

        Some(capture.and_then(|mut capture| {
          for (texel, is_covered) in capture.pixels.as_chunks_mut::<4>().0.iter_mut().zip(depths?) {
            texel[3] = if is_covered { 255 } else { 0 };
          }

          Ok(capture)
        }))
      }
      Some(_) => None,
      None => self.take_picture(),
    }
  }

  /// Whether each texel of a depth copy, rows `row` bytes apart, holds anything nearer than the far plane: reversed,
  /// the far plane is zero.
  fn read_covered(bytes: &[u8], row: u32, (width, height): (u32, u32)) -> Vec<bool> {
    bytes
      .chunks_exact(row as usize)
      .take(height as usize)
      .flat_map(|line| {
        line[..(width * 4) as usize]
          .as_chunks::<4>()
          .0
          .iter()
          .map(|texel| f32::from_le_bytes(*texel) > 0.0)
      })
      .collect()
  }

  /// The picture once it is back, opaque; nothing while it is on its way.
  fn take_picture(&self) -> Option<XrfResult<RenderCapture>> {
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
