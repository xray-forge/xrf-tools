use crate::contract::render_debanding_quality::RenderDebandingQuality;
use crate::contract::render_rect::RenderRect;
use crate::contract::render_view_options::RenderViewOptions;
use crate::pass::present_uniform::PresentUniform;
use crate::scene::level::feature::present_inputs::PresentInputs;
use crate::scene::level::feature::view_feature::ViewFeature;

/// What the window pass puts the view's finished frame into its rectangle by: the debug view shown, the distortion,
/// the corrections, the bloom laid over it, the selection's outline, the sky debanding and the tonemap where the scene
/// holds linear light.
#[derive(Default)]
pub struct PresentFeature {
  uniform: PresentUniform,
  /// How hard the sky is debanded this frame, none where it is drawn as it is.
  debanding: Option<RenderDebandingQuality>,
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

    let present: PresentUniform = PresentUniform::new(
      options.mode.debug_view,
      (inputs.is_occluded, inputs.is_indirect, inputs.reflection_ratio),
      is_upscaled,
      if inputs.is_distorting { water.distortion } else { 0.0 },
      output,
      &options.features.corrections,
      (inputs.selection_color, inputs.is_engine_bloomed),
    );
    let present: PresentUniform = if inputs.is_enhanced_bloomed {
      present.with_enhanced_bloom()
    } else {
      present
    }
    .with_linear_scene(options.is_scene_linear())
    .with_grass_reach(inputs.grass_reach);

    self.uniform = match self.debanding {
      Some(quality) => present.with_debanding(quality.get_passes(), options.features.debanding.radius, time),
      None => present,
    };
  }

  /// What the window pass reads this frame.
  pub fn get_uniform(&self) -> &PresentUniform {
    &self.uniform
  }

  /// How hard the sky is debanded this frame.
  pub fn get_debanding(&self) -> Option<RenderDebandingQuality> {
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
