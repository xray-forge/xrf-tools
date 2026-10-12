use std::time::{Duration, Instant};

use xrf_renderer_core::{FrameGraph, GraphBuffer, GraphRuntime, GraphTexture, StorageValueMut};

use crate::contract::render_exposure_settings::RenderExposureSettings;
use crate::pass::exposure_parameters::ExposureParameters;
use crate::pass::exposure_state::ExposureState;
use crate::pass::exposure_uniform::ExposureUniform;
use crate::pass::level_passes::LevelPasses;
use crate::scene::level::feature::view_feature::ViewFeature;

/// Cells a side the frame's luminance is measured over, as `rt_LUM_64` is.
pub const EXPOSURE_CELLS: u32 = 64;

/// The longest step between frames the adaptation takes, so a stall does not snap it to what one frame measured.
const LONGEST_STEP: Duration = Duration::from_millis(250);

/// One viewport's exposure: the scale its tonemap multiplies by, adapted on the GPU, the cells it was measured from,
/// and the adaptation's rate, which the CPU advances by the time between frames.
pub struct ExposureFeature {
  /// The adapted scale, then each cell's luminance.
  pub state: wgpu::Buffer,
  /// How it adapts this frame.
  pub params: ExposureUniform,
  /// `f_luminance_adapt`, how far this frame moves the scale.
  rate: f32,
  last: Option<Instant>,
  is_adapting: bool,
}

impl ExposureFeature {
  pub fn new(device: &wgpu::Device, queue: &wgpu::Queue) -> Self {
    let state: wgpu::Buffer = device.create_buffer(&wgpu::BufferDescriptor {
      label: Some("exposure"),
      size: size_of::<ExposureState>() as u64,
      usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_DST,
      mapped_at_creation: false,
    });
    let exposure: Self = Self {
      state,
      params: ExposureUniform::default(),
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
    // `f_luminance_adapt = .9 * f + .1 * dt * adaptation`; the scale moves by it at most all the way, since past one
    // the move overshoots and a slow frame under a fast adaptation would swing the scale about its target.
    self.rate = 0.9 * self.rate + 0.1 * delta.as_secs_f32() * settings.adaptation;
    self.params = ExposureUniform::new(settings, self.rate.min(1.0));
  }

  pub fn is_adapting(&self) -> bool {
    self.is_adapting
  }

  /// Adds the adaptation: the frame's luminance measured from the high target and the scale moved towards it, into
  /// `state`, the imported `state` buffer; nothing while it does not adapt.
  pub fn add_passes<'a>(
    &self,
    (graph, runtime): (&mut FrameGraph<'a>, &mut GraphRuntime),
    passes: LevelPasses<'a>,
    (high, state): (GraphTexture, GraphBuffer),
  ) {
    if !self.is_adapting {
      return;
    }

    let parameters: ExposureParameters = ExposureParameters {
      high,
      state: StorageValueMut::new(state),
      params: runtime.push_uniform(&self.params),
    };

    graph
      .add_compute_pass("exposure")
      .parameters(&parameters)
      .record(move |context| passes.exposure.record(context, &parameters));
  }

  fn reset(&self, queue: &wgpu::Queue) {
    queue.write_buffer(&self.state, 0, bytemuck::bytes_of(&[1.0f32, 1.0, 0.0, 0.0]));
  }
}

impl ViewFeature for ExposureFeature {
  /// Its state, which every lit pass reads its scale from, one while it is off; so it is kept whatever the settings.
  fn get_memory(&self) -> u64 {
    self.state.size()
  }

  /// Nothing to let go of on its own: its state goes with it.
  fn release(&mut self) {}
}
