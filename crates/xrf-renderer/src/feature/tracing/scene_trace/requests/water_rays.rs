/// The rays a water draw leaves the scene pass while every pixel traces ("Trace all water"), in place of a frame-sized
/// request list.
///
/// One `WORDS`-word record per drawn pixel (`common/water_rays`), a third of a request's bytes; the start is read back
/// from the water's depth. Never cleared: a record another frame wrote is skipped by its frame stamp. One empty record
/// while the water does not trace every pixel.
pub(crate) struct WaterRays {
  label: String,
  pixels: u32,
  pub records: wgpu::Buffer,
}

impl WaterRays {
  /// Words a pixel's record takes.
  pub(crate) const WORDS: u32 = 4;
  /// The side of the square of pixels a workgroup of the scene pass reads.
  pub(crate) const WORKGROUP: u32 = 8;

  pub(crate) fn new(device: &wgpu::Device, label: &str, pixels: u32) -> Self {
    Self {
      label: label.to_string(),
      pixels,
      records: Self::create_records(device, label, pixels),
    }
  }

  /// Remakes the records for another number of `pixels`: the drawn size's while every pixel traces, none otherwise.
  pub(crate) fn fit(&mut self, device: &wgpu::Device, pixels: u32) {
    if pixels != self.pixels {
      self.pixels = pixels;
      self.records = Self::create_records(device, &self.label, pixels);
    }
  }

  fn create_records(device: &wgpu::Device, label: &str, pixels: u32) -> wgpu::Buffer {
    device.create_buffer(&wgpu::BufferDescriptor {
      label: Some(&format!("{label} rays")),
      size: u64::from(pixels.max(1)) * u64::from(Self::WORDS) * size_of::<u32>() as u64,
      usage: wgpu::BufferUsages::STORAGE,
      mapped_at_creation: false,
    })
  }
}
