use glam::{Mat4, Vec2};
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
use crate::frame::texture_copy::add_texture_copy;
use crate::frame::view_target_handles::ViewTargetHandles;
use crate::frame::view_targets::ViewTargets;
use crate::pass::fsr_uniform::FsrUniform;
use crate::pass::level_passes::LevelPasses;
use crate::pass::temporal_parameters::TemporalParameters;
use crate::pass::temporal_pass::TemporalPass;
use crate::pass::temporal_uniform::TemporalUniform;
use crate::pass::upscale_parameters::UpscaleParameters;
use crate::pass::upscale_pass::UpscalePass;
use crate::pass::upscale_uniform::UpscaleUniform;
use crate::pass::view_binding::ViewBinding;
use crate::scene::level::feature::resolve::Resolve;
use crate::scene::level::feature::temporal_resolve::TemporalResolve;
use crate::scene::level::feature::view_feature::ViewFeature;

/// How a view's frame becomes the image its viewport shows. At the drawn size: the temporal resolve (TAA, or FSR 2
/// upscaling as it resolves) with the jitter its samples are moved by. Then, at the viewport's size where the frame is
/// drawn smaller: EASU's upscale where no temporal resolve upscaled it, and RCAS's sharpening.
pub struct ResolveFeature {
  jitter: TemporalJitter,
  /// Where this frame's samples sit within their pixels, and the phases the jitter's sequence repeats over.
  offset: Vec2,
  phases: u32,
  /// The antialiasing asked for where it resolves temporally and the view shows its final frame.
  temporal: Option<RenderAntialiasing>,
  resolve: Option<Resolve>,
  upscaling: RenderUpscalingSettings,
  /// The frame at the viewport's size while it is drawn smaller: upscaled into the first, sharpened into the second.
  upscaled: Option<[FrameTexture; 2]>,
  upscale: UpscaleUniform,
  /// The last frame's unjittered view projection, which every surface's motion is measured from.
  motion_previous: Option<Mat4>,
  /// The last frame's jittered view projection, which a read of its targets reprojects through.
  view_projection_previous: Option<Mat4>,
  /// Frames a temporal resolve gathered, which the screen-space effects turn their noise by.
  noise_frame: u32,
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
      upscaled: None,
      upscale: UpscaleUniform::default(),
      motion_previous: None,
      view_projection_previous: None,
      noise_frame: 0,
    }
  }
}

impl ResolveFeature {
  /// Where the next frame's samples sit within their pixels, in drawn pixels, `y` down, for a viewport `ratio` times
  /// the drawn size: jittered while it resolves frames temporally and shows the finished frame, still otherwise.
  pub fn next_jitter(&mut self, options: &RenderViewOptions, ratio: f32) -> Vec2 {
    self.temporal = (options.features.antialiasing.is_temporal() && options.mode.debug_view == RenderDebugView::Final)
      .then_some(options.features.antialiasing);
    self.phases = TemporalJitter::get_phases(ratio);
    self.offset = if self.temporal.is_some() {
      self.jitter.next(ratio)
    } else {
      Vec2::ZERO
    };

    self.offset
  }

  /// Takes this frame's resolve for targets `render` texels across, none before they are made, shown in `output`:
  /// the temporal resolve's history made or kept and what it reads, and the frame at the viewport's size while it is
  /// drawn smaller.
  pub fn prepare(
    &mut self,
    device: &wgpu::Device,
    view: &CameraView,
    (render, output): (Option<(u32, u32)>, RenderRect),
    (upscaling, is_linear): (RenderUpscalingSettings, bool),
  ) {
    self.upscaling = upscaling;

    if self.temporal.is_some() {
      self.noise_frame = self.noise_frame.wrapping_add(1);
    }

    let display: (u32, u32) = (output.width.max(1), output.height.max(1));

    match (render, self.temporal) {
      (Some(render), Some(RenderAntialiasing::Fsr2)) => {
        self.prepare_fsr(device, view, (render, display), is_linear);
      }
      (Some(render), Some(_)) => self.prepare_temporal(device, view, (render, display)),
      _ => self.resolve = None,
    }

    let Some(render) = render else {
      return;
    };

    if render == (output.width, output.height) {
      self.upscaled = None;
    } else if self
      .upscaled
      .as_ref()
      .is_none_or(|[upscaled, _]| upscaled.get_size() != (output.width, output.height))
    {
      self.upscaled = Some(["upscaled", "sharpened"].map(|label| {
        FrameTexture::new(
          device,
          label,
          (output.width, output.height),
          UpscalePass::FORMAT,
          wgpu::TextureUsages::COPY_DST,
        )
      }));
    }

    self.upscale = UpscaleUniform {
      output_size: Vec2::new(output.width as f32, output.height as f32),
      sharpness: upscaling.get_sharpness(),
      linear: f32::from(u8::from(is_linear)),
    };
  }

  /// TAA's history at the viewport's size, kept while it still fits, and what it reads: this frame's view projection,
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

  /// Keeps the frame as it stands before the water and the blended surfaces draw, which the temporal resolve's reactive
  /// weight compares with: a copy for TAA, answered, or FSR 2's own, which its passes read.
  pub fn add_reactive_passes<'a>(
    &'a self,
    (graph, bindings): (&mut FrameGraph<'a>, &mut GraphBindings<'a>),
    (scene, (width, height)): (GraphTexture, (u32, u32)),
  ) -> Option<GraphTexture> {
    match &self.resolve {
      Some(Resolve::Temporal(_)) => {
        let opaque: GraphTexture = graph.create_texture(GraphTextureDescriptor::new_2d(
          "temporal opaque",
          width,
          height,
          ViewTargets::SCENE,
        ));

        add_texture_copy(&mut *graph, "temporal opaque", scene, opaque);

        Some(opaque)
      }
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

  /// Adds the resolve of `frame`, the finished frame at the drawn size, against the frame as it stood before the
  /// blended surfaces (`opaque`, the scene where none was kept). At the drawn size, the temporal resolve; then, at the
  /// viewport's, the upscale where nothing upscaled it and the sharpening. Answers the frame at the drawn size, which
  /// the present distorts, and the image the viewport shows.
  pub fn add_passes<'a>(
    &'a self,
    (graph, bindings, runtime): (&mut FrameGraph<'a>, &mut GraphBindings<'a>, &mut GraphRuntime),
    passes: LevelPasses<'a>,
    view: &'a ViewBinding,
    (handles, frame, opaque): (ViewTargetHandles, GraphTexture, Option<GraphTexture>),
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
    let (frame, mut shown): (GraphTexture, Option<GraphTexture>) = match (resolved, upscaled) {
      (Some(resolved), Some(_)) => (frame, Some(resolved)),
      (Some(resolved), None) => (resolved, None),
      (None, Some([upscaled, _])) => {
        let parameters: UpscaleParameters = UpscaleParameters { source: frame, upscale };

        graph
          .add_raster_pass("upscale")
          .parameters(&parameters)
          .color(GraphColorAttachment::new(
            upscaled,
            wgpu::LoadOp::Clear(wgpu::Color::TRANSPARENT),
          ))
          .record(move |context| passes.upscale.record(context, 0, &parameters));

        (frame, Some(upscaled))
      }
      (None, None) => (frame, None),
    };

    if let (true, Some([upscaled, sharpened])) = (self.upscaling.is_sharpened(), upscaled) {
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
        .record(move |context| passes.upscale.record(context, 1, &parameters));
      shown = Some(sharpened);
    }

    (frame, shown.unwrap_or(frame))
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
