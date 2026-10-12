use xrf_renderer_core::{
  FrameGraph, GraphBindings, GraphColorAttachment, GraphRuntime, GraphTexture, GraphTextureDescriptor, UniformBinding,
};

use crate::contract::render_antialiasing::RenderAntialiasing;
use crate::frame::view_targets::ViewTargets;
use crate::pass::fxaa_parameters::FxaaParameters;
use crate::pass::level_passes::LevelPasses;
use crate::pass::smaa_parameters::SmaaParameters;
use crate::pass::smaa_pass::SmaaPass;
use crate::pass::smoothing_uniform::SmoothingUniform;
use crate::scene::level::feature::view_feature::ViewFeature;

/// The smoothing pass over the frame as drawn, FXAA or SMAA, while one smooths it.
#[derive(Default)]
pub struct SmoothingFeature {
  mode: Option<RenderAntialiasing>,
  /// What its passes read: whether the frame holds linear light.
  uniform: SmoothingUniform,
}

impl SmoothingFeature {
  /// Takes this frame's smoothing, where the antialiasing asked for is FXAA or SMAA and the view shows its final
  /// frame (`is_shown`); SMAA whose lookup textures could not be read smooths as FXAA does, which needs none.
  pub fn prepare(&mut self, passes: LevelPasses<'_>, mode: RenderAntialiasing, (is_shown, is_linear): (bool, bool)) {
    self.uniform = SmoothingUniform {
      linear: f32::from(u8::from(is_linear)),
    };

    let mode: RenderAntialiasing = match (mode, passes.smaa) {
      (RenderAntialiasing::Smaa, None) => RenderAntialiasing::Fxaa,
      _ => mode,
    };

    self.mode = (is_shown && matches!(mode, RenderAntialiasing::Fxaa | RenderAntialiasing::Smaa)).then_some(mode);
  }

  /// The pass smoothing the frame, where one does.
  pub fn get_mode(&self) -> Option<RenderAntialiasing> {
    self.mode
  }

  /// Adds the smoothing of `frame`, `width` by `height`, answering the target holding the smoothed frame; the frame
  /// itself where nothing smooths it.
  pub fn add_passes<'a>(
    &self,
    (graph, bindings, runtime): (&mut FrameGraph<'a>, &mut GraphBindings<'a>, &mut GraphRuntime),
    passes: LevelPasses<'a>,
    (frame, (width, height)): (GraphTexture, (u32, u32)),
  ) -> GraphTexture {
    let Some(mode) = self.mode else {
      return frame;
    };
    let smoothing: UniformBinding<SmoothingUniform> = runtime.push_uniform(&self.uniform);
    let sized =
      |label: &'static str, format: wgpu::TextureFormat| GraphTextureDescriptor::new_2d(label, width, height, format);
    let target: GraphTexture = graph.create_texture(sized("smoothed", ViewTargets::SCENE));

    match (mode, passes.smaa) {
      (RenderAntialiasing::Smaa, Some(pass)) => {
        let edges: GraphTexture = graph.create_texture(sized("smaa edges", SmaaPass::TARGET_FORMAT));
        let weights: GraphTexture = graph.create_texture(sized("smaa weights", SmaaPass::TARGET_FORMAT));
        let [area, search, empty] = pass.get_lookups();
        let [area, search] = [("smaa area", area), ("smaa search", search)]
          .map(|(label, view)| bindings.import_view(&mut *graph, label, view));
        // Each stage reads an empty texture where it would read what it writes or what comes after it.
        let [no_edges, no_weights] =
          ["smaa no edges", "smaa no weights"].map(|label| bindings.import_view(&mut *graph, label, empty));
        let [linear_sampler, point_sampler] = pass.get_samplers();

        for (stage, (name, (edges_texture, weights_texture), written)) in [
          ("smaa edges", (no_edges, no_weights), edges),
          ("smaa weights", (edges, no_weights), weights),
          ("smaa", (edges, weights), target),
        ]
        .into_iter()
        .enumerate()
        {
          let parameters: SmaaParameters = SmaaParameters {
            source: frame,
            smoothing,
            edges_texture,
            weights_texture,
            area_texture: area,
            search_texture: search,
            linear_sampler,
            point_sampler,
          };

          graph
            .add_raster_pass(name)
            .parameters(&parameters)
            .color(GraphColorAttachment::new(
              written,
              wgpu::LoadOp::Clear(wgpu::Color::TRANSPARENT),
            ))
            .record(move |context| pass.record(context, stage, &parameters));
        }
      }
      _ => {
        let parameters: FxaaParameters = FxaaParameters {
          frame,
          smoothing,
          frame_sampler: passes.fxaa.get_sampler(),
        };

        graph
          .add_raster_pass("fxaa")
          .parameters(&parameters)
          .color(GraphColorAttachment::new(
            target,
            wgpu::LoadOp::Clear(wgpu::Color::TRANSPARENT),
          ))
          .record(move |context| passes.fxaa.record(context, &parameters));
      }
    }

    target
  }
}

impl ViewFeature for SmoothingFeature {
  /// Its targets are the frame's own.
  fn get_memory(&self) -> u64 {
    0
  }

  fn release(&mut self) {
    *self = Self::default();
  }
}
