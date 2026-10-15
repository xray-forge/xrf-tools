use glam::{Mat4, Vec2};
use xrf_dlss::{Dlss, DlssInputs, DlssPreset, DlssQuality, DlssRayInputs};
use xrf_renderer_core::{
  FrameGraph, GraphBindings, GraphColorAttachment, GraphRuntime, GraphTexture, GraphTextureAccess,
  GraphTextureDescriptor, UniformBinding,
};

use crate::camera::camera_view::CameraView;
use crate::contract::render_antialiasing::RenderAntialiasing;
use crate::contract::render_debug_view::RenderDebugView;
use crate::contract::render_rect::RenderRect;
use crate::contract::render_upscaling_settings::RenderUpscalingSettings;
use crate::contract::render_view_options::RenderViewOptions;
use crate::frame::frame_texture::FrameTexture;
use crate::frame::fsr_targets::FsrTargets;
use crate::frame::history::History;
use crate::frame::temporal_jitter::TemporalJitter;
use crate::frame::view_target_handles::ViewTargetHandles;
use crate::frame::view_targets::ViewTargets;
use crate::pass::dlss_reactive_parameters::DlssReactiveParameters;
use crate::pass::dlss_reactive_pass::DlssReactivePass;
use crate::pass::fsr_uniform::FsrUniform;
use crate::pass::level_passes::LevelPasses;
use crate::pass::lighting_uniform::LightingUniform;
use crate::pass::ray_guides_parameters::RayGuidesParameters;
use crate::pass::ray_guides_pass::RayGuidesPass;
use crate::pass::temporal_parameters::TemporalParameters;
use crate::pass::temporal_pass::TemporalPass;
use crate::pass::temporal_uniform::TemporalUniform;
use crate::pass::upscale_parameters::UpscaleParameters;
use crate::pass::upscale_pass::UpscalePass;
use crate::pass::upscale_uniform::UpscaleUniform;
use crate::pass::view_binding::ViewBinding;
use crate::scene::level::feature::dlss_resolve::DlssResolve;
use crate::scene::level::feature::ray_guides::RayGuides;
use crate::scene::level::feature::ray_reconstruction_resolve::RayReconstructionResolve;
use crate::scene::level::feature::resolve::Resolve;
use crate::scene::level::feature::temporal_resolve::TemporalResolve;
use crate::scene::level::feature::view_feature::ViewFeature;

/// What a DLSS upscaler is made for: its output, quality and preset, and whether the colour is linear light.
type DlssKey = ((u32, u32), (DlssQuality, DlssPreset), bool);

/// What a DLSS ray reconstructor is made for: its output and quality.
type RayKey = ((u32, u32), DlssQuality);

/// How a view's frame becomes the image its viewport shows. From the drawn size to the output's: the temporal resolve
/// (TAA, or FSR 2 or DLSS upscaling as they resolve, DLSS reconstructing rays where asked) with the jitter its samples
/// are moved by. Then, to the viewport's
/// size where what was resolved, or the frame where nothing was, is of another: EASU's upscale or the box downscale; and
/// RCAS's sharpening where the frame was drawn smaller than the viewport.
pub struct ResolveFeature {
  jitter: TemporalJitter,
  /// Where this frame's samples sit within their pixels, and the phases the jitter's sequence repeats over.
  offset: Vec2,
  phases: u32,
  /// The antialiasing asked for where it resolves temporally and the view shows its final frame.
  temporal: Option<RenderAntialiasing>,
  resolve: Option<Resolve>,
  upscaling: RenderUpscalingSettings,
  /// The size the frame is resolved at, which a temporal resolve resolves into.
  resolved: (u32, u32),
  /// The frame at the viewport's size while it is drawn or resolved at another: rescaled into the first, sharpened
  /// into the second.
  upscaled: Option<[FrameTexture; 2]>,
  /// The stage rescaling what was resolved, or the frame, to the viewport's size where it is of another.
  rescale: Option<usize>,
  /// Whether the frame shown is sharpened: drawn smaller than the viewport, with a sharpness asked.
  is_sharpened: bool,
  upscale: UpscaleUniform,
  /// The last frame's unjittered view projection, which every surface's motion is measured from.
  motion_previous: Option<Mat4>,
  /// The last frame's jittered view projection, which a read of its targets reprojects through.
  view_projection_previous: Option<Mat4>,
  /// Frames a temporal resolve gathered, which the screen-space effects turn their noise by.
  noise_frame: u32,
  /// The output, quality, preset and range DLSS last refused, which FSR 2 resolves in its place until they change.
  refused: Option<DlssKey>,
  /// The output and quality ray reconstruction last refused, which super resolution upscales in its place until they
  /// change.
  refused_rays: Option<RayKey>,
}

impl Default for ResolveFeature {
  fn default() -> Self {
    Self {
      jitter: TemporalJitter::default(),
      offset: Vec2::ZERO,
      phases: 1,
      temporal: None,
      resolve: None,
      upscaling: RenderUpscalingSettings::default(),
      resolved: (1, 1),
      upscaled: None,
      rescale: None,
      is_sharpened: false,
      upscale: UpscaleUniform::default(),
      motion_previous: None,
      view_projection_previous: None,
      noise_frame: 0,
      refused: None,
      refused_rays: None,
    }
  }
}

impl ResolveFeature {
  /// Where the next frame's samples sit within their pixels, in drawn pixels, `y` down, for a viewport `ratio` times
  /// the drawn size on a GPU that has DLSS where `has_dlss`: jittered while it resolves frames temporally and shows
  /// the finished frame, still otherwise.
  pub fn next_jitter(&mut self, options: &RenderViewOptions, ratio: f32, has_dlss: bool) -> Vec2 {
    let antialiasing: RenderAntialiasing = options.features.antialiasing.get_available(has_dlss);

    self.temporal =
      (antialiasing.is_temporal() && options.mode.debug_view == RenderDebugView::Final).then_some(antialiasing);
    self.phases = TemporalJitter::get_phases(ratio);
    self.offset = if self.temporal.is_some() {
      self.jitter.next(ratio)
    } else {
      Vec2::ZERO
    };

    self.offset
  }

  /// Takes this frame's resolve for targets `render` texels across, none before they are made, resolved at `resolved`
  /// and shown in `output`: the temporal resolve's history made or kept and what it reads, and the frame at the
  /// viewport's size while it is drawn or resolved at another. DLSS reconstructs rays where asked, the GPU has it and
  /// the scene holds linear light; else it upscales alone.
  pub fn prepare(
    &mut self,
    (device, dlss): (&wgpu::Device, Option<&Dlss>),
    view: &CameraView,
    (render, resolved, output): (Option<(u32, u32)>, (u32, u32), RenderRect),
    (upscaling, is_linear): (RenderUpscalingSettings, bool),
  ) {
    self.upscaling = upscaling;

    if self.temporal.is_some() {
      self.noise_frame = self.noise_frame.wrapping_add(1);
    }

    let display: (u32, u32) = (resolved.0.max(1), resolved.1.max(1));
    let viewport: (u32, u32) = (output.width.max(1), output.height.max(1));

    self.resolved = display;

    match (render, self.temporal) {
      (Some(render), Some(RenderAntialiasing::Fsr2)) => {
        self.prepare_fsr(device, view, (render, display), is_linear);
      }
      (Some(render), Some(RenderAntialiasing::Dlss)) => {
        let is_reconstructing: bool =
          upscaling.is_ray_reconstructed && is_linear && dlss.is_some_and(Dlss::has_ray_reconstruction);

        if !(is_reconstructing && self.prepare_ray_reconstruction((device, dlss), view, (render, display)))
          && !self.prepare_dlss((device, dlss), (render, display), is_linear)
        {
          self.temporal = Some(RenderAntialiasing::Fsr2);
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
      UpscalePass::EASU
    } else {
      UpscalePass::DOWNSAMPLE
    });
    self.is_sharpened = upscaling.sharpening > 0.0 && render.0 < viewport.0;

    if render == viewport && self.rescale.is_none() {
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
      current,
      last,
      last_view,
      self.offset,
      history.is_valid() && previous.is_some(),
    );
    *previous = Some((current, view.view));
  }

  /// FSR 2's targets, kept while they still fit, and its constants, over colour of linear light where `is_hdr`.
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

    *uniform = FsrUniform::new(
      (render, display),
      self.offset,
      view,
      (FsrTargets::get_luma_mip_size(render), self.phases),
      targets.frames.get_frame(),
    )
    .with_hdr(is_hdr);
  }

  /// DLSS's upscaler, kept while it still fits, upscaling targets `render` texels across into `display` pixels over
  /// colour of linear light where `is_hdr`; whether it resolves this frame, which FSR 2 does where DLSS refuses them.
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
              wgpu::TextureUsages::STORAGE_BINDING,
            ),
            is_reset: true,
          })));
        }
        Err(error) => {
          log::warn!("FSR 2 resolves in DLSS's place: {error}");
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
        "FSR 2 resolves in DLSS's place: DLSS upscales from {min:?} to {max:?} into {display:?}, not from {render:?}"
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
    let key: RayKey = (display, quality);

    if self.refused_rays == Some(key) {
      return false;
    }

    let Some(dlss) = dlss else {
      return false;
    };
    let is_fitting: bool = matches!(
      &self.resolve,
      Some(Resolve::RayReconstruction(resolve)) if resolve.reconstructor.is_fitting(display, quality)
    );

    if !is_fitting {
      // NGX makes and releases its features with the device idle: the last one goes first.
      self.resolve = None;

      match dlss.create_ray_reconstructor(display, quality) {
        Ok(reconstructor) => {
          self.resolve = Some(Resolve::RayReconstruction(Box::new(RayReconstructionResolve {
            reconstructor,
            output: FrameTexture::new(
              device,
              "ray reconstruction output",
              display,
              DlssResolve::FORMAT,
              wgpu::TextureUsages::STORAGE_BINDING,
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

    let offset: Vec2 = self.offset;
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

    // The jitter `CameraView::jittered` put into the projection, taken out again.
    let mut projection: Mat4 = view.projection;

    projection.z_axis.x -= 2.0 * offset.x / render.0 as f32;
    projection.z_axis.y += 2.0 * offset.y / render.1 as f32;
    resolve.view = view.view;
    resolve.projection = projection;

    true
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

  /// Frames a temporal resolve gathered, which the screen-space effects turn their noise by.
  pub fn get_noise_frame(&self) -> u32 {
    self.noise_frame
  }

  /// Whether a temporal resolve gathers this frame's samples, which the screen-space effects turn their noise by.
  pub fn is_temporal(&self) -> bool {
    self.temporal.is_some()
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

  /// Whether the temporal resolve's reactive weight, or DLSS's mask of what to upscale from this frame alone, reads the
  /// frame's copy as it stood before the water.
  pub fn is_reading_scene_before_water(&self) -> bool {
    matches!(self.resolve, Some(Resolve::Temporal(_) | Resolve::Dlss(_)))
  }

  /// What the temporal resolve's reactive weight compares the finished frame with, the frame as it stood before the
  /// water and the blended surfaces drew: TAA and DLSS read the frame's copy `before_water`, answered; FSR 2 copies it
  /// into its own target, which its passes read.
  pub fn add_reactive_passes<'a>(
    &'a self,
    (graph, bindings): (&mut FrameGraph<'a>, &mut GraphBindings<'a>),
    (scene, before_water): (GraphTexture, Option<GraphTexture>),
  ) -> Option<GraphTexture> {
    match &self.resolve {
      Some(Resolve::Temporal(_) | Resolve::Dlss(_)) => before_water,
      Some(Resolve::RayReconstruction(_)) => None,
      Some(Resolve::Fsr { targets, .. }) => {
        let opaque: GraphTexture = bindings.import_view(&mut *graph, "fsr2 opaque", &targets.opaque.view);

        graph
          .add_encoder_pass("fsr2 opaque")
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

  /// Adds the guides DLSS ray reconstruction reads, where it resolves this frame, at the drawn size: worked out from the
  /// G-buffer, the wet surface and the reflections' trace (`traced`, a texel of nothing where none is) after combine.
  pub fn add_guide_passes<'a>(
    &self,
    graph: &mut FrameGraph<'a>,
    passes: LevelPasses<'a>,
    view: &'a ViewBinding,
    (handles, lighting, wet_surface, traced): (
      ViewTargetHandles,
      UniformBinding<LightingUniform>,
      GraphTexture,
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
      .record(move |context| passes.ray_guides.record(context, view, &parameters));

    Some(RayGuides {
      diffuse_hit_distance: diffuse_hits.map(|_| diffuse_hit_distance),
      ..guides
    })
  }

  /// Adds the resolve of `frame`, the finished frame at the drawn size, against the frame as it stood before the
  /// blended surfaces (`opaque`, the scene where none was kept), DLSS's ray reconstruction reading `guides`. At the drawn size, the temporal resolve; then, at the
  /// viewport's, the upscale where nothing upscaled it and the sharpening. Answers the frame at the drawn size, which
  /// the present distorts, and the image the viewport shows.
  pub fn add_passes<'a>(
    &'a self,
    (graph, bindings, runtime): (&mut FrameGraph<'a>, &mut GraphBindings<'a>, &mut GraphRuntime),
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
          bindings.import_view(&mut *graph, "fsr2 history", &targets.frames.get_written().history.view);

        passes
          .fsr
          .add_passes((&mut *graph, &mut *bindings, runtime), handles, (targets, uniform));

        Some(history)
      }
      Some(Resolve::Dlss(resolve)) => Some(self.add_dlss_pass(
        (&mut *graph, &mut *bindings),
        (passes, view),
        (resolve, handles, frame, opaque),
      )),
      Some(Resolve::RayReconstruction(resolve)) => guides.map(|guides| {
        self.add_ray_reconstruction_pass((&mut *graph, &mut *bindings), (resolve, handles, frame, guides))
      }),
      Some(Resolve::Temporal(temporal)) => {
        let TemporalResolve { history, uniform, .. } = &**temporal;
        let target: GraphTexture = bindings.import_view(&mut *graph, "temporal history", &history.get_written().view);
        let parameters: TemporalParameters = TemporalParameters {
          frame,
          depth_target: handles.depth,
          history: bindings.import_view(&mut *graph, "temporal history", &history.get_read().view),
          history_sampler: passes.temporal.get_sampler(),
          temporal: runtime.push_uniform(uniform),
          motion_target: handles.motion,
          opaque: opaque.unwrap_or(handles.scene),
        };

        graph
          .add_raster_pass("temporal")
          .parameters(&parameters)
          .color(GraphColorAttachment::new(
            target,
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
          .add_raster_pass(if stage == UpscalePass::EASU {
            "upscale"
          } else {
            "downsample"
          })
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
        .add_raster_pass("sharpen")
        .parameters(&parameters)
        .color(GraphColorAttachment::new(
          sharpened,
          wgpu::LoadOp::Clear(wgpu::Color::TRANSPARENT),
        ))
        .record(move |context| passes.upscale.record(context, UpscalePass::RCAS, &parameters));
      shown = Some(sharpened);
    }

    (frame, shown.unwrap_or(frame))
  }

  /// Adds DLSS's upscale of `frame`, the finished frame at the drawn size, through the view's depth and motion, and
  /// answers the frame it writes at the output's size.
  fn add_dlss_pass<'a>(
    &'a self,
    (graph, bindings): (&mut FrameGraph<'a>, &mut GraphBindings<'a>),
    (passes, view): (LevelPasses<'a>, &'a ViewBinding),
    (resolve, handles, frame, opaque): (&'a DlssResolve, ViewTargetHandles, GraphTexture, Option<GraphTexture>),
  ) -> GraphTexture {
    let output: GraphTexture = bindings.import_view(&mut *graph, "dlss output", &resolve.output.view);
    // Where what drew over the surfaces changed the frame, upscaled from this frame rather than the history, which
    // moves it with the surface behind it.
    let reactive: Option<GraphTexture> = opaque.map(|opaque| {
      let (width, height) = handles.size;
      let mask: GraphTexture = graph.create_texture(GraphTextureDescriptor::new_2d(
        "dlss reactive",
        width,
        height,
        DlssReactivePass::FORMAT,
      ));
      let parameters: DlssReactiveParameters = DlssReactiveParameters { frame, opaque };

      graph
        .add_raster_pass("dlss reactive")
        .parameters(&parameters)
        .color(GraphColorAttachment::new(
          mask,
          wgpu::LoadOp::Clear(wgpu::Color::TRANSPARENT),
        ))
        .record(move |context| passes.dlss_reactive.record(context, view, &parameters));

      mask
    });
    // FSR 2's sense of the jitter, which DLSS shares: a drawn texel `m` stands at `m + 0.5 - jitter`.
    let jitter: [f32; 2] = (-self.offset).to_array();
    let is_reset: bool = resolve.is_reset;

    let mut pass = graph
      .add_encoder_pass("dlss")
      .texture(frame, GraphTextureAccess::Sampled)
      .texture(handles.depth, GraphTextureAccess::Sampled)
      .texture(handles.motion, GraphTextureAccess::Sampled)
      .texture(output, GraphTextureAccess::StorageReadWrite);

    if let Some(reactive) = reactive {
      pass = pass.texture(reactive, GraphTextureAccess::Sampled);
    }

    pass.record(move |context| {
      let color = context.get_texture(frame);
      let render: (u32, u32) = (color.texture.width(), color.texture.height());
      let inputs: DlssInputs<'_> = DlssInputs {
        color: color.view,
        depth: context.get_texture(handles.depth).view,
        motion: context.get_texture(handles.motion).view,
        // The motion is kept in texture coordinates, from where a point stood to where it stands; DLSS reads render
        // pixels back to where it stood.
        motion_scale: [-(render.0 as f32), -(render.1 as f32)],
        bias_current_color: reactive.map(|reactive| context.get_texture(reactive).view),
        jitter,
        render,
        output: context.get_texture(output).view,
        is_reset,
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
    (graph, bindings): (&mut FrameGraph<'a>, &mut GraphBindings<'a>),
    (resolve, handles, frame, guides): (&'a RayReconstructionResolve, ViewTargetHandles, GraphTexture, RayGuides),
  ) -> GraphTexture {
    let output: GraphTexture = bindings.import_view(&mut *graph, "ray reconstruction output", &resolve.output.view);
    // FSR 2's sense of the jitter, which DLSS shares: a drawn texel `m` stands at `m + 0.5 - jitter`.
    let jitter: [f32; 2] = (-self.offset).to_array();
    let is_reset: bool = resolve.is_reset;

    let mut pass = graph
      .add_encoder_pass("dlss ray reconstruction")
      .texture(frame, GraphTextureAccess::Sampled)
      .texture(handles.depth, GraphTextureAccess::Sampled)
      .texture(handles.motion, GraphTextureAccess::Sampled)
      .texture(handles.albedo, GraphTextureAccess::Sampled)
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
        diffuse_albedo: context.get_texture(handles.albedo).view,
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
      };

      match resolve.reconstructor.record(context.get_encoder(), &inputs) {
        Ok(commands) => context.hand_over(commands),
        Err(error) => log::error!("{error}"),
      }
    });

    output
  }

  /// The history this frame wrote becomes the one the next reads.
  pub fn end_frame(&mut self) {
    if let Some(resolve) = &mut self.resolve {
      resolve.swap();
    }
  }
}

impl ViewFeature for ResolveFeature {
  /// The temporal resolve's histories and FSR 2's targets, and the frame at the viewport's size.
  fn get_memory(&self) -> u64 {
    let upscaled: u64 = self.upscaled.iter().flatten().map(|texture| texture.get_memory()).sum();

    self.resolve.as_ref().map_or(0, Resolve::get_memory) + upscaled
  }

  fn release(&mut self) {
    self.resolve = None;
    self.upscaled = None;
  }
}
