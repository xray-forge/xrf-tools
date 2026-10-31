use xrf_error::XrfResult;

use crate::pass::bloom_pass::BloomPass;
use crate::pass::camera_effects_pass::CameraEffectsPass;
use crate::pass::enhanced_bloom_pass::EnhancedBloomPass;
use crate::pass::exposure_pass::ExposurePass;
use crate::pass::fsr_pass::FsrPass;
use crate::pass::fxaa_pass::FxaaPass;
use crate::pass::meter_pass::MeterPass;
use crate::pass::pipeline_source::PipelineSource;
use crate::pass::smaa_pass::SmaaPass;
use crate::pass::stage::pass_build::PassBuild;
use crate::pass::temporal_pass::TemporalPass;
use crate::pass::upscale_pass::UpscalePass;

/// The passes turning the lit frame into the image shown: the bloom, the smoothing, the temporal resolves and the
/// upscale, the camera's effects, the exposure and the spot meter.
pub struct ImagePasses {
  pub bloom: BloomPass,
  pub enhanced_bloom: EnhancedBloomPass,
  pub fxaa: FxaaPass,
  /// None where its lookup textures could not be read, and the frame smooths as FXAA does.
  pub smaa: Option<SmaaPass>,
  pub temporal: TemporalPass,
  pub fsr: FsrPass,
  pub upscale: UpscalePass,
  pub camera_effects: CameraEffectsPass,
  pub exposure: ExposurePass,
  pub meter: MeterPass,
}

impl ImagePasses {
  /// Builds every pass at once, each on a thread of its own; the enhanced bloom and the camera's effects make their
  /// pipelines on demand.
  ///
  /// # Errors
  ///
  /// Returns an error when a pass's shaders do not compose or compile.
  pub fn new(build: PassBuild<'_>) -> XrfResult<Self> {
    let PassBuild {
      source,
      queue,
      bundle,
      view,
      ..
    } = build;

    std::thread::scope(|threads| {
      let bloom = threads.spawn(move || BloomPass::new(source));
      let fxaa = threads.spawn(move || FxaaPass::new(source));
      let smaa = threads.spawn(move || SmaaPass::new(queue, source, bundle));
      let temporal = threads.spawn(move || TemporalPass::new(source, view));
      let fsr = threads.spawn(move || FsrPass::new(source));
      let upscale = threads.spawn(move || UpscalePass::new(source));
      let exposure = threads.spawn(move || ExposurePass::new(source));
      let meter = threads.spawn(move || MeterPass::new(source, view));

      Ok(Self {
        bloom: PassBuild::join(bloom)?,
        enhanced_bloom: EnhancedBloomPass::new(source.device, view),
        fxaa: PassBuild::join(fxaa)?,
        smaa: PassBuild::join(smaa)
          .inspect_err(|error| log::error!("SMAA cannot be built, smoothing as FXAA does instead: {error}"))
          .ok(),
        temporal: PassBuild::join(temporal)?,
        fsr: PassBuild::join(fsr)?,
        upscale: PassBuild::join(upscale)?,
        camera_effects: CameraEffectsPass::new(source.device),
        exposure: PassBuild::join(exposure)?,
        meter: PassBuild::join(meter)?,
      })
    })
  }

  /// Rebuilds the pipelines a shader reload made stale, each pass keeping its last good ones where a rebuild fails.
  pub fn refresh(&mut self, source: PipelineSource<'_>) {
    self.bloom.refresh(source);
    self.fxaa.refresh(source);

    if let Some(smaa) = &mut self.smaa {
      smaa.refresh(source);
    }

    self.temporal.refresh(source);
    self.fsr.refresh(source);
    self.upscale.refresh(source);
    self.exposure.refresh(source);
    self.meter.refresh(source);
  }
}
