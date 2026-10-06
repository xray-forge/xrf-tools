//! The enhanced bloom's stages and its place beside the engine's.

use glam::Vec4;

use crate::contract::render_bloom_mode::RenderBloomMode;
use crate::contract::render_debug_view::RenderDebugView;
use crate::contract::render_enhanced_bloom_settings::RenderEnhancedBloomSettings;
use crate::contract::render_image_corrections::RenderImageCorrections;
use crate::contract::render_rect::RenderRect;
use crate::contract::render_view_options::RenderViewOptions;
use crate::pass::enhanced_bloom_pass::EnhancedBloomPass;
use crate::pass::enhanced_bloom_uniform::EnhancedBloomUniform;
use crate::pass::present_uniform::PresentUniform;

fn enhanced() -> RenderEnhancedBloomSettings {
  RenderEnhancedBloomSettings {
    mode: RenderBloomMode::Enhanced,
    ..RenderEnhancedBloomSettings::default()
  }
}

fn options_with(bloom: RenderEnhancedBloomSettings) -> RenderViewOptions {
  let mut options: RenderViewOptions = RenderViewOptions::default();

  options.mode.is_lit = true;
  options.features.enhanced_bloom = bloom;

  options
}

#[test]
fn ships_the_engines_bloom_with_the_enhanced_strengths() {
  let defaults: RenderEnhancedBloomSettings = RenderEnhancedBloomSettings::default();

  assert_eq!(defaults.mode, RenderBloomMode::Engine);
  assert_eq!(
    (
      defaults.threshold,
      defaults.exposure,
      defaults.blur,
      defaults.vibrance,
      defaults.sky
    ),
    (5.0, 3.0, 3.0, 1.5, 0.6)
  );
  assert!(!defaults.is_enhanced());
}

// No pass runs while the engine's bloom is asked for, unlit or in wireframe.
#[test]
fn blooms_only_enhanced_lit_and_solid() {
  assert_eq!(
    EnhancedBloomUniform::for_view(&options_with(enhanced())),
    Some(enhanced())
  );
  assert_eq!(
    EnhancedBloomUniform::for_view(&options_with(RenderEnhancedBloomSettings::default())),
    None
  );

  let mut unlit: RenderViewOptions = options_with(enhanced());

  unlit.mode.is_lit = false;

  assert_eq!(EnhancedBloomUniform::for_view(&unlit), None);

  let mut wireframe: RenderViewOptions = options_with(enhanced());

  wireframe.mode.is_wireframe = true;

  assert_eq!(EnhancedBloomUniform::for_view(&wireframe), None);
}

// Half the frame, then halved six times in all, rounding up and never under a texel; a pass named for each.
#[test]
fn halves_six_times_below_the_frame_and_names_every_stage() {
  let sizes: Vec<(u32, u32)> = (1..=EnhancedBloomUniform::LEVELS as u32)
    .map(|level| EnhancedBloomUniform::get_level_size((1001, 63), level))
    .collect();

  assert_eq!(sizes, [(501, 32), (251, 16), (126, 8), (63, 4), (32, 2), (16, 1)]);
  assert_eq!(EnhancedBloomPass::HALVE_PASSES.len(), EnhancedBloomUniform::LEVELS);
  assert_eq!(EnhancedBloomPass::DOUBLE_PASSES.len(), EnhancedBloomUniform::LEVELS - 1);
}

#[test]
fn writes_each_stages_sizes_way_and_strengths() {
  let stage: EnhancedBloomUniform = EnhancedBloomUniform::new((500, 250), (1000, 500), &enhanced());

  assert_eq!(stage.size, Vec4::new(500.0, 250.0, 1000.0, 500.0));
  assert_eq!(stage.spread, Vec4::new(0.0, 0.0, 3.0, 0.0));
  assert_eq!(stage.strengths, Vec4::new(5.0, 3.0, 0.6, 1.5));
  assert_eq!(stage.along(true).spread, Vec4::new(1.0 / 500.0, 0.0, 3.0, 0.0));
  assert_eq!(stage.along(false).spread, Vec4::new(0.0, 1.0 / 250.0, 3.0, 0.0));
}

// The present screens the enhanced bloom in the engine's place.
#[test]
fn tells_the_present_which_bloom_it_lays_over_the_frame() {
  let present = |is_bloomed: bool| {
    PresentUniform::new(
      RenderDebugView::Final,
      (false, false, None),
      false,
      0.0,
      RenderRect::default(),
      &RenderImageCorrections::default(),
      (None, is_bloomed),
    )
  };

  assert_eq!(present(false).is_bloomed, 0);
  assert_eq!(present(true).is_bloomed, 1);
  assert_eq!(present(false).with_enhanced_bloom().is_bloomed, 2);
}
