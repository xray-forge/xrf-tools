use xrf_renderer_core::{GraphColorAttachment, GraphDeclaration, GraphTexture, GraphTextureDescriptor};

use crate::frame::view_target_handles::ViewTargetHandles;
use crate::pass::indirect_trace_parameters::IndirectTraceParameters;
use crate::pass::indirect_trace_pass::IndirectTracePass;
use crate::pass::indirect_trace_uniform::IndirectTraceUniform;
use crate::pass::level_passes::LevelPasses;
use crate::pass::sky_parameters::SkyParameters;
use crate::pass::trace_parameters::TraceParameters;
use crate::pass::vbao_pass::VbaoPass;
use crate::pass::view_binding::ViewBinding;
use crate::scene::level::feature::view_feature::ViewFeature;
use crate::scene::level::lighting_handles::LightingHandles;

/// The indirect light traced in the screen-space bounce's place, where DLSS ray reconstruction denoises the frame and
/// the bitmask search gathers light: one ray a pixel at the frame's size, as ray reconstruction takes its noise, lit
/// where it meets something as the reflections light a hit, written as the bounce's filter writes its light.
#[derive(Default)]
pub struct IndirectTraceFeature {
  /// Whether it traces this frame, the scene's trace allowing.
  is_asked: bool,
  uniform: IndirectTraceUniform,
}

impl IndirectTraceFeature {
  /// Takes this frame's trace, `is_asked` where its pass is built, DLSS ray reconstruction resolves the frame and the
  /// bitmask search gathers light, at the light's `intensity`, its noise turned by `noise_frame`.
  pub fn prepare(&mut self, is_asked: bool, (intensity, noise_frame): (f32, u32)) {
    self.is_asked = is_asked;
    self.uniform = IndirectTraceUniform::new(noise_frame, intensity);
  }

  /// Whether it traces this frame where the scene's trace is built (`is_scene_traced`).
  pub fn is_traced(&self, is_scene_traced: bool) -> bool {
    self.is_asked && is_scene_traced
  }

  /// Traces the indirect light where it does this frame and the scene's trace is built (`traced`), against the sky's
  /// group a missed ray reads. Answers the light and how far each ray went.
  pub fn add_passes<'a>(
    &self,
    GraphDeclaration { graph, runtime, .. }: GraphDeclaration<'_, 'a>,
    passes: LevelPasses<'a>,
    view: &'a ViewBinding,
    (targets, lit, sky): (ViewTargetHandles, LightingHandles<'a>, SkyParameters<'a>),
    traced: Option<(TraceParameters<'a>, &'a wgpu::BindGroup)>,
  ) -> Option<(GraphTexture, GraphTexture)> {
    let (trace, textures) = traced.filter(|_| self.is_asked)?;
    let (width, height) = targets.size;
    let light: GraphTexture = graph.create_texture(GraphTextureDescriptor::new_2d(
      "indirect traced",
      width,
      height,
      VbaoPass::LIGHT,
    ));
    let hits: GraphTexture = graph.create_texture(GraphTextureDescriptor::new_2d(
      "indirect traced hits",
      width,
      height,
      IndirectTracePass::HITS,
    ));
    let parameters: IndirectTraceParameters<'a> = IndirectTraceParameters {
      normal_target: targets.normal,
      depth_target: targets.depth,
      material_target: targets.material,
      lighting: lit.lighting,
      material_lut: lit.material_lut,
      lut_sampler: lit.lut_sampler,
      indirect: runtime.push_uniform(&self.uniform),
    };

    graph
      .add_raster_pass("indirect trace")
      .parameters(&parameters)
      .parameters(&sky)
      .parameters(&trace)
      .color(GraphColorAttachment::new(
        light,
        wgpu::LoadOp::Clear(wgpu::Color::TRANSPARENT),
      ))
      .color(GraphColorAttachment::new(
        hits,
        wgpu::LoadOp::Clear(wgpu::Color::TRANSPARENT),
      ))
      .record(move |context| {
        passes
          .indirect_trace
          .record(context, view, (&parameters, &sky), (&trace, textures))
      });

    Some((light, hits))
  }
}

impl ViewFeature for IndirectTraceFeature {
  /// Nothing between frames: its targets are the frame's own.
  fn get_memory(&self) -> u64 {
    0
  }

  fn release(&mut self) {}
}
