use glam::Mat4;
use xrf_renderer_core::{FrameGraph, GraphBindings, GraphColorAttachment, GraphRuntime, GraphTexture, UniformBinding};

use crate::contract::render_ambient_occlusion_quality::RenderAmbientOcclusionQuality;
use crate::contract::render_ambient_occlusion_settings::RenderAmbientOcclusionSettings;
use crate::contract::render_view_options::RenderViewOptions;
use crate::frame::view_target_handles::ViewTargetHandles;
use crate::pass::ambient_occlusion_parameters::AmbientOcclusionParameters;
use crate::pass::ambient_occlusion_pass::AmbientOcclusionPass;
use crate::pass::ambient_occlusion_uniform::AmbientOcclusionUniform;
use crate::pass::level_passes::LevelPasses;
use crate::pass::view_binding::ViewBinding;
use crate::scene::level::feature::view_feature::ViewFeature;

/// The ambient occlusion at half the frame's size, which combine multiplies the hemisphere and ambient light by: GTAO's
/// own search and denoise here, or VBAO's in the bitmask search's place.
#[derive(Default)]
pub struct AmbientOcclusionFeature {
  settings: RenderAmbientOcclusionSettings,
  /// Whether the view draws it at all: lit, solid and asked for.
  is_drawn: bool,
  uniform: AmbientOcclusionUniform,
}

impl AmbientOcclusionFeature {
  /// Takes this frame's occlusion, searched over `size`, half the frame's, through `projection`.
  pub fn prepare(&mut self, options: &RenderViewOptions, projection: Mat4, size: (u32, u32)) {
    self.settings = options.features.ambient_occlusion;
    self.is_drawn = options.mode.is_lit && !options.mode.is_wireframe && self.settings.is_enabled;
    self.uniform = AmbientOcclusionUniform::new(&self.settings, projection, size);
  }

  /// Whether the view draws an ambient occlusion this frame, GTAO's or VBAO's.
  pub fn is_drawn(&self) -> bool {
    self.is_drawn
  }

  pub fn get_quality(&self) -> RenderAmbientOcclusionQuality {
    self.settings.quality
  }

  /// Adds GTAO's search and its denoise to and fro between the occlusion targets, the first holding it finished;
  /// nothing where it is not drawn or VBAO searches in its place.
  pub fn add_passes<'a>(
    &self,
    (graph, bindings, runtime): (&mut FrameGraph<'a>, &mut GraphBindings<'a>, &mut GraphRuntime),
    passes: LevelPasses<'a>,
    view: &'a ViewBinding,
    targets: ViewTargetHandles,
  ) {
    if !self.is_drawn || self.settings.is_vbao() {
      return;
    }

    let quality: RenderAmbientOcclusionQuality = self.settings.quality;
    let occlusion: UniformBinding<AmbientOcclusionUniform> = runtime.push_uniform(&self.uniform);
    // The search reads no source, but binds one: a texel of nothing, as neither target is written yet.
    let unsearched: GraphTexture =
      bindings.import_view(&mut *graph, "ambient occlusion unsearched", passes.vbao.get_dark());

    for (stage, (name, (read, written))) in ["ambient occlusion", "occlusion denoise", "occlusion denoise back"]
      .into_iter()
      .zip(AmbientOcclusionPass::STAGES)
      .enumerate()
    {
      // Each denoise reads the other target than it draws into, with the normals and the depth.
      let parameters: AmbientOcclusionParameters = AmbientOcclusionParameters {
        normal_target: targets.normal,
        depth_target: targets.depth,
        occlusion,
        source: if stage == 0 {
          unsearched
        } else {
          targets.occlusion[1 - read]
        },
      };

      graph
        .add_raster_pass(name)
        .parameters(&parameters)
        .color(GraphColorAttachment::new(
          targets.occlusion[written],
          wgpu::LoadOp::Clear(wgpu::Color::TRANSPARENT),
        ))
        .record(move |context| {
          passes
            .ambient_occlusion
            .record(context, (stage, quality), view, &parameters)
        });
    }
  }
}

impl ViewFeature for AmbientOcclusionFeature {
  /// Its targets are the view's occlusion pair.
  fn get_memory(&self) -> u64 {
    0
  }

  fn release(&mut self) {}
}
