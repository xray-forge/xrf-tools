use std::time::{Duration, Instant};

use xrf_error::XrfResult;
use xrf_renderer_core::{
  FrameGraph, GraphBindings, GraphBuffer, GraphBufferAccess, GraphRuntime, GraphTexture, StorageArrayMut,
  StorageValueMut,
};

use crate::contract::render_exposure_settings::RenderExposureSettings;
use crate::contract::render_light_level::RenderLightLevel;
use crate::contract::render_view_options::RenderViewOptions;
use crate::frame::gpu_readback::GpuReadback;
use crate::pass::exposure_head::ExposureHead;
use crate::pass::exposure_histogram_parameters::ExposureHistogramParameters;
use crate::pass::exposure_histogram_uniform::ExposureHistogramUniform;
use crate::pass::exposure_metering::ExposureMetering;
use crate::pass::exposure_parameters::ExposureParameters;
use crate::pass::exposure_state::ExposureState;
use crate::pass::exposure_uniform::ExposureUniform;
use crate::pass::level_passes::LevelPasses;
use crate::pass::pre_exposure::PreExposure;
use crate::scene::level::feature::view_feature::ViewFeature;
use crate::scene::level::lighting_handles::LightingHandles;

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
  /// Whether the copy on its way was made before the light model changed, so what it brings is dropped.
  is_readback_stale: bool,
  /// The scale this frame's light is written at, and the last frame's.
  pre_exposure: PreExposure,
  /// What the physical light's histogram last metered: the key it is shown at, the histogram's or a daylit exterior's
  /// at the scale it starts at or holds, and the land's and the sky's levels, none while it holds.
  metering: ExposureMetering,
  /// Whether the scale holds where it stands, metered no more: a capture's tiles or faces share one exposure, or
  /// their seams show.
  is_frozen: bool,
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
      size: u64::from(ExposureHistogramUniform::BINS + ExposureHistogramUniform::SKY_SLOTS) * size_of::<u32>() as u64,
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
      is_readback_stale: false,
      pre_exposure: PreExposure::NONE,
      metering: ExposureMetering::default(),
      is_frozen: false,
    };

    exposure.reset(queue, 1.0);

    exposure
  }

  /// Advances the rate by the time since the last frame and writes what the adaptation reads; off, the scale goes back
  /// to one, the engine's answer at noon. Under the physical light (`is_physical`) the histogram meters instead, or,
  /// off, the scale holds at a daylit exterior's; a change of light model starts the scale afresh, dropping what the
  /// other model metered. Then the scale the frame's light is written at: the physical light's last read back, one
  /// under the engine's.
  pub fn prepare(&mut self, queue: &wgpu::Queue, options: &RenderViewOptions, now: Instant) {
    let settings: &RenderExposureSettings = &options.features.exposure;
    let is_physical: bool = options.is_light_physical();
    let is_switched: bool = is_physical != self.is_physical;
    let read: Option<ExposureHead> = self.take_readback();
    let scale: Option<f32> = read
      .map(|head| head.adapted)
      .filter(|adapted| adapted.is_finite() && *adapted > 0.0);

    if self.is_frozen && !is_switched {
      // The scale the passes read stays in the state unmeasured, and the light is written at the last frame's.
      self.is_measuring = false;
      self.pre_exposure.previous = self.pre_exposure.current;

      return;
    }

    if is_switched {
      // The physical light starts where a daylit exterior is shown, its light written there from the first frame; a
      // copy still on its way measured the other model's scale.
      let start: f32 = if is_physical {
        ExposureHistogramUniform::get_daylight_scale(settings.compensation)
      } else {
        1.0
      };

      self.is_physical = is_physical;
      self.is_readback_stale = !self.readback.is_free();
      self.last = None;
      self.rate = 0.5;
      self.held = None;
      self.reset(queue, start);
      self.pre_exposure = PreExposure {
        current: start,
        previous: start,
      };
      self.metering = ExposureMetering {
        key: ExposureHistogramUniform::get_daylit_key(start),
        ..ExposureMetering::default()
      };
    }

    if is_physical {
      let previous: f32 = self.pre_exposure.current;

      self.prepare_physical(queue, settings, options.features.light_level, now);
      self.pre_exposure = PreExposure {
        current: self.held.or(scale).unwrap_or(self.pre_exposure.current),
        previous,
      };

      self.metering = match (self.held, read) {
        (Some(held), _) => ExposureMetering {
          key: ExposureHistogramUniform::get_daylit_key(held),
          ..ExposureMetering::default()
        },
        (None, Some(head)) => ExposureMetering {
          key: Some(head.key.exp2())
            .filter(|key| key.is_finite() && *key > 0.0)
            .unwrap_or(self.metering.key),
          bright: Some(head.bright.exp2())
            .filter(|bright| bright.is_finite() && *bright > 0.0)
            .or(self.metering.bright),
          land: Some(head.key - head.adapted.log2())
            .filter(|land| land.is_finite())
            .or(self.metering.land),
          sky: Some(head.sky).filter(|sky| sky.is_finite()).or(self.metering.sky),
          sky_bright: Some(head.sky_bright)
            .filter(|bright| bright.is_finite())
            .or(self.metering.sky_bright),
        },
        (None, None) => self.metering,
      };

      return;
    }

    self.pre_exposure = PreExposure::NONE;
    self.metering = ExposureMetering::default();

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

    let delta: Duration = self.get_delta(now);

    self.is_adapting = true;
    self.is_measuring = true;
    self.last = Some(now);
    // `f_luminance_adapt = .9 * f + .1 * dt * adaptation`; the scale moves by it at most all the way, since past one
    // the move overshoots and a slow frame under a fast adaptation would swing the scale about its target.
    self.rate = 0.9 * self.rate + 0.1 * delta.as_secs_f32() * settings.adaptation;
    self.params = ExposureUniform::new(settings, self.rate.min(1.0));
  }

  /// The physical light's metering for this frame: the histogram from the time since the last frame, straight to its
  /// target on the first; or, off, a daylit exterior's scale held, raised by the compensation.
  fn prepare_physical(
    &mut self,
    queue: &wgpu::Queue,
    settings: &RenderExposureSettings,
    level: RenderLightLevel,
    now: Instant,
  ) {
    self.is_adapting = true;

    if !settings.is_enabled {
      let held: f32 = ExposureHistogramUniform::get_daylight_scale(settings.compensation);

      self.is_measuring = false;
      self.last = None;

      if self.held != Some(held) {
        self.held = Some(held);
        self.reset(queue, held);
      }

      return;
    }

    let delta: Duration = self.get_delta(now);

    self.histogram = ExposureHistogramUniform::new(settings, level, delta.as_secs_f32(), self.last.is_none());
    self.is_measuring = true;
    self.last = Some(now);
    self.held = None;
  }

  /// Holds the scale where it stands, or lets it adapt again.
  pub fn set_frozen(&mut self, is_frozen: bool) {
    self.is_frozen = is_frozen;
    self.last = None;
  }

  /// The time this frame advances the adaptation by: the time since the last frame, at most its longest step, none
  /// on the first.
  fn get_delta(&self, now: Instant) -> Duration {
    self.last.map_or(Duration::ZERO, |last| (now - last).min(LONGEST_STEP))
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

  /// What the physical light's histogram last metered, which the game's pictures are shown by (`PictureShares`).
  pub fn get_metering(&self) -> ExposureMetering {
    self.metering
  }

  /// The head a copy brought back since the last frame, none from a copy made before the light model changed. Any
  /// scale it holds writes the light right, since the passes divide it out again (`frame_scale`); the latest keeps the
  /// light near the range it is shown in.
  fn take_readback(&mut self) -> Option<ExposureHead> {
    let head: XrfResult<ExposureHead> = self
      .readback
      .take(|bytes| bytemuck::pod_read_unaligned::<ExposureHead>(&bytes[..size_of::<ExposureHead>()]))?;

    if std::mem::take(&mut self.is_readback_stale) {
      return None;
    }

    head.ok()
  }

  /// Adds the adaptation into the exposure's state the lit passes share (`lit`): the frame's luminance measured from the
  /// high target and the scale moved towards it, or under the physical light the `size` scene's light counted into the
  /// histogram and the scale settled from it, the sky's level read through the frame's lighting; nothing while it does
  /// not measure.
  pub fn add_passes<'a>(
    &'a self,
    (graph, bindings, runtime): (&mut FrameGraph<'a>, &mut GraphBindings<'a>, &mut GraphRuntime),
    passes: LevelPasses<'a>,
    (high, (scene, depth_target), size, lit): (
      GraphTexture,
      (GraphTexture, GraphTexture),
      (u32, u32),
      &LightingHandles<'a>,
    ),
  ) {
    if !self.is_measuring {
      return;
    }

    let state: GraphBuffer = lit.exposure;

    if self.is_physical {
      let bins: GraphBuffer = bindings.import_buffer(&mut *graph, "exposure histogram", &self.bins);
      let parameters: ExposureHistogramParameters = ExposureHistogramParameters {
        scene,
        depth_target,
        state: StorageValueMut::new(state),
        bins: StorageArrayMut::new(bins),
        params: runtime.push_uniform(&self.histogram),
        lighting: lit.lighting,
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
          let source: &wgpu::Buffer = context.get_buffer(state);

          self.record_readback(context.get_encoder(), source);
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

  /// Copies the head of the exposure's state (`source`) out with the frame `encoder` records, unless a copy is still on
  /// its way.
  pub(crate) fn record_readback(&self, encoder: &mut wgpu::CommandEncoder, source: &wgpu::Buffer) {
    if self.readback.is_free() {
      encoder.copy_buffer_to_buffer(
        source,
        0,
        self.readback.get_buffer(),
        0,
        size_of::<ExposureHead>() as u64,
      );
      self.readback.mark_recorded();
    }
  }

  /// Writes the state's scale and the scale the last frame used as `scale`.
  fn reset(&self, queue: &wgpu::Queue, scale: f32) {
    let state: ExposureState = ExposureState {
      adapted: scale,
      used: scale,
      key: ExposureHistogramUniform::get_daylit_key(scale).log2(),
      sky: f32::NAN,
      bright: f32::NAN,
      sky_bright: f32::NAN,
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
