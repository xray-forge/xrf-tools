use std::time::{Duration, Instant};

use crate::contract::render_exposure_settings::RenderExposureSettings;
use crate::pass::exposure_uniform::ExposureUniform;

/// Cells a side the frame's luminance is measured over, as `rt_LUM_64` is.
pub const EXPOSURE_CELLS: u32 = 64;

/// The longest step between frames the adaptation takes, so a stall does not snap it to what one frame measured.
const LONGEST_STEP: Duration = Duration::from_millis(250);

/// One viewport's exposure: the scale its tonemap multiplies by, adapted on the GPU, the cells it was measured from,
/// and the adaptation's rate, which the CPU advances by the time between frames.
pub struct ViewExposure {
  /// The adapted scale, then each cell's luminance.
  pub state: wgpu::Buffer,
  pub params: wgpu::Buffer,
  /// `f_luminance_adapt`, how far this frame moves the scale.
  rate: f32,
  last: Option<Instant>,
  is_adapting: bool,
}

impl ViewExposure {
  pub fn new(device: &wgpu::Device, queue: &wgpu::Queue) -> Self {
    let state: wgpu::Buffer = device.create_buffer(&wgpu::BufferDescriptor {
      label: Some("exposure"),
      size: 16 + (EXPOSURE_CELLS * EXPOSURE_CELLS * 4) as u64,
      usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_DST,
      mapped_at_creation: false,
    });
    let params: wgpu::Buffer = device.create_buffer(&wgpu::BufferDescriptor {
      label: Some("exposure parameters"),
      size: size_of::<ExposureUniform>() as u64,
      usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
      mapped_at_creation: false,
    });
    let exposure: Self = Self {
      state,
      params,
      rate: 0.5,
      last: None,
      is_adapting: false,
    };

    exposure.reset(queue);

    exposure
  }

  /// Advances the rate by the time since the last frame and writes what the adaptation reads; off, the scale goes back
  /// to one, the engine's answer at noon.
  pub fn prepare(&mut self, queue: &wgpu::Queue, settings: &RenderExposureSettings, now: Instant) {
    if !settings.is_enabled {
      if self.is_adapting {
        self.is_adapting = false;
        self.last = None;
        self.rate = 0.5;
        self.reset(queue);
      }

      return;
    }

    let delta: Duration = self.last.map_or(Duration::ZERO, |last| (now - last).min(LONGEST_STEP));

    self.is_adapting = true;
    self.last = Some(now);
    // `f_luminance_adapt = .9 * f + .1 * dt * adaptation`.
    self.rate = 0.9 * self.rate + 0.1 * delta.as_secs_f32() * settings.adaptation;
    queue.write_buffer(
      &self.params,
      0,
      bytemuck::bytes_of(&ExposureUniform::new(settings, self.rate)),
    );
  }

  pub fn is_adapting(&self) -> bool {
    self.is_adapting
  }

  fn reset(&self, queue: &wgpu::Queue) {
    queue.write_buffer(&self.state, 0, bytemuck::bytes_of(&[1.0f32, 0.0, 0.0, 0.0]));
  }
}
