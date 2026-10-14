use glam::Vec4;
use xrf_renderer_core::{
  FrameGraph, GraphColorAttachment, GraphRuntime, GraphTexture, GraphTextureDescriptor, UniformBinding,
};

use crate::contract::render_view_options::RenderViewOptions;
use crate::frame::view_target_handles::ViewTargetHandles;
use crate::pass::level_passes::LevelPasses;
use crate::pass::sun_trace_parameters::SunTraceParameters;
use crate::pass::sun_trace_pass::SunTracePass;
use crate::pass::sun_trace_uniform::SunTraceUniform;
use crate::pass::trace_parameters::TraceParameters;
use crate::pass::view_binding::ViewBinding;
use crate::scene::level::feature::view_feature::ViewFeature;

/// The sun's shadow traced, where DLSS ray reconstruction resolves a lit, solid frame and the scene's trace is built:
/// one ray a pixel towards the sun's disc, raw, which the sun's pass takes in its cascades' place.
#[derive(Default)]
pub struct SunTraceFeature {
  /// Whether it traces this frame, the scene's trace allowing.
  is_traced: bool,
  uniform: SunTraceUniform,
}

impl SunTraceFeature {
  /// Takes this frame's trace towards the sun's view space direction `to_sun`, its noise turned by `noise_frame`,
  /// where DLSS ray reconstruction resolves the frame (`is_reconstructed`).
  pub fn prepare(&mut self, options: &RenderViewOptions, to_sun: Vec4, noise_frame: u32, is_reconstructed: bool) {
    self.is_traced = Self::is_asked(options, is_reconstructed);
    self.uniform = SunTraceUniform::new(to_sun, noise_frame);
  }

  /// Whether a view traces the sun's shadow, the scene's trace allowing: where DLSS ray reconstruction resolves its
  /// frame (`is_reconstructed`), lit and solid.
  pub fn is_asked(options: &RenderViewOptions, is_reconstructed: bool) -> bool {
    is_reconstructed && options.mode.is_lit && !options.mode.is_wireframe
  }

  /// Traces the sun's shadow where it does this frame and the scene's trace is built (`traced`), answering what the sun
  /// sees at each pixel; none otherwise, the sun's cascades shadowing the frame.
  pub fn add_passes<'a>(
    &self,
    (graph, runtime): (&mut FrameGraph<'a>, &mut GraphRuntime),
    passes: LevelPasses<'a>,
    view: &'a ViewBinding,
    targets: ViewTargetHandles,
    traced: Option<(TraceParameters<'a>, &'a wgpu::BindGroup)>,
  ) -> Option<GraphTexture> {
    let (trace, textures) = traced.filter(|_| self.is_traced)?;
    let sun_trace: UniformBinding<SunTraceUniform> = runtime.push_uniform(&self.uniform);
    let (width, height) = targets.size;
    let seen: GraphTexture = graph.create_texture(GraphTextureDescriptor::new_2d(
      "sun traced",
      width,
      height,
      SunTracePass::FORMAT,
    ));
    let parameters: SunTraceParameters = SunTraceParameters {
      normal_target: targets.normal,
      material_target: targets.material,
      depth_target: targets.depth,
      sun_trace,
    };

    graph
      .add_raster_pass("sun trace")
      .parameters(&parameters)
      .parameters(&trace)
      .color(GraphColorAttachment::new(seen, wgpu::LoadOp::Clear(wgpu::Color::WHITE)))
      .record(move |context| passes.sun_trace.record(context, view, (&parameters, &trace, textures)));

    Some(seen)
  }
}

impl ViewFeature for SunTraceFeature {
  /// Its target is the frame's own.
  fn get_memory(&self) -> u64 {
    0
  }

  fn release(&mut self) {}
}
