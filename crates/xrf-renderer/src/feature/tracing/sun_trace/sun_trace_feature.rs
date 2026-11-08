use xrf_renderer_core::{GraphColorAttachment, GraphDeclaration, GraphTexture, GraphTextureDescriptor, UniformBinding};

use crate::contract::view::render_view_options::RenderViewOptions;
use crate::feature::image::resolve::resolve_decisions::ResolveDecisions;
use crate::feature::tracing::sun_trace::sun_trace_parameters::SunTraceParameters;
use crate::feature::tracing::sun_trace::sun_trace_pass::SunTracePass;
use crate::feature::tracing::sun_trace::sun_trace_uniform::SunTraceUniform;
use crate::feature::tracing::view_trace::traced_scene::TracedScene;
use crate::feature::view::view_feature::ViewFeature;
use crate::frame::targets::view_target_handles::ViewTargetHandles;
use crate::pass::frame_lighting::lighting_uniform::LightingUniform;
use crate::pass::level_passes::LevelPasses;
use crate::pass::view::view_binding::ViewBinding;

/// The sun's shadow traced, where DLSS ray reconstruction resolves a lit, solid frame and the scene's trace is built:
/// one ray a pixel towards the sun's disc, raw, which the sun's pass takes in its cascades' place.
#[derive(Default)]
pub struct SunTraceFeature {
  /// Whether it traces this frame, the scene's trace allowing.
  is_traced: bool,
  uniform: SunTraceUniform,
}

impl SunTraceFeature {
  /// Takes this frame's trace towards the sun as the frame's `lighting` gives it, where DLSS ray reconstruction resolves
  /// the frame, its noise turned as the `resolve` decided.
  pub fn prepare(&mut self, options: &RenderViewOptions, lighting: &LightingUniform, resolve: ResolveDecisions) {
    self.is_traced = Self::is_asked(options, resolve);
    self.uniform = SunTraceUniform::new(lighting.to_sun, resolve.noise_frame);
  }

  /// Whether a view traces the sun's shadow, the scene's trace allowing: where DLSS ray reconstruction resolves its
  /// frame, as its `resolve` decided, lit and solid.
  pub fn is_asked(options: &RenderViewOptions, resolve: ResolveDecisions) -> bool {
    resolve.is_ray_reconstructed && options.mode.is_lit && !options.mode.is_wireframe
  }

  /// Traces the sun's shadow where it does this frame and the scene's trace is built (`traced`), answering what the sun
  /// sees at each pixel; none otherwise, the sun's cascades shadowing the frame.
  pub fn add_passes<'a>(
    &self,
    GraphDeclaration { graph, runtime, .. }: GraphDeclaration<'_, 'a>,
    passes: LevelPasses<'a>,
    view: &'a ViewBinding,
    targets: ViewTargetHandles,
    traced: Option<TracedScene<'a>>,
  ) -> Option<GraphTexture> {
    let TracedScene { trace, textures } = traced.filter(|_| self.is_traced)?;
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
      .record(move |context| {
        passes
          .tracing
          .sun_trace
          .record(context, view, &parameters, &trace, textures);
      });

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
