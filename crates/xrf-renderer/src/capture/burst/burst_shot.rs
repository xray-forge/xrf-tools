use xrf_error::XrfResult;

use crate::capture::burst::burst_event::BurstEvent;
use crate::capture::burst::burst_marks::BurstMarks;
use crate::contract::render_capture::RenderCapture;
use crate::frame::frame_capture::FrameCapture;
use crate::frame::gpu_readback::GpuReadback;

/// A burst's frame on its way back: the finished frame as a screenshot draws it, and the G-buffer's material target
/// beside it, whose alpha holds the marks.
pub struct BurstShot {
  /// Its place among the frames read back, its sample where a reference draws it, and the scene's seconds it is drawn
  /// at.
  frame: (u32, Option<u32>, f32),
  picture: FrameCapture,
  material: GpuReadback,
  /// The material target's texels across and down, and the bytes a row of its copy takes.
  material_size: (u32, u32),
  material_row: u32,
}

impl BurstShot {
  /// A frame `size` pixels across, drawn in `format`, the viewport's frame `drawn`, and its material target
  /// `material_size` texels across, waiting for their copies; `frame` says which of the burst's frames it is.
  ///
  /// # Errors
  ///
  /// Returns an error for a frame of a format a capture cannot read.
  pub fn new(
    device: &wgpu::Device,
    (format, size, drawn): (wgpu::TextureFormat, (u32, u32), u64),
    material_size: (u32, u32),
    frame: (u32, Option<u32>, f32),
  ) -> XrfResult<Self> {
    // The material target is four bytes a texel, as the frame is.
    let material_row: u32 = (material_size.0 * 4).next_multiple_of(wgpu::COPY_BYTES_PER_ROW_ALIGNMENT);

    Ok(Self {
      frame,
      picture: FrameCapture::new_screenshot(device, format, size, drawn)?,
      material: GpuReadback::new(device, "burst material", (material_row * material_size.1) as u64),
      material_size,
      material_row,
    })
  }

  /// The finished frame's copy, which the present draws into.
  pub fn get_picture(&self) -> &FrameCapture {
    &self.picture
  }

  /// Copies the marks out of `material`, the frame's material target, beside the picture.
  pub fn encode_marks(&self, encoder: &mut wgpu::CommandEncoder, material: &wgpu::Texture) {
    let (width, height): (u32, u32) = self.material_size;

    encoder.copy_texture_to_buffer(
      wgpu::TexelCopyTextureInfo {
        texture: material,
        mip_level: 0,
        origin: wgpu::Origin3d::ZERO,
        aspect: wgpu::TextureAspect::All,
      },
      wgpu::TexelCopyBufferInfo {
        buffer: self.material.get_buffer(),
        layout: wgpu::TexelCopyBufferLayout {
          offset: 0,
          bytes_per_row: Some(self.material_row),
          rows_per_image: Some(height),
        },
      },
      wgpu::Extent3d {
        width,
        height,
        depth_or_array_layers: 1,
      },
    );
    self.material.mark_recorded();
  }

  /// Whether both copies were recorded with the frame, which a frame that drew nothing of it did not.
  pub fn is_recorded(&self) -> bool {
    self.picture.is_recorded() && self.material.is_recorded()
  }

  /// Asks for both copies back, after the frame copying them is submitted.
  pub fn request(&self) {
    self.picture.request();
    self.material.request();
  }

  /// The frame once both copies are back, as its burst tells it; nothing while either is on its way.
  pub fn take(&self) -> Option<XrfResult<BurstEvent>> {
    if !self.material.is_back() {
      return None;
    }

    let picture: XrfResult<RenderCapture> = self.picture.take()?;
    let marks: XrfResult<BurstMarks> = self.material.take(|bytes| self.read_marks(bytes))?;
    let (index, sample, time) = self.frame;

    Some(picture.and_then(|picture| {
      Ok(BurstEvent::Shot {
        index,
        sample,
        time,
        picture,
        marks: marks?,
      })
    }))
  }

  /// Each texel's alpha of the material target's copy, its rows `material_row` bytes apart.
  fn read_marks(&self, bytes: &[u8]) -> BurstMarks {
    let (width, height): (u32, u32) = self.material_size;

    BurstMarks {
      width,
      height,
      marks: bytes
        .chunks_exact(self.material_row as usize)
        .take(height as usize)
        .flat_map(|line| {
          line[..(width * 4) as usize]
            .as_chunks::<4>()
            .0
            .iter()
            .map(|texel| texel[3])
        })
        .collect(),
    }
  }
}
