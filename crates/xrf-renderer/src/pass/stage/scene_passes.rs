use xrf_error::XrfResult;

use crate::pass::calibration_pass::CalibrationPass;
use crate::pass::depth_pyramid_pass::DepthPyramidPass;
use crate::pass::grass_colour_pass::GrassColourPass;
use crate::pass::grass_pass::GrassPass;
use crate::pass::pipeline_source::PipelineSource;
use crate::pass::stage::pass_build::PassBuild;
use crate::pass::static_cull_pass::StaticCullPass;
use crate::pass::static_gbuffer_pass::StaticGBufferPass;

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
}
