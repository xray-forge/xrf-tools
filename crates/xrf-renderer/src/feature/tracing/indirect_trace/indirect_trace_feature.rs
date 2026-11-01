use xrf_renderer_core::{GraphColorAttachment, GraphDeclaration, GraphTexture, GraphTextureDescriptor};

use crate::feature::lighting::bitmask::vbao::vbao_pass::VbaoPass;
use crate::feature::tracing::indirect_trace::indirect_trace_parameters::IndirectTraceParameters;
use crate::feature::tracing::indirect_trace::indirect_trace_pass::IndirectTracePass;
use crate::feature::tracing::indirect_trace::indirect_trace_uniform::IndirectTraceUniform;
use crate::feature::tracing::indirect_trace::indirect_traced::IndirectTraced;
use crate::feature::tracing::view_trace::traced_scene::TracedScene;
use crate::feature::weather::sky::sky_parameters::SkyParameters;
use crate::scene::level::feature::view_feature::ViewFeature;
use crate::scene::level::view_record::ViewRecord;

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
  /// Takes this frame's trace at the light's `intensity`, where its pass is built, DLSS ray reconstruction resolves the
  /// frame and the bitmask search gathers light, none otherwise; its noise turned by `noise_frame`.
  pub fn prepare(&mut self, intensity: Option<f32>, noise_frame: u32) {
    self.is_asked = intensity.is_some();
    self.uniform = IndirectTraceUniform::new(noise_frame, intensity.unwrap_or_default());
  }

  /// Whether it traces this frame through the scene's trace, where that is built (`traced`).
  pub fn is_traced(&self, traced: Option<&TracedScene<'_>>) -> bool {
    self.is_asked && traced.is_some()
  }

  /// Traces the indirect light where it does this frame and the scene's trace is built (`traced`), against the sky's
  /// group a missed ray reads. Answers the light and how far each ray went.
  pub fn add_passes<'a>(
    &self,
    GraphDeclaration { graph, runtime, .. }: GraphDeclaration<'_, 'a>,
    ViewRecord {
      passes,
      view,
      handles: targets,
      lit,
      ..
    }: ViewRecord<'a>,
    sky: SkyParameters<'a>,
    traced: Option<TracedScene<'a>>,
  ) -> Option<IndirectTraced> {
    let TracedScene { trace, textures } = traced.filter(|_| self.is_asked)?;
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
          .tracing
          .indirect_trace
          .record(context, view, &parameters, &sky, &trace, textures)
      });

    Some(IndirectTraced { light, hits })
  }
}

impl ViewFeature for IndirectTraceFeature {
  /// Nothing between frames: its targets are the frame's own.
  fn get_memory(&self) -> u64 {
    0
  }

  fn release(&mut self) {}
}
