use xrf_renderer_core::{GraphColorAttachment, GraphDeclaration, GraphTexture, GraphTextureDescriptor};

use crate::contract::view::render_view_options::RenderViewOptions;
use crate::contract::viewport::render_rect::RenderRect;
use crate::feature::effects::distort::distort_parameters::DistortParameters;
use crate::feature::effects::distort::distort_uniform::DistortUniform;
use crate::feature::image::present::present_frame::PresentFrame;
use crate::feature::image::present::present_inputs::PresentInputs;
use crate::feature::image::present::present_uniform::PresentUniform;
use crate::feature::view::view_feature::ViewFeature;
use crate::frame::targets::screen_tiles::ScreenTiles;
use crate::frame::targets::texture_copy::add_texture_rects_copy;
use crate::frame::targets::view_target_handles::ViewTargetHandles;
use crate::frame::targets::view_targets::ViewTargets;
use crate::pass::level_passes::LevelPasses;
use crate::pass::view::view_binding::ViewBinding;

/// What the window pass presents the view's finished frame by: the debug view, the distortion, the corrections, the
/// bloom, the selection's outline and the tonemap where the scene holds linear light.
#[derive(Default)]
pub(crate) struct PresentFeature {
  uniform: PresentUniform,
  /// The distortion's strength and blur, and the tiles its target was written in; none where nothing wrote it.
  distortion: Option<(DistortUniform, ScreenTiles)>,
}

impl PresentFeature {
  /// Prepares this frame's present into `output`, once every other unit has settled its `inputs`.
  pub(crate) fn prepare(&mut self, options: &RenderViewOptions, output: RenderRect, inputs: PresentInputs) {
    let water = &options.features.water;

    // `combine_2` blurs towards the engine's bloom, which is built under the enhanced one where a writer may blur.
    self.distortion = (!inputs.distortion.is_empty() && water.distortion > 0.0).then(|| {
      (
        DistortUniform {
          strength: water.distortion,
          is_blurred: u32::from(inputs.is_engine_built),
        },
        inputs.distortion,
      )
    });

    let present: PresentUniform = PresentUniform::new(
      PresentFrame {
        view: options.mode.debug_view,
        output,
        is_upscaled: inputs.is_upscaled,
        is_occluded: inputs.is_occluded,
        is_indirect: inputs.is_indirect,
        reflection_ratio: inputs.reflection_ratio,
        selection: inputs.selection_color,
        is_bloomed: inputs.is_engine_bloomed,
      },
      &options.features.corrections,
    );
    self.uniform = if inputs.is_enhanced_bloomed {
      present.with_enhanced_bloom()
    } else {
      present
    }
    .with_display(inputs.display)
    .with_marks(inputs.marks)
    .with_grass_reach(inputs.grass_reach)
    .with_reflection_light(inputs.light)
    .with_grain(&options.features.camera, inputs.grain_frame);
  }

  /// What the window pass reads this frame.
  pub(crate) fn get_uniform(&self) -> &PresentUniform {
    &self.uniform
  }

  /// Moves the scene where the distortion target was written, blurring towards the engine's bloom by its `z`, within
  /// the written tiles alone; nothing where nothing distorts.
  ///
  /// Runs before the resolve, at the drawn size, so the resolve smooths the moved frame as it smooths the rest.
  pub(crate) fn add_distortion_pass<'a>(
    &self,
    GraphDeclaration { graph, runtime, .. }: GraphDeclaration<'_, 'a>,
    passes: LevelPasses<'a>,
    view: &'a ViewBinding,
    handles: ViewTargetHandles,
  ) {
    let Some((distortion, tiles)) = self.distortion else {
      return;
    };

    let (width, height) = handles.size;
    // A texel reads at most this share of the screen away, plus a pixel for truncation, so the copy grows by as many
    // tiles.
    let farthest: f32 = (1.0 - ViewTargets::NEUTRAL_DISTORTION as f32) * distortion.strength;
    let reach = |size: u32, tiles: u32| ((farthest * size as f32 + 1.0) * tiles as f32 / size as f32).ceil() as u32;
    let copied: Vec<RenderRect> = tiles
      .grown(reach(width, ScreenTiles::COLUMNS), reach(height, ScreenTiles::ROWS))
      .get_rects(handles.size);
    let moved: Vec<RenderRect> = tiles.get_rects(handles.size);
    let before: GraphTexture = graph.create_texture(GraphTextureDescriptor::new_2d(
      "scene before distortion",
      width,
      height,
      ViewTargets::SCENE,
    ));

    add_texture_rects_copy(&mut *graph, "scene before distortion", handles.scene, before, copied);

    let parameters: DistortParameters<'a> = DistortParameters {
      frame: before,
      distortion: handles.distortion,
      depth_target: handles.depth,
      bloom_target: handles.bloom[0],
      bloom_sampler: passes.image.bloom.get_sampler(),
      distort: runtime.push_uniform(&distortion),
    };

    graph
      .add_raster_pass("distortion")
      .parameters(&parameters)
      .color(GraphColorAttachment::new(handles.scene, wgpu::LoadOp::Load))
      .record(move |context| passes.effects.distort.record(context, view, &parameters, &moved));
  }
}

impl ViewFeature for PresentFeature {
  fn release(&mut self) {}
}
