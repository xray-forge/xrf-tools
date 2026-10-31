use xrf_renderer_core::{GraphColorAttachment, GraphDeclaration, GraphTexture, GraphTextureDescriptor};

use crate::contract::render_rect::RenderRect;
use crate::contract::render_view_options::RenderViewOptions;
use crate::frame::texture_copy::add_texture_copy;
use crate::frame::view_target_handles::ViewTargetHandles;
use crate::frame::view_targets::ViewTargets;
use crate::pass::display_mode::DisplayMode;
use crate::pass::distort_parameters::DistortParameters;
use crate::pass::distort_uniform::DistortUniform;
use crate::pass::level_passes::LevelPasses;
use crate::pass::present_frame::PresentFrame;
use crate::pass::present_uniform::PresentUniform;
use crate::pass::view_binding::ViewBinding;
use crate::scene::level::feature::present_inputs::PresentInputs;
use crate::scene::level::feature::view_feature::ViewFeature;

/// What the window pass puts the view's finished frame into its rectangle by: the debug view shown, the distortion,
/// the corrections, the bloom laid over it, the selection's outline, the sky debanding and the tonemap where the scene
/// holds linear light.
#[derive(Default)]
pub struct PresentFeature {
  uniform: PresentUniform,
  /// How many times the sky is averaged this frame, none where it is drawn as it is.
  debanding: Option<u32>,
  /// How far the distortion target moves the frame before the resolve and whether it blurs it; none where nothing
  /// wrote the target this frame.
  distortion: Option<DistortUniform>,
}

impl PresentFeature {
  /// Takes this frame's present into `output`, of a scene upscaled into it where `is_upscaled`, once every other unit
  /// has decided what it reads (`inputs`), the debanding's noise turning with `time`.
  pub fn prepare(
    &mut self,
    options: &RenderViewOptions,
    (is_upscaled, output): (bool, RenderRect),
    inputs: PresentInputs,
    time: f32,
  ) {
    let water = &options.features.water;

    self.debanding = PresentUniform::get_debanding(options);
    // `combine_2` blurs towards the engine's bloom, which is built under the enhanced one for it.
    self.distortion = (inputs.is_distorting && water.distortion > 0.0).then(|| DistortUniform {
      strength: water.distortion,
      is_blurred: u32::from(inputs.is_engine_bloomed || inputs.is_enhanced_bloomed),
    });

    let present: PresentUniform = PresentUniform::new(
      PresentFrame {
        view: options.mode.debug_view,
        output,
        is_upscaled,
        is_occluded: inputs.is_occluded,
        is_indirect: inputs.is_indirect,
        reflection_ratio: inputs.reflection_ratio,
        selection: inputs.selection_color,
        is_bloomed: inputs.is_engine_bloomed,
      },
      &options.features.corrections,
    );
    let present: PresentUniform = if inputs.is_enhanced_bloomed {
      present.with_enhanced_bloom()
    } else {
      present
    }
    .with_display(DisplayMode::of(options), &options.features.grading, &inputs.metering)
    .with_marks(inputs.marks)
    .with_grass_reach(inputs.grass_reach)
    .with_reflection_light(inputs.light)
    .with_grain(&options.features.camera, inputs.grain_frame);

    self.uniform = match self.debanding {
      Some(passes) => present.with_debanding(passes, options.features.debanding.radius, time),
      None => present,
    };
  }

  /// What the window pass reads this frame.
  pub fn get_uniform(&self) -> &PresentUniform {
    &self.uniform
  }

  /// Moves the frame (`handles.scene`, at the drawn size) where the water, the particles, the rain and the distorting
  /// surfaces wrote the distortion target, and blurs it towards the engine's bloom by its `z` where a bloom draws,
  /// before the resolve, so the resolve smooths it as it smooths the rest: a copy of the frame read where the target
  /// says, drawn back into it. Nothing where nothing distorts.
  pub fn add_distortion_pass<'a>(
    &self,
    GraphDeclaration { graph, runtime, .. }: GraphDeclaration<'_, 'a>,
    passes: LevelPasses<'a>,
    view: &'a ViewBinding,
    handles: ViewTargetHandles,
  ) {
    let Some(distortion) = self.distortion else {
      return;
    };

    let (width, height) = handles.size;
    let before: GraphTexture = graph.create_texture(GraphTextureDescriptor::new_2d(
      "scene before distortion",
      width,
      height,
      ViewTargets::SCENE,
    ));

    add_texture_copy(&mut *graph, "scene before distortion", handles.scene, before);

    let parameters: DistortParameters<'a> = DistortParameters {
      frame: before,
      distortion: handles.distortion,
      depth_target: handles.depth,
      bloom_target: handles.bloom[0],
      bloom_sampler: passes.bloom.get_sampler(),
      distort: runtime.push_uniform(&distortion),
    };

    graph
      .add_raster_pass("distortion")
      .parameters(&parameters)
      .color(GraphColorAttachment::new(
        handles.scene,
        wgpu::LoadOp::Clear(wgpu::Color::TRANSPARENT),
      ))
      .record(move |context| passes.distort.record(context, view, &parameters));
  }

  /// How many times the sky is averaged this frame.
  pub fn get_debanding(&self) -> Option<u32> {
    self.debanding
  }
}

impl ViewFeature for PresentFeature {
  /// Its targets are the window's own.
  fn get_memory(&self) -> u64 {
    0
  }

  fn release(&mut self) {}
}
