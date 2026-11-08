use xrf_error::XrfResult;

use crate::feature::image::bloom::bloom_pass::BloomPass;
use crate::feature::image::bloom::enhanced::enhanced_bloom_pass::EnhancedBloomPass;
use crate::feature::image::camera::camera_effects_pass::CameraEffectsPass;
use crate::feature::image::exposure::exposure_pass::ExposurePass;
use crate::feature::image::meter::meter_pass::MeterPass;
use crate::feature::image::resolve::fsr::fsr_pass::FsrPass;
use crate::feature::image::resolve::temporal::temporal_pass::TemporalPass;
use crate::feature::image::resolve::upscale::upscale_pass::UpscalePass;
use crate::feature::image::smoothing::fxaa_pass::FxaaPass;
use crate::feature::image::smoothing::smaa_pass::SmaaPass;
use crate::pass::build::pass_build::PassBuild;
use crate::pass::build::pipeline_source::PipelineSource;
#[cfg(test)]
use crate::shader::generated_modules::GeneratedModules;

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

  /// Declares the generated WGSL its passes draw with: the structs their uniforms share and each module's bindings.
  ///
  /// # Errors
  ///
  /// Returns an error when passes sharing a module declare one of its bindings otherwise.
  #[cfg(test)]
  pub fn declare_shaders(modules: &mut GeneratedModules) -> XrfResult<()> {
    use crate::feature::image::bloom::bloom_parameters::BloomParameters;
    use crate::feature::image::bloom::bloom_uniform::BloomUniform;
    use crate::feature::image::bloom::enhanced::enhanced_bloom_parameters::EnhancedBloomParameters;
    use crate::feature::image::bloom::enhanced::enhanced_bloom_uniform::EnhancedBloomUniform;
    use crate::feature::image::camera::camera_effects_parameters::CameraEffectsParameters;
    use crate::feature::image::camera::camera_effects_uniform::CameraEffectsUniform;
    use crate::feature::image::exposure::exposure_parameters::ExposureParameters;
    use crate::feature::image::exposure::exposure_state::ExposureState;
    use crate::feature::image::exposure::exposure_uniform::ExposureUniform;
    use crate::feature::image::exposure::histogram::exposure_histogram_parameters::ExposureHistogramParameters;
    use crate::feature::image::exposure::histogram::exposure_histogram_uniform::ExposureHistogramUniform;
    use crate::feature::image::exposure::reading::exposure_head::ExposureHead;
    use crate::feature::image::meter::meter_parameters::MeterParameters;
    use crate::feature::image::meter::meter_sample::MeterSample;
    use crate::feature::image::resolve::fsr::fsr_uniform::FsrUniform;
    use crate::feature::image::resolve::temporal::temporal_parameters::TemporalParameters;
    use crate::feature::image::resolve::temporal::temporal_uniform::TemporalUniform;
    use crate::feature::image::resolve::upscale::upscale_parameters::UpscaleParameters;
    use crate::feature::image::resolve::upscale::upscale_uniform::UpscaleUniform;
    use crate::feature::image::smoothing::fxaa_parameters::FxaaParameters;
    use crate::feature::image::smoothing::smaa_parameters::SmaaParameters;
    use crate::feature::image::smoothing::smoothing_uniform::SmoothingUniform;

    modules
      .declare::<SmoothingUniform>()
      .declare::<UpscaleUniform>()
      .declare::<BloomUniform>()
      .declare::<EnhancedBloomUniform>()
      .declare::<ExposureUniform>()
      .declare::<ExposureState>()
      .declare::<ExposureHead>()
      .declare::<ExposureHistogramUniform>()
      .declare::<MeterSample>()
      .declare::<TemporalUniform>()
      .declare::<FsrUniform>()
      .declare::<CameraEffectsUniform>();
    modules
      .add::<UpscaleParameters>("generated/frame/upscale")?
      .add::<BloomParameters<'_>>("generated/frame/bloom")?
      .add::<EnhancedBloomParameters<'_>>("generated/frame/enhanced_bloom")?
      .add::<FxaaParameters<'_>>("generated/frame/fxaa")?
      .add::<SmaaParameters<'_>>("generated/frame/smaa")?
      .add::<ExposureParameters>("generated/frame/exposure")?
      .add::<ExposureHistogramParameters>("generated/frame/exposure_histogram")?
      .add::<MeterParameters>("generated/frame/meter")?
      .add::<CameraEffectsParameters<'_>>("generated/frame/camera_effects")?
      .add::<TemporalParameters<'_>>("generated/frame/temporal")?;

    Ok(())
  }
}
