use std::time::{Duration, Instant};

use xrf_renderer_core::{
  FrameGraph, GraphBindings, GraphBuffer, GraphBufferAccess, GraphRuntime, GraphTexture, StorageArrayMut,
  StorageValueMut,
};

use crate::contract::render_exposure_settings::RenderExposureSettings;
use crate::frame::gpu_readback::GpuReadback;
use crate::pass::exposure_head::ExposureHead;
use crate::pass::exposure_histogram_parameters::ExposureHistogramParameters;
use crate::pass::exposure_histogram_uniform::ExposureHistogramUniform;
use crate::pass::exposure_parameters::ExposureParameters;
use crate::pass::exposure_state::ExposureState;
use crate::pass::exposure_uniform::ExposureUniform;
use crate::pass::level_passes::LevelPasses;
use crate::pass::pre_exposure::PreExposure;
use crate::scene::level::feature::view_feature::ViewFeature;

/// The longest step between frames the adaptation takes, so a stall does not snap it to what one frame measured.
const LONGEST_STEP: Duration = Duration::from_millis(250);

/// One viewport's exposure: the scale its tonemap multiplies by, adapted on the GPU, the cells it was measured from,
/// and the adaptation's rate, which the CPU advances by the time between frames; under the physical light, a histogram
/// of the scene's light in place of the cells, or a fixed scale where it does not adapt, the scale read back as the one
/// the next frames write their light at.
pub struct ExposureFeature {
  /// The adapted scale, then each cell's luminance.
  pub state: wgpu::Buffer,
  /// The physical light's histogram, emptied by each frame's settling.
  bins: wgpu::Buffer,
  /// How it adapts this frame.
  pub params: ExposureUniform,
  /// How the physical light meters this frame.
  histogram: ExposureHistogramUniform,
  /// `f_luminance_adapt`, how far this frame moves the scale.
  rate: f32,
  last: Option<Instant>,
  /// Whether the passes read the state's scale, rather than one.
  is_adapting: bool,
  /// Whether the frame is metered and the scale moved this frame.
  is_measuring: bool,
  /// Whether the light is physical, which the histogram meters.
  is_physical: bool,
  /// The scale the physical light holds while it does not adapt, as last written.
  held: Option<f32>,
  /// The state's head copied out without waiting, whose scale the physical light's next frames write their light at.
  readback: GpuReadback,
  /// The scale this frame's light is written at, and the last frame's.
  pre_exposure: PreExposure,
}

impl ExposureFeature {
  /// Cells a side the frame's luminance is measured over, as `rt_LUM_64` is.
  pub const CELLS: u32 = 64;

  pub fn new(device: &wgpu::Device, queue: &wgpu::Queue) -> Self {
    let state: wgpu::Buffer = device.create_buffer(&wgpu::BufferDescriptor {
      label: Some("exposure"),
      size: size_of::<ExposureState>() as u64,
      usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_DST | wgpu::BufferUsages::COPY_SRC,
      mapped_at_creation: false,
    });
    let bins: wgpu::Buffer = device.create_buffer(&wgpu::BufferDescriptor {
      label: Some("exposure histogram"),
      size: u64::from(ExposureHistogramUniform::BINS) * size_of::<u32>() as u64,
      usage: wgpu::BufferUsages::STORAGE,
      mapped_at_creation: false,
    });
    let exposure: Self = Self {
      state,
      bins,
      params: ExposureUniform::default(),
      histogram: ExposureHistogramUniform::default(),
      rate: 0.5,
      last: None,
      is_adapting: false,
      is_measuring: false,
      is_physical: false,
      held: None,
      readback: GpuReadback::new(device, "exposure readback", size_of::<ExposureHead>() as u64),
      pre_exposure: PreExposure::NONE,
    };

    exposure.reset(queue, 1.0);

    exposure
  }

  /// Advances the rate by the time since the last frame and writes what the adaptation reads; off, the scale goes back
  /// to one, the engine's answer at noon. Under the physical light (`is_physical`) the histogram meters instead, or,
  /// off, the scale holds at the compensation's stops; a change of light model starts the scale afresh. Then the scale
  /// the frame's light is written at: the physical light's last read back, one under the engine's.
  pub fn prepare(&mut self, queue: &wgpu::Queue, settings: &RenderExposureSettings, is_physical: bool, now: Instant) {
    let previous: f32 = self.pre_exposure.current;
    let read: Option<f32> = self.take_readback();

    if is_physical != self.is_physical {
      self.is_physical = is_physical;
      self.last = None;
      self.rate = 0.5;
      self.held = None;
      self.reset(queue, 1.0);
      self.pre_exposure = PreExposure::NONE;
    }

    if is_physical {
      self.prepare_physical(queue, settings, now);
      self.pre_exposure = PreExposure {
        current: self.held.or(read).unwrap_or(self.pre_exposure.current),
        previous,
      };

      return;
    }

    self.pre_exposure = PreExposure::NONE;

    if !settings.is_enabled {
      if self.is_adapting {
        self.is_adapting = false;
        self.last = None;
        self.rate = 0.5;
        self.reset(queue, 1.0);
      }

      self.is_measuring = false;

      return;
    }

    let delta: Duration = self.last.map_or(Duration::ZERO, |last| (now - last).min(LONGEST_STEP));

    self.is_adapting = true;
    self.is_measuring = true;
    self.last = Some(now);
    // `f_luminance_adapt = .9 * f + .1 * dt * adaptation`; the scale moves by it at most all the way, since past one
    // the move overshoots and a slow frame under a fast adaptation would swing the scale about its target.
    self.rate = 0.9 * self.rate + 0.1 * delta.as_secs_f32() * settings.adaptation;
    self.params = ExposureUniform::new(settings, self.rate.min(1.0));
  }

  /// The physical light's metering for this frame: the histogram from the time since the last frame, straight to its
  /// target on the first; or, off, the compensation's scale held.
  fn prepare_physical(&mut self, queue: &wgpu::Queue, settings: &RenderExposureSettings, now: Instant) {
    self.is_adapting = true;

    if !settings.is_enabled {
      let held: f32 = settings.compensation.exp2();

      self.is_measuring = false;
      self.last = None;

      if self.held != Some(held) {
        self.held = Some(held);
        self.reset(queue, held);
      }

      return;
    }

    let delta: Duration = self.last.map_or(Duration::ZERO, |last| (now - last).min(LONGEST_STEP));

    self.histogram = ExposureHistogramUniform::new(settings, delta.as_secs_f32(), self.last.is_none());
    self.is_measuring = true;
    self.last = Some(now);
    self.held = None;
  }

  /// Whether the passes read the adapted scale, rather than one.
  pub fn is_adapting(&self) -> bool {
    self.is_adapting
  }

  /// The scale this frame's light is written at, and the last frame's.
  pub fn get_pre_exposure(&self) -> PreExposure {
    self.pre_exposure
  }

  /// Asks for the scale copied out with the frame just submitted.
  pub fn request_readback(&self) {
    self.readback.request();
  }

  /// Drops the scale a frame that was never submitted copied out.
  pub fn abandon_readback(&self) {
    self.readback.abandon();
  }

  /// The scale a copy brought back since the last frame, unless it is not a scale. Any scale writes the light right,
  /// since the passes divide it out again (`frame_scale`); the latest keeps the light near the range it is shown in.
  fn take_readback(&self) -> Option<f32> {
    let adapted: f32 = self
      .readback
      .take(|bytes| bytemuck::pod_read_unaligned::<ExposureHead>(&bytes[..size_of::<ExposureHead>()]).adapted)?
      .ok()?;

    (adapted.is_finite() && adapted > 0.0).then_some(adapted)
  }

  /// Adds the adaptation into `state`, the imported `state` buffer: the frame's luminance measured from the high target
  /// and the scale moved towards it, or under the physical light the `size` scene's light counted into the histogram and
  /// the scale settled from it; nothing while it does not measure.
  pub fn add_passes<'a>(
    &'a self,
    (graph, bindings, runtime): (&mut FrameGraph<'a>, &mut GraphBindings<'a>, &mut GraphRuntime),
    passes: LevelPasses<'a>,
    (high, scene, size, state): (GraphTexture, GraphTexture, (u32, u32), GraphBuffer),
  ) {
    if !self.is_measuring {
      return;
    }

    if self.is_physical {
      let bins: GraphBuffer = bindings.import_buffer(&mut *graph, "exposure histogram", &self.bins);
      let parameters: ExposureHistogramParameters = ExposureHistogramParameters {
        scene,
        state: StorageValueMut::new(state),
        bins: StorageArrayMut::new(bins),
        params: runtime.push_uniform(&self.histogram),
      };

      graph
        .add_compute_pass("exposure histogram")
        .parameters(&parameters)
        .record(move |context| passes.exposure.record_histogram(context, &parameters, size));
      graph
        .add_encoder_pass("exposure readback")
        .buffer(state, GraphBufferAccess::CopySource)
        .keep()
        .record(move |context| {
          if self.readback.is_free() {
            let source: &wgpu::Buffer = context.get_buffer(state);

            context.get_encoder().copy_buffer_to_buffer(
              source,
              0,
              self.readback.get_buffer(),
              0,
              size_of::<ExposureHead>() as u64,
            );
            self.readback.mark_recorded();
          }
        });

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

  /// Writes the state's scale and the scale the last frame used as `scale`.
  fn reset(&self, queue: &wgpu::Queue, scale: f32) {
    let state: ExposureState = ExposureState {
      adapted: scale,
      used: scale,
      ..bytemuck::Zeroable::zeroed()
    };

    queue.write_buffer(&self.state, 0, bytemuck::bytes_of(&state));
  }
}

impl ViewFeature for ExposureFeature {
  /// Its state, the histogram's bins and the readback, which every lit pass reads its scale from, one while it is off;
  /// so they are kept whatever the settings.
  fn get_memory(&self) -> u64 {
    self.state.size() + self.bins.size() + self.readback.get_buffer().size()
  }

  /// Nothing to let go of on its own: its state goes with it.
  fn release(&mut self) {}
}
