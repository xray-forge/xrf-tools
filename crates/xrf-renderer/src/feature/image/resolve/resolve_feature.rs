use std::time::{Duration, Instant};

use glam::{Mat4, Vec2, Vec3};
use xrf_dlss::{Dlss, DlssPreset, DlssQuality, DlssRayPreset};

use crate::camera::camera_view::CameraView;
use crate::contract::feature::image::resolve::render_antialiasing::RenderAntialiasing;
use crate::contract::feature::image::resolve::render_upscaling_settings::RenderUpscalingSettings;
use crate::contract::report::applied::render_applied_report::RenderAppliedReport;
use crate::contract::view::output::render_debug_view::RenderDebugView;
use crate::contract::view::render_view_options::RenderViewOptions;
use crate::feature::image::exposure::reading::pre_exposure::PreExposure;
use crate::feature::image::resolve::resolve::Resolve;
use crate::feature::image::resolve::temporal_jitter::TemporalJitter;
use crate::feature::image::resolve::upscale::upscale_stage::UpscaleStage;
use crate::feature::image::resolve::upscale::upscale_uniform::UpscaleUniform;
use crate::feature::view::view_feature::ViewFeature;
use crate::feature::view::view_frame_end::ViewFrameEnd;
use crate::feature::view::view_restart::ViewRestart;
use crate::pass::view::view_motion::ViewMotion;

mod resolve_feature_prepare;
mod resolve_feature_record;

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
  /// The viewport's size while the frame shown is one apart from the frame as drawn (`is_rescaled`): rescaled into a
  /// frame graph texture of that size where `is_upscaled`, sharpened into another where `is_sharpened`, each made by the
  /// frame that writes it and read within it alone.
  shown_size: Option<(u32, u32)>,
  /// Whether this frame may write the rescaled frame (`is_upscale_written`).
  is_upscaled: bool,
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
  /// Whether the camera cut away this frame, as its motion was taken (`next_motion`).
  is_camera_cut: bool,
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
      shown_size: None,
      is_upscaled: false,
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
      is_camera_cut: false,
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
  /// the drawn size on a GPU that has DLSS where `dlss` gives it: jittered while it resolves frames temporally and
  /// shows the finished frame or one of FSR's debug views, which FSR resolves whatever the antialiasing; still
  /// otherwise.
  pub fn next_jitter(&mut self, options: &RenderViewOptions, ratio: f32, dlss: Option<&Dlss>) -> Vec2 {
    let has_dlss: bool = dlss.is_some();
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

  /// Whether this frame's camera cut away from the last one's, as `next_motion` found it, until the prepare takes it.
  pub fn is_camera_cut(&self) -> bool {
    self.is_camera_cut
  }

  /// Draws the next frames as a reference's `sample` of `samples`, unresolved; none resolves them again.
  pub fn set_reference(&mut self, reference: Option<(u32, u32)>) {
    self.reference = reference;
  }

  /// What this frame, seen through `view` and drawn by the `jittered` view projection, measures its motion and
  /// reprojects by, each last frame's taking this one's place: this frame's own on a first frame and where the camera
  /// cut away (`is_cut`), so nothing moves across a teleport. The cut is kept for the prepare, which resets the history.
  pub fn next_motion(&mut self, view: &CameraView, jittered: Mat4) -> ViewMotion {
    let current: Mat4 = view.get_view_projection();

    self.is_camera_cut = self.is_cut(view);

    let still: ViewMotion = ViewMotion::still(current, jittered);
    let previous: Option<Mat4> = self.motion_previous.replace(current);
    let jittered_previous: Option<Mat4> = self.view_projection_previous.replace(jittered);

    if self.is_camera_cut {
      return still;
    }

    ViewMotion {
      previous: previous.unwrap_or(current),
      jittered_previous: jittered_previous.unwrap_or(jittered),
      ..still
    }
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

  /// Writes into `applied` what it resolves this frame at: the share of the output the scene is rendered at.
  pub fn describe(&self, applied: &mut RenderAppliedReport) {
    applied.render_scale = self.upscaling.scale;
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
    self.shown_size.is_some()
  }

  /// Whether a frame drawn `render` texels across into a `viewport` writes the rescaled frame: where it is rescaled
  /// (`rescale`), and where ray reconstruction resolves it at another size, since a frame it draws nothing for (before
  /// its guides stand) leaves the frame at the drawn size for the rescale to take.
  pub fn is_upscale_written(
    rescale: Option<UpscaleStage>,
    is_ray_reconstructed: bool,
    render: (u32, u32),
    viewport: (u32, u32),
  ) -> bool {
    rescale.is_some() || (is_ray_reconstructed && render != viewport)
  }

  /// The pass taking an image `source` texels across to the `viewport`'s size: up by EASU, down by the downsample;
  /// none where they are the same.
  fn get_rescale(source: (u32, u32), viewport: (u32, u32)) -> Option<UpscaleStage> {
    (source != viewport).then_some(if source.0 < viewport.0 {
      UpscaleStage::Easu
    } else {
      UpscaleStage::Downsample
    })
  }

  /// The size of the image shown, that of the frame as drawn (`drawn`) unless it is rescaled.
  pub fn get_shown_size(&self, drawn: (u32, u32)) -> (u32, u32) {
    self.shown_size.unwrap_or(drawn)
  }
}

impl ViewFeature for ResolveFeature {
  /// The temporal resolve's histories and FSR's targets.
  fn get_memory(&self) -> u64 {
    self.resolve.as_ref().map_or(0, Resolve::get_memory)
  }

  fn release(&mut self) {
    self.resolve = None;
    self.shown_size = None;
    self.is_upscaled = false;
  }

  /// The resolve's history this frame wrote becomes the one the next reads.
  fn end_frame(&mut self, _end: &mut ViewFrameEnd<'_>) {
    if let Some(resolve) = &mut self.resolve {
      resolve.swap();
    }
  }

  /// Its motion restarted, it starts again from nothing at the next frame: no history, made anew; the jitter's
  /// sequence from its first place; no last frame's cameras; and the noise from its first turn. Its frames from then on
  /// follow from those alone. A cut it found itself (`is_cut`), and its prepare resets the history for it.
  fn restart(&mut self, restart: ViewRestart) {
    if restart == ViewRestart::Motion {
      self.resolve = None;
      self.jitter = TemporalJitter::default();
      self.motion_previous = None;
      self.view_projection_previous = None;
      self.prepared_at = None;
      self.noise_frame = 0;
      self.last_camera = None;
      self.is_camera_cut = false;
    }
  }
}
