use std::time::{Duration, Instant};

use glam::{Mat4, Vec2, Vec3};
use xrf_dlss::{Dlss, DlssInputs, DlssPreset, DlssQuality, DlssRayInputs, DlssRayPreset};
use xrf_renderer_core::{
  FrameGraph, GraphColorAttachment, GraphDeclaration, GraphTexture, GraphTextureAccess, GraphTextureDescriptor,
  UniformBinding,
};

use crate::camera::camera_view::CameraView;
use crate::contract::render_antialiasing::RenderAntialiasing;
use crate::contract::render_debug_view::RenderDebugView;
use crate::contract::render_rect::RenderRect;
use crate::contract::render_upscaling_settings::RenderUpscalingSettings;
use crate::contract::render_view_options::RenderViewOptions;
use crate::frame::frame_texture::FrameTexture;
use crate::frame::history::History;
use crate::frame::texture_copy::add_texture_copy;
use crate::frame::view_target_handles::ViewTargetHandles;
use crate::frame::view_targets::ViewTargets;
use crate::pass::fsr_uniform::FsrUniform;
use crate::pass::level_passes::LevelPasses;
use crate::pass::lighting_uniform::LightingUniform;
use crate::pass::pre_exposure::PreExposure;
use crate::pass::ray_guides_parameters::RayGuidesParameters;
use crate::pass::ray_guides_pass::RayGuidesPass;
use crate::pass::temporal_parameters::TemporalParameters;
use crate::pass::temporal_pass::TemporalPass;
use crate::pass::temporal_uniform::TemporalUniform;
use crate::pass::upscale_parameters::UpscaleParameters;
use crate::pass::upscale_pass::UpscalePass;
use crate::pass::upscale_stage::UpscaleStage;
use crate::pass::upscale_uniform::UpscaleUniform;
use crate::pass::view_binding::ViewBinding;
use crate::scene::level::feature::dlss_resolve::DlssResolve;
use crate::scene::level::feature::fsr_targets::FsrTargets;
use crate::scene::level::feature::ray_guides::RayGuides;
use crate::scene::level::feature::ray_reconstruction_resolve::RayReconstructionResolve;
use crate::scene::level::feature::resolve::Resolve;
use crate::scene::level::feature::temporal_jitter::TemporalJitter;
use crate::scene::level::feature::temporal_resolve::TemporalResolve;
use crate::scene::level::feature::view_feature::ViewFeature;
use crate::scene::level::feature::view_frame_end::ViewFrameEnd;

/// What a DLSS upscaler is made for: its output, quality and preset, and whether the colour is linear light.
type DlssKey = ((u32, u32), (DlssQuality, DlssPreset), bool);

/// What a DLSS ray reconstructor is made for: its output and quality.
type RayKey = ((u32, u32), DlssQuality, DlssRayPreset);

/// How a view's frame becomes the image its viewport shows: the temporal resolve from the drawn size to the output's,
/// then EASU's upscale or the box downscale to the viewport's, and RCAS's sharpening.
pub struct ResolveFeature {
  jitter: TemporalJitter,
  /// Where this frame's samples sit within their pixels, and the phases the jitter's sequence repeats over.
  offset: Vec2,
  phases: u32,
  /// The antialiasing asked for where it resolves temporally and the view shows its final frame; FSR where the view
  /// shows one of FSR's debug views.
  temporal: Option<RenderAntialiasing>,
  /// The FSR debug view shown, which FSR draws in place of the frame it resolved.
  fsr_view: Option<RenderDebugView>,
  /// The mip bias the surfaces' textures are read at this frame (`get_texture_bias`).
  texture_bias: f32,
  resolve: Option<Resolve>,
  upscaling: RenderUpscalingSettings,
  /// Whether the view lights in physical light, which picks ray reconstruction's model where NVIDIA's is asked.
  is_light_physical: bool,
  /// The size the frame is resolved at, which a temporal resolve resolves into.
  resolved: (u32, u32),
  /// The frame at the viewport's size while it is drawn or resolved at another: rescaled into the first, sharpened
  /// into the second.
  upscaled: Option<[FrameTexture; 2]>,
  /// The stage rescaling what was resolved, or the frame, to the viewport's size where it is of another.
  rescale: Option<UpscaleStage>,
  /// Whether the frame shown is sharpened: drawn smaller than the viewport or resolved temporally, with a sharpness
  /// asked.
  is_sharpened: bool,
  upscale: UpscaleUniform,
  /// The last frame's unjittered view projection, which every surface's motion is measured from.
  motion_previous: Option<Mat4>,
  /// The last frame's jittered view projection, which a read of its targets reprojects through.
  view_projection_previous: Option<Mat4>,
  /// Frames a temporal resolve gathered, which the screen-space effects turn their noise by.
  noise_frame: u32,
  /// When the view was last prepared, and the time since the one before, which DLSS weighs its history by.
  prepared_at: Option<Instant>,
  frame_time: Duration,
  /// The output, quality, preset and range DLSS last refused, which FSR resolves in its place until they change.
  refused: Option<DlssKey>,
  /// The output and quality ray reconstruction last refused, which super resolution upscales in its place until they
  /// change.
  refused_rays: Option<RayKey>,
  /// The sample of a reference this frame is drawn at, and the samples it takes: unresolved, at that place of the
  /// jitter's sequence.
  reference: Option<(u32, u32)>,
  /// The last frame's camera, its place and the way it looked, which a cut is told from (`is_cut`).
  last_camera: Option<(Vec3, Vec3)>,
  /// The scale this frame's light is written at and the last frame's, which each resolve is told so a history kept at
  /// the last frame's scale matches this one's while the exposure adapts.
  pre_exposure: PreExposure,
}

impl Default for ResolveFeature {
  fn default() -> Self {
    Self {
      jitter: TemporalJitter::default(),
      offset: Vec2::ZERO,
      phases: 1,
      temporal: None,
      fsr_view: None,
      texture_bias: 0.0,
      resolve: None,
      upscaling: RenderUpscalingSettings::default(),
      is_light_physical: false,
      resolved: (1, 1),
      upscaled: None,
      rescale: None,
      is_sharpened: false,
      upscale: UpscaleUniform::default(),
      motion_previous: None,
      view_projection_previous: None,
      noise_frame: 0,
      prepared_at: None,
      frame_time: Duration::ZERO,
      refused: None,
      refused_rays: None,
      reference: None,
      last_camera: None,
      pre_exposure: PreExposure::NONE,
    }
  }
}

impl ResolveFeature {
  /// The passes that resolve the frame: TAA, DLSS and DLSS's ray reconstruction.
  pub(crate) const TEMPORAL: &str = "temporal";
  pub(crate) const DLSS: &str = "dlss";
  pub(crate) const RAY_RECONSTRUCTION: &str = "dlss ray reconstruction";
  /// Metres moved, and the cosine of the angle turned, in one frame past which the camera cut away rather than moved:
  /// past a sprint or a fast fly at a low frame rate, and past a third of a turn.
  const CUT_DISTANCE: f32 = 10.0;
  const CUT_TURN_COSINE: f32 = 0.5;

  /// Where the next frame's samples sit within their pixels, in drawn pixels, `y` down, for a viewport `ratio` times
  /// the drawn size on a GPU that has DLSS where `has_dlss`: jittered while it resolves frames temporally and shows
  /// the finished frame or one of FSR's debug views, which FSR resolves whatever the antialiasing; still otherwise.
  pub fn next_jitter(&mut self, options: &RenderViewOptions, ratio: f32, has_dlss: bool) -> Vec2 {
    let view: RenderDebugView = options.mode.debug_view;

    // A reference's sample is drawn unresolved at its own place, its textures read at the bias DLSS reads them at
    // unscaled, as a resolve converging on it would; the screen-space effects' noise turned by the sample, so every
    // instant's mean averages the same turns and a still scene's reference holds still.
    if let Some((sample, samples)) = self.reference {
      self.fsr_view = None;
      self.temporal = None;
      self.phases = samples;
      self.texture_bias = Self::get_bias(Some(RenderAntialiasing::Dlss), 1.0);
      self.offset = TemporalJitter::at(sample);
      self.noise_frame = sample;

      return self.offset;
    }

    self.fsr_view = view.is_fsr().then_some(view);
    self.temporal = if self.fsr_view.is_some() {
      Some(RenderAntialiasing::Fsr)
    } else {
      let antialiasing: RenderAntialiasing = options.features.antialiasing.get_available(has_dlss);

      (antialiasing.is_temporal() && view == RenderDebugView::Final).then_some(antialiasing)
    };
    self.phases = TemporalJitter::get_phases(ratio);
    self.texture_bias = Self::get_bias(self.temporal, ratio);
    self.offset = if self.temporal.is_some() {
      self.jitter.next(ratio)
    } else {
      Vec2::ZERO
    };

    self.offset
  }

  /// The mip bias textures are read at while `temporal` resolves a frame `ratio` times smaller than its output a side.
  pub fn get_bias(temporal: Option<RenderAntialiasing>, ratio: f32) -> f32 {
    let lost: f32 = -ratio.max(1.0).log2();

    match temporal {
      Some(RenderAntialiasing::Fsr | RenderAntialiasing::Dlss) => lost - 1.0,
      Some(_) => lost,
      None => 0.0,
    }
  }

  /// The mip bias the surfaces' textures are read at this frame, as the last jitter decided it.
  pub fn get_texture_bias(&self) -> f32 {
    self.texture_bias
  }

  /// Takes this frame's resolve for targets `render` texels across, none before they are made, resolved at `resolved`
  /// and shown in `output`: the temporal resolve's history made or kept and what it reads, and the frame at the
  /// viewport's size while it is drawn or resolved at another. DLSS reconstructs rays where asked, the GPU has it and
  /// the scene holds linear light (`is_linear`), with the model the light asks (`is_light_physical`); else it upscales
  /// alone. The light is written at `pre_exposure`, which each resolve carries its history by.
  pub fn prepare(
    &mut self,
    (device, dlss): (&wgpu::Device, Option<&Dlss>),
    (view, now): (&CameraView, Instant),
    (render, resolved, output): (Option<(u32, u32)>, (u32, u32), RenderRect),
    (upscaling, (is_linear, is_light_physical), pre_exposure): (RenderUpscalingSettings, (bool, bool), PreExposure),
  ) {
    self.upscaling = upscaling;
    self.is_light_physical = is_light_physical;
    self.pre_exposure = pre_exposure;

    self.frame_time = self
      .prepared_at
      .map_or(Duration::ZERO, |before| now.duration_since(before));
    self.prepared_at = Some(now);

    // A resolve's frames turn the screen-space effects' noise on; a reference's sample set it (`next_jitter`).
    if self.temporal.is_some() && self.reference.is_none() {
      self.noise_frame = self.noise_frame.wrapping_add(1);
    }

    let display: (u32, u32) = (resolved.0.max(1), resolved.1.max(1));
    let viewport: (u32, u32) = (output.width.max(1), output.height.max(1));

    self.resolved = display;

    if self.is_cut(view)
      && let Some(resolve) = &mut self.resolve
    {
      resolve.reset();
    }

    match (render, self.temporal) {
      (Some(render), Some(RenderAntialiasing::Fsr)) => {
        self.prepare_fsr(device, view, (render, display), is_linear);
      }
      (Some(render), Some(RenderAntialiasing::Dlss)) => {
        let is_reconstructing: bool =
          upscaling.is_ray_reconstructed && is_linear && dlss.is_some_and(Dlss::has_ray_reconstruction);

        if !(is_reconstructing && self.prepare_ray_reconstruction((device, dlss), view, (render, display)))
          && !self.prepare_dlss((device, dlss), (render, display), is_linear)
        {
          self.temporal = Some(RenderAntialiasing::Fsr);
          self.prepare_fsr(device, view, (render, display), is_linear);
        }
      }
      (Some(render), Some(_)) => self.prepare_temporal(device, view, (render, display)),
      _ => self.resolve = None,
    }

    let Some(render) = render else {
      return;
    };
    // What reaches the viewport's rescale: what the temporal resolve resolved, or the frame as drawn.
    let source: (u32, u32) = if self.resolve.is_some() { display } else { render };

    self.rescale = (source != viewport).then_some(if source.0 < viewport.0 {
      UpscaleStage::Easu
    } else {
      UpscaleStage::Downsample
    });
    // Sharpened where upscaled, and where a temporal resolve gathers frames at the viewport's size, whose filtering and
    // resampled history soften what moves as an upscale does; never an FSR debug view, which shows its values as they
    // stand.
    self.is_sharpened =
      upscaling.sharpening > 0.0 && (render.0 < viewport.0 || self.resolve.is_some()) && self.fsr_view.is_none();

    if render == viewport && self.rescale.is_none() && !self.is_sharpened {
      self.upscaled = None;
    } else if self
      .upscaled
      .as_ref()
      .is_none_or(|[upscaled, _]| upscaled.get_size() != viewport)
    {
      self.upscaled = Some(["upscaled", "sharpened"].map(|label| {
        FrameTexture::new(
          device,
          label,
          viewport,
          UpscalePass::FORMAT,
          wgpu::TextureUsages::COPY_DST,
        )
      }));
    }

    self.upscale = UpscaleUniform {
      output_size: Vec2::new(viewport.0 as f32, viewport.1 as f32),
      sharpness: upscaling.get_sharpness(),
      linear: f32::from(u8::from(is_linear)),
    };
  }

  /// TAA's history at the output's size, kept while it still fits, and what it reads: this frame's view projection,
  /// the last one's, and the jitter.
  fn prepare_temporal(
    &mut self,
    device: &wgpu::Device,
    view: &CameraView,
    (render, display): ((u32, u32), (u32, u32)),
  ) {
    let is_fitting: bool = matches!(
      &self.resolve,
      Some(Resolve::Temporal(temporal))
        if temporal.render == render && temporal.history.get_written().get_size() == display
    );

    if !is_fitting {
      self.resolve = Some(Resolve::Temporal(Box::new(TemporalResolve {
        history: History::new(|side| {
          FrameTexture::new(
            device,
            ["temporal history 0", "temporal history 1"][side],
            display,
            TemporalPass::HISTORY,
            wgpu::TextureUsages::COPY_SRC,
          )
        }),
        flicker: History::new(|side| {
          FrameTexture::new(
            device,
            ["temporal flicker 0", "temporal flicker 1"][side],
            display,
            TemporalPass::FLICKER,
            wgpu::TextureUsages::empty(),
          )
        }),
        render,
        previous: None,
        uniform: TemporalUniform::default(),
      })));
    }

    let Some(Resolve::Temporal(temporal)) = &mut self.resolve else {
      return;
    };
    let TemporalResolve {
      history,
      previous,
      uniform,
      ..
    } = &mut **temporal;
    let current: Mat4 = view.get_view_projection();
    let (last, last_view): (Mat4, Mat4) = previous.unwrap_or((current, view.view));

    *uniform = TemporalUniform::new(
      (current, last, last_view),
      self.offset,
      history.is_valid() && previous.is_some(),
      self.pre_exposure.get_history_scale(),
    );
    *previous = Some((current, view.view));
  }

  /// FSR's targets, kept while they still fit, and its constants, over colour of linear light where `is_hdr`.
  fn prepare_fsr(
    &mut self,
    device: &wgpu::Device,
    view: &CameraView,
    (render, display): ((u32, u32), (u32, u32)),
    is_hdr: bool,
  ) {
    if !matches!(&self.resolve, Some(Resolve::Fsr { targets, .. }) if targets.is_sized(render, display)) {
      self.resolve = Some(Resolve::Fsr {
        targets: Box::new(FsrTargets::new(device, render, display, ViewTargets::SCENE)),
        uniform: FsrUniform::default(),
      });
    }

    let Some(Resolve::Fsr { targets, uniform }) = &mut self.resolve else {
      return;
    };

    // The frame before's jitter, turned back from FSR's sense; none read on a fresh frame.
    let previous_offset: Vec2 = -uniform.jitter;

    *uniform = FsrUniform::new(
      (render, display),
      (self.offset, previous_offset),
      view,
      (self.phases, targets.frames.get_frame()),
    )
    .with_hdr(is_hdr)
    .with_delta_pre_exposure(self.pre_exposure.get_history_scale());
  }

  /// DLSS's upscaler, kept while it still fits, upscaling targets `render` texels across into `display` pixels over
  /// colour of linear light where `is_hdr`; whether it resolves this frame, which FSR does where DLSS refuses them.
  fn prepare_dlss(
    &mut self,
    (device, dlss): (&wgpu::Device, Option<&Dlss>),
    (render, display): ((u32, u32), (u32, u32)),
    is_hdr: bool,
  ) -> bool {
    let model: (DlssQuality, DlssPreset) = (
      self.upscaling.scale.get_dlss_quality(),
      self.upscaling.dlss_preset.to_dlss(),
    );
    let key: DlssKey = (display, model, is_hdr);

    if self.refused == Some(key) {
      return false;
    }

    let Some(dlss) = dlss else {
      return false;
    };

    if !matches!(&self.resolve, Some(Resolve::Dlss(resolve)) if resolve.upscaler.is_fitting(display, model, is_hdr)) {
      // NGX makes and releases its upscalers with the device idle: the last one goes first.
      self.resolve = None;

      match dlss.create_upscaler(display, model, is_hdr) {
        Ok(upscaler) => {
          self.resolve = Some(Resolve::Dlss(Box::new(DlssResolve {
            upscaler,
            output: FrameTexture::new(
              device,
              "dlss output",
              display,
              DlssResolve::FORMAT,
              // NGX writes the output as storage and clears it with a transfer command.
              wgpu::TextureUsages::STORAGE_BINDING | wgpu::TextureUsages::COPY_DST,
            ),
            is_reset: true,
          })));
        }
        Err(error) => {
          log::warn!("FSR resolves in DLSS's place: {error}");
          self.refused = Some(key);

          return false;
        }
      }
    }

    let Some(Resolve::Dlss(resolve)) = &self.resolve else {
      return false;
    };
    let (min, max) = resolve.upscaler.get_render_range();

    if !((min.0..=max.0).contains(&render.0) && (min.1..=max.1).contains(&render.1)) {
      log::warn!(
        "FSR resolves in DLSS's place: DLSS upscales from {min:?} to {max:?} into {display:?}, not from {render:?}"
      );
      self.refused = Some(key);

      return false;
    }

    true
  }

  /// DLSS's ray reconstructor, kept while it still fits, reconstructing targets `render` texels across into `display`
  /// pixels, and this frame's view and projection with `view`'s jitter taken out; whether it resolves this frame,
  /// which super resolution does where it refuses them.
  fn prepare_ray_reconstruction(
    &mut self,
    (device, dlss): (&wgpu::Device, Option<&Dlss>),
    view: &CameraView,
    (render, display): ((u32, u32), (u32, u32)),
  ) -> bool {
    let quality: DlssQuality = self.upscaling.scale.get_dlss_quality();
    let preset: DlssRayPreset = self.upscaling.ray_reconstruction_preset.to_dlss(self.is_light_physical);
    let key: RayKey = (display, quality, preset);

    if self.refused_rays == Some(key) {
      return false;
    }

    let Some(dlss) = dlss else {
      return false;
    };
    let is_fitting: bool = matches!(
      &self.resolve,
      Some(Resolve::RayReconstruction(resolve)) if resolve.reconstructor.is_fitting(display, (quality, preset))
    );

    if !is_fitting {
      // NGX makes and releases its features with the device idle: the last one goes first.
      self.resolve = None;

      match dlss.create_ray_reconstructor(display, (quality, preset)) {
        Ok(reconstructor) => {
          self.resolve = Some(Resolve::RayReconstruction(Box::new(RayReconstructionResolve {
            reconstructor,
            output: FrameTexture::new(
              device,
              "ray reconstruction output",
              display,
              DlssResolve::FORMAT,
              // As super resolution's: NGX writes it as storage and clears it with a transfer command.
              wgpu::TextureUsages::STORAGE_BINDING | wgpu::TextureUsages::COPY_DST,
            ),
            is_reset: true,
            view: Mat4::IDENTITY,
            projection: Mat4::IDENTITY,
          })));
        }
        Err(error) => {
          log::warn!("DLSS upscales without reconstructing rays: {error}");
          self.refused_rays = Some(key);

          return false;
        }
      }
    }

    let Some(Resolve::RayReconstruction(resolve)) = &mut self.resolve else {
      return false;
    };
    let (min, max) = resolve.reconstructor.get_render_range();

    if !((min.0..=max.0).contains(&render.0) && (min.1..=max.1).contains(&render.1)) {
      log::warn!(
        "DLSS upscales without reconstructing rays: it reconstructs from {min:?} to {max:?} into {display:?}, not from {render:?}"
      );
      self.refused_rays = Some(key);

      return false;
    }

    (resolve.view, resolve.projection) = Self::get_ray_camera(view);

    true
  }

  /// The view and projection DLSS ray reconstruction places hits through: the frame's `view` as prepared, unjittered,
  /// since NGX takes the jitter apart.
  pub fn get_ray_camera(view: &CameraView) -> (Mat4, Mat4) {
    (view.view, view.projection)
  }

  /// Forgets the cameras a frame that was never submitted kept as the next frame's previous, the motion's and the
  /// jittered camera's, and resets the resolve, so the next frame reprojects nothing and resolves itself alone rather
  /// than over a history its motion no longer leads back to.
  pub fn abandon_frame(&mut self) {
    self.motion_previous = None;
    self.view_projection_previous = None;

    if let Some(resolve) = &mut self.resolve {
      resolve.reset();
    }
  }

  /// Whether the camera cut away since the last frame: moved or turned further in one frame than any camera travels,
  /// a teleport or a jump, after which no history leads anywhere true. Keeps this frame's camera for the next.
  pub(crate) fn is_cut(&mut self, view: &CameraView) -> bool {
    let forward: Vec3 = -view.view.row(2).truncate();
    let is_cut: bool = self.last_camera.is_some_and(|(position, last_forward)| {
      position.distance(view.position) > Self::CUT_DISTANCE || forward.dot(last_forward) < Self::CUT_TURN_COSINE
    });

    self.last_camera = Some((view.position, forward));
    is_cut
  }

  /// Starts again from nothing at the next frame: no history, made anew; the jitter's sequence from its first place;
  /// no last frame's cameras; and the noise from its first turn. Its frames from then on follow from those alone.
  pub fn restart(&mut self) {
    self.resolve = None;
    self.jitter = TemporalJitter::default();
    self.motion_previous = None;
    self.view_projection_previous = None;
    self.prepared_at = None;
    self.noise_frame = 0;
    self.last_camera = None;
  }

  /// Draws the next frames as a reference's `sample` of `samples`, unresolved; none resolves them again.
  pub fn set_reference(&mut self, reference: Option<(u32, u32)>) {
    self.reference = reference;
  }

  /// This frame's unjittered view projection and the last one's, which the surfaces' motion is measured between; the
  /// same twice for a first frame.
  pub fn next_motion(&mut self, current: Mat4) -> (Mat4, Mat4) {
    (current, self.motion_previous.replace(current).unwrap_or(current))
  }

  /// The last frame's jittered view projection, this frame's taking its place; this frame's own on the first.
  pub fn next_jittered(&mut self, current: Mat4) -> Mat4 {
    self.view_projection_previous.replace(current).unwrap_or(current)
  }

  /// Frames a temporal resolve gathered, or a reference's sample, which the screen-space effects turn their noise by.
  pub fn get_noise_frame(&self) -> u32 {
    self.noise_frame
  }

  /// The reference's sample this frame is drawn as, which its noise turns by; none outside a reference.
  pub fn get_reference_sample(&self) -> Option<u32> {
    self.reference.map(|(sample, _)| sample)
  }

  /// The antialiasing asked for where it resolves temporally.
  pub fn get_temporal(&self) -> Option<RenderAntialiasing> {
    self.temporal
  }

  pub fn get_upscaling(&self) -> RenderUpscalingSettings {
    self.upscaling
  }

  /// The size the frame is resolved at.
  pub fn get_resolved(&self) -> (u32, u32) {
    self.resolved
  }

  /// Whether DLSS reconstructs rays as it resolves this frame.
  pub fn is_ray_reconstructed(&self) -> bool {
    matches!(self.resolve, Some(Resolve::RayReconstruction(_)))
  }

  /// Whether the frame shown is one at the viewport's size apart from the frame as drawn, which the present reads.
  pub fn is_rescaled(&self) -> bool {
    self.upscaled.is_some()
  }

  /// The size of the image shown, that of the frame as drawn (`drawn`) unless it is rescaled.
  pub fn get_shown_size(&self, drawn: (u32, u32)) -> (u32, u32) {
    self
      .upscaled
      .as_ref()
      .map_or(drawn, |[upscaled, _]| upscaled.get_size())
  }

  /// Whether the temporal resolve's reactive weight reads the frame's copy as it stood before the water.
  fn is_reading_scene_before_water(&self) -> bool {
    matches!(self.resolve, Some(Resolve::Temporal(_)))
  }

  /// Copies the frame as it stands before the water, the composited surfaces and the particles draw, once for all that
  /// read it: the temporal resolve's reactive weight is what the rest change of it, ray reconstruction tells the
  /// transparent layers apart by it where it reads its guides (`is_guided`), and the enhanced water refracts and
  /// reflects it (`is_water_reading`). None where nothing reads it.
  pub fn add_before_water(
    &self,
    graph: &mut FrameGraph<'_>,
    handles: ViewTargetHandles,
    (is_guided, is_water_reading): (bool, bool),
  ) -> Option<GraphTexture> {
    (self.is_reading_scene_before_water() || is_guided || is_water_reading).then(|| {
      let (width, height) = handles.size;
      let copy: GraphTexture = graph.create_texture(GraphTextureDescriptor::new_2d(
        "scene before water",
        width,
        height,
        ViewTargets::SCENE,
      ));

      add_texture_copy(&mut *graph, "scene before water", handles.scene, copy);

      copy
    })
  }

  /// Copies the frame once its transparent layers, the rain last, drew, before the flares, the fog and the bloom, into
  /// the `guides` ray reconstruction reads it by against the frame before them (`before_water`).
  pub fn add_transparency_guide(
    graph: &mut FrameGraph<'_>,
    handles: ViewTargetHandles,
    (guides, before_water): (&mut Option<RayGuides>, Option<GraphTexture>),
  ) {
    let (Some(guides), Some(before)) = (guides, before_water) else {
      return;
    };
    let (width, height) = handles.size;
    let after: GraphTexture = graph.create_texture(GraphTextureDescriptor::new_2d(
      "scene after transparency",
      width,
      height,
      ViewTargets::SCENE,
    ));

    add_texture_copy(&mut *graph, "scene after transparency", handles.scene, after);
    guides.transparency = Some([before, after]);
  }

  /// What the temporal resolve's reactive weight compares the finished frame with, the frame as it stood before the
  /// water and the blended surfaces drew: TAA reads the frame's copy `before_water`, answered; FSR copies it into its
  /// own target, which its passes read.
  pub fn add_reactive_passes<'a>(
    &'a self,
    GraphDeclaration { graph, bindings, .. }: GraphDeclaration<'_, 'a>,
    (scene, before_water): (GraphTexture, Option<GraphTexture>),
  ) -> Option<GraphTexture> {
    match &self.resolve {
      Some(Resolve::Temporal(_)) => before_water,
      Some(Resolve::Dlss(_) | Resolve::RayReconstruction(_)) => None,
      Some(Resolve::Fsr { targets, .. }) => {
        let opaque: GraphTexture = bindings.import_view(&mut *graph, "fsr opaque", &targets.opaque.view);

        graph
          .add_encoder_pass("fsr opaque")
          .texture(scene, GraphTextureAccess::CopySource)
          .texture(opaque, GraphTextureAccess::CopyDestination)
          .record(move |context| {
            let (scene, opaque) = (context.get_texture(scene), context.get_texture(opaque));

            context.get_encoder().copy_texture_to_texture(
              scene.texture.as_image_copy(),
              opaque.texture.as_image_copy(),
              opaque.texture.size(),
            );
          });

        None
      }
      None => None,
    }
  }

  /// Adds the guides DLSS ray reconstruction reads, where it resolves this frame, at the drawn size: worked out from
  /// the G-buffer, the wet surface and reach and the reflections' trace (`traced`, a texel of nothing where none is)
  /// after combine.
  pub fn add_guide_passes<'a>(
    &self,
    graph: &mut FrameGraph<'a>,
    passes: LevelPasses<'a>,
    view: &'a ViewBinding,
    (handles, lighting, (wet_surface, wet_reach), traced): (
      ViewTargetHandles,
      UniformBinding<LightingUniform>,
      (GraphTexture, GraphTexture),
      GraphTexture,
    ),
    (diffuse_hits, none): (Option<GraphTexture>, GraphTexture),
  ) -> Option<RayGuides> {
    if !self.is_ray_reconstructed() {
      return None;
    }

    let (width, height) = handles.size;
    let mut create = |label: &'static str, format: wgpu::TextureFormat| -> GraphTexture {
      graph.create_texture(GraphTextureDescriptor::new_2d(label, width, height, format))
    };
    let guides: RayGuides = RayGuides {
      diffuse_albedo: create("ray guides diffuse albedo", RayGuidesPass::DIFFUSE_ALBEDO),
      specular_albedo: create("ray guides specular albedo", RayGuidesPass::SPECULAR_ALBEDO),
      normals: create("ray guides normals", RayGuidesPass::NORMALS),
      hit_distance: create("ray guides hit distance", RayGuidesPass::HIT_DISTANCE),
      bias: create("ray guides bias", RayGuidesPass::BIAS),
      transparency: None,
      diffuse_hit_distance: None,
    };
    // Written whether or not the indirect light is traced, given to the reconstruction only where it is.
    let diffuse_hit_distance: GraphTexture = create("ray guides diffuse hit distance", RayGuidesPass::HIT_DISTANCE);
    let parameters: RayGuidesParameters = RayGuidesParameters {
      albedo_target: handles.albedo,
      normal_target: handles.normal,
      material_target: handles.material,
      depth_target: handles.depth,
      wet_surface,
      wet_reach,
      lighting,
      traced,
      diffuse_hits: diffuse_hits.unwrap_or(none),
    };
    let clear =
      |texture: GraphTexture| GraphColorAttachment::new(texture, wgpu::LoadOp::Clear(wgpu::Color::TRANSPARENT));

    graph
      .add_raster_pass("ray guides")
      .parameters(&parameters)
      .color(clear(guides.specular_albedo))
      .color(clear(guides.normals))
      .color(clear(guides.hit_distance))
      .color(clear(guides.bias))
      .color(clear(diffuse_hit_distance))
      .color(clear(guides.diffuse_albedo))
      .record(move |context| passes.ray_guides.record(context, view, &parameters));

    Some(RayGuides {
      diffuse_hit_distance: diffuse_hits.map(|_| diffuse_hit_distance),
      ..guides
    })
  }

  /// Adds the resolve of `frame`, the finished frame at the drawn size, against the frame as it stood before the
  /// blended surfaces (`opaque`, the scene where none was kept), DLSS's ray reconstruction reading `guides`. At the
  /// drawn size, the temporal resolve; then, at the viewport's, the upscale where nothing upscaled it and the
  /// sharpening. Answers the frame at the drawn size and the image the viewport shows.
  pub fn add_passes<'a>(
    &'a self,
    GraphDeclaration {
      graph,
      bindings,
      runtime,
    }: GraphDeclaration<'_, 'a>,
    passes: LevelPasses<'a>,
    view: &'a ViewBinding,
    (handles, frame, opaque, guides): (ViewTargetHandles, GraphTexture, Option<GraphTexture>, Option<RayGuides>),
  ) -> (GraphTexture, GraphTexture) {
    let upscaled: Option<[GraphTexture; 2]> = self.upscaled.as_ref().map(|[upscaled, sharpened]| {
      [
        bindings.import_view(&mut *graph, "upscaled", &upscaled.view),
        bindings.import_view(&mut *graph, "sharpened", &sharpened.view),
      ]
    });
    let upscale: UniformBinding<UpscaleUniform> = runtime.push_uniform(&self.upscale);
    let resolved: Option<GraphTexture> = match &self.resolve {
      Some(Resolve::Fsr { targets, uniform }) => {
        let history: GraphTexture =
          bindings.import_view(&mut *graph, "fsr history", &targets.frames.get_written().history.view);
        // An FSR debug view is drawn at the display's size and shown in place of the history.
        let debug: Option<(RenderDebugView, GraphTexture)> = self.fsr_view.map(|view| {
          let (width, height) = targets.display;

          (
            view,
            graph.create_texture(GraphTextureDescriptor::new_2d(
              "fsr debug view",
              width,
              height,
              FsrTargets::DEBUG,
            )),
          )
        });

        passes.fsr.add_passes(
          GraphDeclaration::new(graph, bindings, runtime),
          handles,
          (targets, uniform),
          debug,
        );

        Some(debug.map_or(history, |(_, shown)| shown))
      }
      Some(Resolve::Dlss(resolve)) => Some(self.add_dlss_pass(
        GraphDeclaration::new(graph, bindings, runtime),
        (resolve, handles, frame),
      )),
      Some(Resolve::RayReconstruction(resolve)) => guides.map(|guides| {
        self.add_ray_reconstruction_pass(
          GraphDeclaration::new(graph, bindings, runtime),
          (resolve, handles, frame, guides),
        )
      }),
      Some(Resolve::Temporal(temporal)) => {
        let TemporalResolve {
          history,
          flicker,
          uniform,
          ..
        } = &**temporal;
        let target: GraphTexture = bindings.import_view(&mut *graph, "temporal history", &history.get_written().view);
        let flickered: GraphTexture =
          bindings.import_view(&mut *graph, "temporal flicker", &flicker.get_written().view);
        let parameters: TemporalParameters = TemporalParameters {
          frame,
          depth_target: handles.depth,
          history: bindings.import_view(&mut *graph, "temporal history", &history.get_read().view),
          history_sampler: passes.temporal.get_sampler(),
          temporal: runtime.push_uniform(uniform),
          motion_target: handles.motion,
          opaque: opaque.unwrap_or(handles.scene),
          flicker_history: bindings.import_view(&mut *graph, "temporal flicker", &flicker.get_read().view),
        };

        graph
          .add_raster_pass(Self::TEMPORAL)
          .parameters(&parameters)
          .color(GraphColorAttachment::new(
            target,
            wgpu::LoadOp::Clear(wgpu::Color::TRANSPARENT),
          ))
          .color(GraphColorAttachment::new(
            flickered,
            wgpu::LoadOp::Clear(wgpu::Color::TRANSPARENT),
          ))
          .record(move |context| passes.temporal.record(context, view, &parameters));

        Some(target)
      }
      None => None,
    };
    let (frame, mut shown): (GraphTexture, Option<GraphTexture>) = match (self.rescale, upscaled, resolved) {
      (Some(stage), Some([upscaled, _]), resolved) => {
        let parameters: UpscaleParameters = UpscaleParameters {
          source: resolved.unwrap_or(frame),
          upscale,
        };

        graph
          .add_raster_pass(stage.get_name())
          .parameters(&parameters)
          .color(GraphColorAttachment::new(
            upscaled,
            wgpu::LoadOp::Clear(wgpu::Color::TRANSPARENT),
          ))
          .record(move |context| passes.upscale.record(context, stage, &parameters));

        (frame, Some(upscaled))
      }
      (_, Some(_), Some(resolved)) => (frame, Some(resolved)),
      (_, None, Some(resolved)) => (resolved, None),
      _ => (frame, None),
    };

    if let (true, Some([upscaled, sharpened])) = (self.is_sharpened, upscaled) {
      let parameters: UpscaleParameters = UpscaleParameters {
        source: shown.unwrap_or(upscaled),
        upscale,
      };

      graph
        .add_raster_pass(UpscaleStage::Rcas.get_name())
        .parameters(&parameters)
        .color(GraphColorAttachment::new(
          sharpened,
          wgpu::LoadOp::Clear(wgpu::Color::TRANSPARENT),
        ))
        .record(move |context| passes.upscale.record(context, UpscaleStage::Rcas, &parameters));
      shown = Some(sharpened);
    }

    (frame, shown.unwrap_or(frame))
  }

  /// Adds DLSS's upscale of `frame`, the finished frame at the drawn size, through the view's depth and motion, and
  /// answers the frame it writes at the output's size. It is given no mask of what to take from this frame alone:
  /// the transformer presets ignore one.
  fn add_dlss_pass<'a>(
    &'a self,
    GraphDeclaration { graph, bindings, .. }: GraphDeclaration<'_, 'a>,
    (resolve, handles, frame): (&'a DlssResolve, ViewTargetHandles, GraphTexture),
  ) -> GraphTexture {
    let output: GraphTexture = bindings.import_view(&mut *graph, "dlss output", &resolve.output.view);
    // FSR's sense of the jitter, which DLSS shares: a drawn texel `m` stands at `m + 0.5 - jitter`.
    let jitter: [f32; 2] = (-self.offset).to_array();
    let is_reset: bool = resolve.is_reset;
    let frame_time: Duration = self.frame_time;
    let pre_exposure: f32 = self.pre_exposure.current;

    graph
      .add_encoder_pass(Self::DLSS)
      .texture(frame, GraphTextureAccess::Sampled)
      .texture(handles.depth, GraphTextureAccess::Sampled)
      .texture(handles.motion, GraphTextureAccess::Sampled)
      .texture(output, GraphTextureAccess::StorageReadWrite)
      .record(move |context| {
        let color = context.get_texture(frame);
        let render: (u32, u32) = (color.texture.width(), color.texture.height());
        let inputs: DlssInputs<'_> = DlssInputs {
          color: color.view,
          depth: context.get_texture(handles.depth).view,
          motion: context.get_texture(handles.motion).view,
          // The motion is kept in texture coordinates, from where a point stood to where it stands; DLSS reads render
          // pixels back to where it stood.
          motion_scale: [-(render.0 as f32), -(render.1 as f32)],
          jitter,
          render,
          output: context.get_texture(output).view,
          is_reset,
          frame_time,
          pre_exposure,
        };

        match resolve.upscaler.record(context.get_encoder(), &inputs) {
          Ok(commands) => context.hand_over(commands),
          Err(error) => log::error!("{error}"),
        }
      });

    output
  }

  /// Adds DLSS's ray reconstruction of `frame`, the finished frame at the drawn size, through the view's depth, motion
  /// and albedo and the frame's `guides`, and answers the frame it writes at the output's size.
  fn add_ray_reconstruction_pass<'a>(
    &'a self,
    GraphDeclaration { graph, bindings, .. }: GraphDeclaration<'_, 'a>,
    (resolve, handles, frame, guides): (&'a RayReconstructionResolve, ViewTargetHandles, GraphTexture, RayGuides),
  ) -> GraphTexture {
    let output: GraphTexture = bindings.import_view(&mut *graph, "ray reconstruction output", &resolve.output.view);
    // FSR's sense of the jitter, which DLSS shares: a drawn texel `m` stands at `m + 0.5 - jitter`.
    let jitter: [f32; 2] = (-self.offset).to_array();
    let is_reset: bool = resolve.is_reset;
    let pre_exposure: f32 = self.pre_exposure.current;

    let mut pass = graph
      .add_encoder_pass(Self::RAY_RECONSTRUCTION)
      .texture(frame, GraphTextureAccess::Sampled)
      .texture(handles.depth, GraphTextureAccess::Sampled)
      .texture(handles.motion, GraphTextureAccess::Sampled)
      .texture(guides.diffuse_albedo, GraphTextureAccess::Sampled)
      .texture(guides.specular_albedo, GraphTextureAccess::Sampled)
      .texture(guides.normals, GraphTextureAccess::Sampled)
      .texture(guides.hit_distance, GraphTextureAccess::Sampled)
      .texture(guides.bias, GraphTextureAccess::Sampled)
      .texture(output, GraphTextureAccess::StorageReadWrite);

    if let Some(diffuse) = guides.diffuse_hit_distance {
      pass = pass.texture(diffuse, GraphTextureAccess::Sampled);
    }

    if let Some([before, after]) = guides.transparency {
      pass = pass
        .texture(before, GraphTextureAccess::Sampled)
        .texture(after, GraphTextureAccess::Sampled);
    }

    pass.record(move |context| {
      let color = context.get_texture(frame);
      let render: (u32, u32) = (color.texture.width(), color.texture.height());
      let inputs: DlssRayInputs<'_> = DlssRayInputs {
        color: color.view,
        depth: context.get_texture(handles.depth).view,
        motion: context.get_texture(handles.motion).view,
        // As super resolution reads it: render pixels back to where a point stood.
        motion_scale: [-(render.0 as f32), -(render.1 as f32)],
        diffuse_albedo: context.get_texture(guides.diffuse_albedo).view,
        specular_albedo: context.get_texture(guides.specular_albedo).view,
        normals: context.get_texture(guides.normals).view,
        specular_hit_distance: context.get_texture(guides.hit_distance).view,
        diffuse_hit_distance: guides
          .diffuse_hit_distance
          .map(|diffuse| context.get_texture(diffuse).view),
        bias_current_color: Some(context.get_texture(guides.bias).view),
        transparency: guides
          .transparency
          .map(|[before, after]| (context.get_texture(before).view, context.get_texture(after).view)),
        world_to_view: resolve.view,
        view_to_clip: resolve.projection,
        jitter,
        render,
        output: context.get_texture(output).view,
        is_reset,
        pre_exposure,
      };

      match resolve.reconstructor.record(context.get_encoder(), &inputs) {
        Ok(commands) => context.hand_over(commands),
        Err(error) => log::error!("{error}"),
      }
    });

    output
  }
}

impl ViewFeature for ResolveFeature {
  /// The temporal resolve's histories and FSR's targets, and the frame at the viewport's size.
  fn get_memory(&self) -> u64 {
    let upscaled: u64 = self.upscaled.iter().flatten().map(|texture| texture.get_memory()).sum();

    self.resolve.as_ref().map_or(0, Resolve::get_memory) + upscaled
  }

  fn release(&mut self) {
    self.resolve = None;
    self.upscaled = None;
  }

  /// The resolve's history this frame wrote becomes the one the next reads.
  fn end_frame(&mut self, _end: &mut ViewFrameEnd<'_>) {
    if let Some(resolve) = &mut self.resolve {
      resolve.swap();
    }
  }
}
