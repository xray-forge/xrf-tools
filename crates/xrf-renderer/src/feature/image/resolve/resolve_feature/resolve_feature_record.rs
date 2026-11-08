use std::time::Duration;

use xrf_dlss::{DlssInputs, DlssRayInputs};
use xrf_renderer_core::{
  FrameGraph, GraphColorAttachment, GraphDeclaration, GraphTexture, GraphTextureAccess, GraphTextureDescriptor,
  UniformBinding,
};

use crate::contract::view::output::render_debug_view::RenderDebugView;
use crate::feature::image::image_chain::ImageChain;
use crate::feature::image::resolve::before_water_readers::BeforeWaterReaders;
use crate::feature::image::resolve::dlss::dlss_resolve::DlssResolve;
use crate::feature::image::resolve::dlss::ray_reconstruction_resolve::RayReconstructionResolve;
use crate::feature::image::resolve::fsr::targets::fsr_targets::FsrTargets;
use crate::feature::image::resolve::resolve::Resolve;
use crate::feature::image::resolve::resolve_feature::ResolveFeature;
use crate::feature::image::resolve::temporal::temporal_parameters::TemporalParameters;
use crate::feature::image::resolve::temporal::temporal_resolve::TemporalResolve;
use crate::feature::image::resolve::upscale::upscale_parameters::UpscaleParameters;
use crate::feature::image::resolve::upscale::upscale_pass::UpscalePass;
use crate::feature::image::resolve::upscale::upscale_stage::UpscaleStage;
use crate::feature::image::resolve::upscale::upscale_uniform::UpscaleUniform;
use crate::feature::tracing::ray_guides::ray_guide_inputs::RayGuideInputs;
use crate::feature::tracing::ray_guides::ray_guides::RayGuides;
use crate::feature::tracing::ray_guides::ray_guides_parameters::RayGuidesParameters;
use crate::feature::tracing::ray_guides::ray_guides_pass::RayGuidesPass;
use crate::frame::jitter_guard::JitterGuard;
use crate::frame::targets::texture_copy::add_texture_copy;
use crate::frame::targets::view_target_handles::ViewTargetHandles;
use crate::frame::targets::view_targets::ViewTargets;
use crate::scene::level::view::record::view_record::ViewRecord;

impl ResolveFeature {
  /// Whether the temporal resolve's reactive weight reads the frame's copy as it stood before the water.
  fn is_reading_scene_before_water(&self) -> bool {
    matches!(self.resolve, Some(Resolve::Temporal(_)))
  }

  /// Copies the frame as it stands before the water, the composited surfaces and the particles draw, once for all that
  /// read it: the temporal resolve's reactive weight is what the rest change of it, and the `readers` besides. None
  /// where nothing reads it.
  pub(crate) fn add_before_water(
    &self,
    graph: &mut FrameGraph<'_>,
    handles: ViewTargetHandles,
    readers: BeforeWaterReaders,
  ) -> Option<GraphTexture> {
    (self.is_reading_scene_before_water() || readers.is_guided || readers.is_water_reading).then(|| {
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
  pub(crate) fn add_transparency_guide(
    graph: &mut FrameGraph<'_>,
    handles: ViewTargetHandles,
    guides: &mut Option<RayGuides>,
    before_water: Option<GraphTexture>,
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
  pub(crate) fn add_reactive_passes<'a>(
    &'a self,
    GraphDeclaration { graph, bindings, .. }: GraphDeclaration<'_, 'a>,
    scene: GraphTexture,
    before_water: Option<GraphTexture>,
  ) -> Option<GraphTexture> {
    match &self.resolve {
      Some(Resolve::Temporal(_)) => before_water,
      Some(Resolve::Dlss(_) | Resolve::RayReconstruction(_)) => None,
      Some(Resolve::Fsr { targets, .. }) => {
        let opaque: GraphTexture = bindings.import_view(&mut *graph, "fsr opaque", &targets.opaque.view);

        add_texture_copy(graph, "fsr opaque", scene, opaque);

        None
      }
      None => None,
    }
  }

  /// Adds the guides DLSS ray reconstruction reads, where it resolves this frame, at the drawn size: worked out from
  /// the G-buffer and `inputs`, the record's stand-ins for the reflections' trace and the indirect light's hit
  /// distances where they are not drawn, after combine.
  pub(crate) fn add_guide_passes<'a>(
    &self,
    graph: &mut FrameGraph<'a>,
    ViewRecord {
      passes,
      view,
      handles,
      lit,
      unlit: none,
      untraced,
      ..
    }: ViewRecord<'a>,
    RayGuideInputs {
      wet,
      reflections,
      indirect,
    }: RayGuideInputs,
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
      wet_surface: wet.surface,
      wet_reach: wet.reach,
      lighting: lit.lighting,
      traced: reflections.map_or(untraced, |reflections| reflections.trace),
      diffuse_hits: indirect.map_or(none, |indirect| indirect.hits),
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
      .record(move |context| passes.tracing.ray_guides.record(context, view, &parameters));

    Some(RayGuides {
      diffuse_hit_distance: indirect.map(|_| diffuse_hit_distance),
      ..guides
    })
  }

  /// Adds the resolve of `frame`, the finished frame at the drawn size, against the frame as it stood before the
  /// blended surfaces (`opaque`, the scene where none was kept), DLSS's ray reconstruction reading `guides`. At the
  /// drawn size, the temporal resolve; then, at the viewport's, the upscale where nothing upscaled it and the
  /// sharpening. Answers the frame at the drawn size and the image the viewport shows, which the layers after the
  /// resolve are laid over.
  pub(crate) fn add_passes<'a>(
    &'a self,
    GraphDeclaration {
      graph,
      bindings,
      runtime,
    }: GraphDeclaration<'_, 'a>,
    ViewRecord {
      passes, view, handles, ..
    }: ViewRecord<'a>,
    frame: GraphTexture,
    opaque: Option<GraphTexture>,
    guides: Option<RayGuides>,
  ) -> ImageChain {
    // The frames at the viewport's size are this frame's own, read by its passes alone, so the graph pools them with
    // its other textures of their kind.
    let mut shown_texture = |label: &'static str, is_written: bool| {
      self.shown_size.filter(|_| is_written).map(|(width, height)| {
        graph.create_texture(GraphTextureDescriptor::new_2d(
          label,
          width,
          height,
          UpscalePass::FORMAT,
        ))
      })
    };
    let upscaled: Option<GraphTexture> = shown_texture("upscaled", self.is_upscaled);
    let sharpened: Option<GraphTexture> = shown_texture("sharpened", self.is_sharpened);
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

        passes.image.fsr.add_passes(
          GraphDeclaration::new(graph, bindings, runtime),
          handles,
          targets,
          uniform,
          debug,
        );

        Some(debug.map_or(history, |(_, shown)| shown))
      }
      Some(Resolve::Dlss(resolve)) => {
        Some(self.add_dlss_pass(GraphDeclaration::new(graph, bindings, runtime), resolve, handles, frame))
      }
      Some(Resolve::RayReconstruction(resolve)) => guides.map(|guides| {
        self.add_ray_reconstruction_pass(
          GraphDeclaration::new(graph, bindings, runtime),
          resolve,
          handles,
          frame,
          guides,
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
        let parameters: TemporalParameters<'_> = TemporalParameters {
          frame,
          depth_target: handles.depth,
          history: bindings.import_view(&mut *graph, "temporal history", &history.get_read().view),
          history_sampler: passes.image.temporal.get_sampler(),
          temporal: runtime.push_uniform(uniform),
          motion_target: handles.motion,
          opaque: opaque.unwrap_or(handles.scene),
          flicker_history: bindings.import_view(&mut *graph, "temporal flicker", &flicker.get_read().view),
        };

        graph
          .add_raster_pass(Self::TEMPORAL)
          .tag(JitterGuard::RESOLVES)
          .parameters(&parameters)
          .color(GraphColorAttachment::new(
            target,
            wgpu::LoadOp::Clear(wgpu::Color::TRANSPARENT),
          ))
          .color(GraphColorAttachment::new(
            flickered,
            wgpu::LoadOp::Clear(wgpu::Color::TRANSPARENT),
          ))
          .record(move |context| passes.image.temporal.record(context, view, &parameters));

        Some(target)
      }
      None => None,
    };
    // A resolve that drew nothing this frame (ray reconstruction before its guides stand) leaves the frame at the drawn
    // size, which the viewport's rescale then takes in its place.
    let rescale: Option<UpscaleStage> = match (self.rescale, resolved) {
      (None, None) if self.resolve.is_some() => Self::get_rescale(handles.size, self.get_shown_size(handles.size)),
      (rescale, _) => rescale,
    };
    let (frame, mut shown): (GraphTexture, Option<GraphTexture>) = match (rescale, upscaled, resolved) {
      (Some(stage), Some(upscaled), resolved) => {
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
          .record(move |context| passes.image.upscale.record(context, stage, &parameters));

        (frame, Some(upscaled))
      }
      (_, _, Some(resolved)) if self.is_rescaled() => (frame, Some(resolved)),
      (_, _, Some(resolved)) => (resolved, None),
      _ => (frame, None),
    };

    // The sharpening reads what this frame wrote: the upscaled or resolved image, else the frame at the viewport's size.
    if let (true, Some(sharpened)) = (self.is_sharpened, sharpened) {
      let parameters: UpscaleParameters = UpscaleParameters {
        source: shown.unwrap_or(frame),
        upscale,
      };

      graph
        .add_raster_pass(UpscaleStage::Rcas.get_name())
        .parameters(&parameters)
        .color(GraphColorAttachment::new(
          sharpened,
          wgpu::LoadOp::Clear(wgpu::Color::TRANSPARENT),
        ))
        .record(move |context| passes.image.upscale.record(context, UpscaleStage::Rcas, &parameters));
      shown = Some(sharpened);
    }

    ImageChain {
      resolved: frame,
      shown: shown.unwrap_or(frame),
      shown_size: self.get_shown_size(handles.size),
      is_rescaled: self.is_rescaled(),
    }
  }

  /// Adds DLSS's upscale of `frame`, the finished frame at the drawn size, through the view's depth and motion, and
  /// answers the frame it writes at the output's size. It is given no mask of what to take from this frame alone:
  /// the transformer presets ignore one.
  fn add_dlss_pass<'a>(
    &'a self,
    GraphDeclaration { graph, bindings, .. }: GraphDeclaration<'_, 'a>,
    resolve: &'a DlssResolve,
    handles: ViewTargetHandles,
    frame: GraphTexture,
  ) -> GraphTexture {
    let output: GraphTexture = bindings.import_view(&mut *graph, "dlss output", &resolve.output.view);
    // FSR's sense of the jitter, which DLSS shares: a drawn texel `m` stands at `m + 0.5 - jitter`.
    let jitter: [f32; 2] = (-self.offset).to_array();
    let is_reset: bool = resolve.is_reset;
    let frame_time: Duration = self.frame_time;
    let pre_exposure: f32 = self.pre_exposure.current;

    graph
      .add_encoder_pass(Self::DLSS)
      .tag(JitterGuard::RESOLVES)
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
    resolve: &'a RayReconstructionResolve,
    handles: ViewTargetHandles,
    frame: GraphTexture,
    guides: RayGuides,
  ) -> GraphTexture {
    let output: GraphTexture = bindings.import_view(&mut *graph, "ray reconstruction output", &resolve.output.view);
    // FSR's sense of the jitter, which DLSS shares: a drawn texel `m` stands at `m + 0.5 - jitter`.
    let jitter: [f32; 2] = (-self.offset).to_array();
    let is_reset: bool = resolve.is_reset;
    let pre_exposure: f32 = self.pre_exposure.current;

    let mut pass = graph
      .add_encoder_pass(Self::RAY_RECONSTRUCTION)
      .tag(JitterGuard::RESOLVES)
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
