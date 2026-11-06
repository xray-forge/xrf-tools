/// The rays a water draw leaves the scene pass while every pixel traces every frame ("Trace all water"), in place of a
/// request list as large as the frame: a record of `WORDS` words at each drawn pixel's place (`common/water_rays`),
/// the ray's direction, what the screen answered and its distance and the frame it was written on, the ray's start
/// read back from the water's own depth. A third of a request's bytes, never cleared: a record another frame wrote is
/// passed over by its stamp. A record of nothing while the water does not trace every pixel.
pub struct WaterRays {
  label: String,
  pixels: u32,
  pub records: wgpu::Buffer,
}

impl WaterRays {
  /// Words a pixel's record takes.
  pub const WORDS: u32 = 4;
  /// The side of the square of pixels a workgroup of the scene pass reads.
  pub const WORKGROUP: u32 = 8;

  pub fn new(device: &wgpu::Device, label: &str, pixels: u32) -> Self {
    Self {
      label: label.to_string(),
      pixels,
      records: Self::create_records(device, label, pixels),
    }
  }

  /// Makes the records anew where they hold another number of pixels: the drawn size's while every pixel traces, none
  /// otherwise.
  pub fn fit(&mut self, device: &wgpu::Device, pixels: u32) {
    if pixels != self.pixels {
      self.pixels = pixels;
      self.records = Self::create_records(device, &self.label, pixels);
    }
  }

  /// Bytes its records take.
  pub fn get_memory(&self) -> u64 {
    self.records.size()
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
