use xrf_error::XrfResult;

use crate::feature::scene::calibration::calibration_parameters::CalibrationParameters;
use crate::feature::scene::calibration::calibration_pass::CalibrationPass;
use crate::feature::scene::calibration::calibration_uniform::CalibrationUniform;
use crate::feature::scene::grass::build::grass_build_parameters::GrassBuildParameters;
use crate::feature::scene::grass::build::grass_sort_parameters::GrassSortParameters;
use crate::feature::scene::grass::cards::grass_card_shadow_parameters::GrassCardShadowParameters;
use crate::feature::scene::grass::cards::grass_cards_parameters::GrassCardsParameters;
use crate::feature::scene::grass::colour::grass_colour_parameters::GrassColourParameters;
use crate::feature::scene::grass::colour::grass_colour_pass::GrassColourPass;
use crate::feature::scene::grass::draw::grass_draw_parameters::GrassDrawParameters;
use crate::feature::scene::grass::draw::grass_occlusion_parameters::GrassOcclusionParameters;
use crate::feature::scene::grass::draw::grass_wind_uniform::GrassWindUniform;
use crate::feature::scene::grass::grass_pass::GrassPass;
use crate::feature::scene::grass::grass_uniform::GrassUniform;
use crate::feature::scene::grass::level::grass_level_parameters::GrassLevelParameters;
use crate::feature::scene::grass::level::grass_model_record::GrassModelRecord;
use crate::feature::scene::grass::trace::trace_grass_uniform::TraceGrassUniform;
use crate::feature::scene::pyramid::cover_pyramid_depth_parameters::CoverPyramidDepthParameters;
use crate::feature::scene::pyramid::cover_pyramid_level_parameters::CoverPyramidLevelParameters;
use crate::feature::scene::pyramid::depth_pyramid_pass::DepthPyramidPass;
use crate::feature::scene::pyramid::pyramid_depth_parameters::PyramidDepthParameters;
use crate::feature::scene::pyramid::pyramid_level_parameters::PyramidLevelParameters;
use crate::feature::scene::statics::cull::static_cull_parameters::StaticCullParameters;
use crate::feature::scene::statics::cull::static_cull_pass::StaticCullPass;
use crate::feature::scene::statics::draw::static_draw_parameters::StaticDrawParameters;
use crate::feature::scene::statics::draw::static_impostor_parameters::StaticImpostorParameters;
use crate::feature::scene::statics::gbuffer::static_gbuffer_pass::StaticGBufferPass;
use crate::pass::build::pass_build::PassBuild;
use crate::pass::build::pipeline_source::PipelineSource;
use crate::shader::generated_modules::GeneratedModules;

/// The passes drawing the scene into the G-buffer: the static scene's cull, its depth pyramid and its draws, the grass
/// and its colours, and the calibration props.
pub struct ScenePasses {
  pub cull: StaticCullPass,
  pub gbuffer: StaticGBufferPass,
  pub pyramid: DepthPyramidPass,
  pub grass: GrassPass,
  pub grass_colours: GrassColourPass,
  pub calibration: CalibrationPass,
}

impl ScenePasses {
  /// Builds every pass at once, each on a thread of its own.
  ///
  /// # Errors
  ///
  /// Returns an error when a pass's shaders do not compose or compile.
  pub fn new(build: PassBuild<'_>) -> XrfResult<Self> {
    let PassBuild {
      source, view, texture, ..
    } = build;

    std::thread::scope(|threads| {
      let cull = threads.spawn(move || StaticCullPass::new(source, view));
      let gbuffer = threads.spawn(move || StaticGBufferPass::new(source, view, texture));
      let pyramid = threads.spawn(move || DepthPyramidPass::new(source));
      let grass = threads.spawn(move || GrassPass::new(source, view, texture));
      let grass_colours = threads.spawn(move || GrassColourPass::new(source, texture));
      let calibration = threads.spawn(move || CalibrationPass::new(source, view));

      Ok(Self {
        cull: PassBuild::join(cull)?,
        gbuffer: PassBuild::join(gbuffer)?,
        pyramid: PassBuild::join(pyramid)?,
        grass: PassBuild::join(grass)?,
        grass_colours: PassBuild::join(grass_colours)?,
        calibration: PassBuild::join(calibration)?,
      })
    })
  }

  /// Rebuilds the pipelines a shader reload made stale, each pass keeping its last good ones where a rebuild fails.
  pub fn refresh(&mut self, source: PipelineSource<'_>) {
    self.cull.refresh(source);
    self.gbuffer.refresh(source);
    self.pyramid.refresh(source);
    self.grass.refresh(source);
    self.grass_colours.refresh(source);
    self.calibration.refresh(source);
  }

  /// Declares the generated WGSL its passes draw with: the structs their uniforms and storage share and each module's
  /// bindings.
  ///
  /// # Errors
  ///
  /// Returns an error when passes sharing a module declare one of its bindings otherwise.
  pub fn declare_shaders(modules: &mut GeneratedModules) -> XrfResult<()> {
    modules
      .declare::<GrassModelRecord>()
      .declare::<GrassUniform>()
      .declare::<GrassWindUniform>()
      .declare::<TraceGrassUniform>()
      .declare::<CalibrationUniform>()
      .declare_held::<StaticCullParameters>()
      .declare_held::<StaticDrawParameters>()
      .declare_held::<StaticImpostorParameters>();
    modules
      .add::<GrassOcclusionParameters>("generated/grass/occlusion")?
      .add::<GrassLevelParameters>("generated/grass/planting")?
      .add::<GrassBuildParameters>("generated/grass/planting")?
      .add::<GrassSortParameters>("generated/grass/sort")?
      .add::<GrassCardsParameters>("generated/grass/cards")?
      .add::<GrassDrawParameters>("generated/grass/draw")?
      .add::<GrassCardShadowParameters>("generated/grass/card_shadow")?
      .add::<GrassColourParameters>("generated/grass/colours")?
      .add::<CalibrationParameters>("generated/frame/calibration")?
      .add::<PyramidDepthParameters>("generated/frame/pyramid")?
      .add::<PyramidLevelParameters>("generated/frame/pyramid")?
      .add::<CoverPyramidDepthParameters>("generated/frame/pyramid")?
      .add::<CoverPyramidLevelParameters>("generated/frame/pyramid")?
      .add::<StaticCullParameters>("generated/static/cull")?
      .add::<StaticDrawParameters>("generated/static/draw")?
      .add::<StaticImpostorParameters>("generated/static/impostor")?;

    Ok(())
  }
}
