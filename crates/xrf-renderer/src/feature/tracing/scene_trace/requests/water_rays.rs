use wgpu::util::DeviceExt;

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
  /// The screen's and the trace's summed luminance where they meet, two frames' slots of two words, and the ratio
  /// the trace's light is brought to the screen's by (`frame/reflection_scene`), one at first.
  pub calibration: wgpu::Buffer,
}

impl WaterRays {
  /// Words a pixel's record takes.
  pub(crate) const WORDS: u32 = 4;
  /// The side of the square of pixels a workgroup of the scene pass reads.
  pub(crate) const WORKGROUP: u32 = 8;
  /// The calibration's word holding its ratio (`WATER_CALIBRATION_RATIO`), past the two frames' sums.
  pub(crate) const CALIBRATION_RATIO: usize = 4;

  pub(crate) fn new(device: &wgpu::Device, label: &str, pixels: u32) -> Self {
    Self {
      label: label.to_string(),
      pixels,
      records: Self::create_records(device, label, pixels),
      calibration: Self::create_calibration(device, label),
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

  /// The calibration's words (`frame/reflection_scene`): two frames' sums, empty, then the ratio, one, padded to eight.
  fn create_calibration(device: &wgpu::Device, label: &str) -> wgpu::Buffer {
    let mut words: [u32; 8] = [0; 8];

    words[Self::CALIBRATION_RATIO] = 1.0_f32.to_bits();

    device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
      label: Some(&format!("{label} calibration")),
      contents: bytemuck::cast_slice(&words),
      usage: wgpu::BufferUsages::STORAGE,
    })
  }
}
