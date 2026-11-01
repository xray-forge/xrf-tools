use xrf_error::XrfResult;

use crate::feature::effects::composited::composited_parameters::CompositedParameters;
use crate::feature::effects::composited::composited_pass::CompositedPass;
use crate::feature::effects::composited::glass::glass_copy_dispatch_parameters::GlassCopyDispatchParameters;
use crate::feature::effects::composited::glass::glass_copy_parameters::GlassCopyParameters;
use crate::feature::effects::composited::glass::glass_parameters::GlassParameters;
use crate::feature::effects::composited::glass::glass_pass::GlassPass;
use crate::feature::effects::composited::glass::glass_uniform::GlassUniform;
use crate::feature::effects::distort::distort_parameters::DistortParameters;
use crate::feature::effects::distort::distort_pass::DistortPass;
use crate::feature::effects::distort::distort_uniform::DistortUniform;
use crate::feature::effects::flares::flare_draw_parameters::FlareDrawParameters;
use crate::feature::effects::flares::flare_measure_parameters::FlareMeasureParameters;
use crate::feature::effects::flares::flare_pass::FlarePass;
use crate::feature::effects::flares::flare_texture_parameters::FlareTextureParameters;
use crate::feature::effects::flares::flare_uniform::FlareUniform;
use crate::feature::effects::particles::particle_parameters::ParticleParameters;
use crate::feature::effects::particles::particle_pass::ParticlePass;
use crate::feature::effects::rain::enhanced::rain_blur_parameters::RainBlurParameters;
use crate::feature::effects::rain::enhanced::rain_look_parameters::RainLookParameters;
use crate::feature::effects::rain::rain_parameters::RainParameters;
use crate::feature::effects::rain::rain_pass::RainPass;
use crate::feature::effects::rain::rain_uniform::RainUniform;
use crate::feature::effects::rain::rings::rain_rings_reduce_parameters::RainRingsReduceParameters;
use crate::feature::effects::thunder::thunder_parameters::ThunderParameters;
use crate::feature::effects::thunder::thunder_pass::ThunderPass;
use crate::feature::effects::thunder::thunder_uniform::ThunderUniform;
use crate::pass::build::pass_build::PassBuild;
use crate::pass::build::pipeline_source::PipelineSource;
use crate::shader::generated_modules::GeneratedModules;

/// The passes blending over the lit scene: the composited surfaces and the glass, the particles and the distortion
/// they bend the frame by, the falling rain, the thunder's bolts, and the sun's flares.
pub struct EffectPasses {
  pub composited: CompositedPass,
  pub glass: GlassPass,
  pub particles: ParticlePass,
  pub distort: DistortPass,
  pub rain: RainPass,
  pub thunder: ThunderPass,
  pub flares: FlarePass,
}

impl EffectPasses {
  /// Builds every pass at once, each on a thread of its own.
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
      scene,
      texture,
      sky,
    } = build;

    std::thread::scope(|threads| {
      let composited = threads.spawn(move || CompositedPass::new(source, view, scene, texture, sky));
      let glass = threads.spawn(move || GlassPass::new(source, view, scene, texture, sky));
      let particles = threads.spawn(move || ParticlePass::new(source, view, texture));
      let distort = threads.spawn(move || DistortPass::new(source, view));
      let rain = threads.spawn(move || RainPass::new(queue, source, view, bundle));
      let thunder = threads.spawn(move || ThunderPass::new(source, view));
      let flares = threads.spawn(move || FlarePass::new(source, view));

      Ok(Self {
        composited: PassBuild::join(composited)?,
        glass: PassBuild::join(glass)?,
        particles: PassBuild::join(particles)?,
        distort: PassBuild::join(distort)?,
        rain: PassBuild::join(rain)?,
        thunder: PassBuild::join(thunder)?,
        flares: PassBuild::join(flares)?,
      })
    })
  }

  /// Rebuilds the pipelines a shader reload made stale, each pass keeping its last good ones where a rebuild fails.
  pub fn refresh(&mut self, source: PipelineSource<'_>) {
    self.composited.refresh(source);
    self.glass.refresh(source);
    self.particles.refresh(source);
    self.distort.refresh(source);
    self.rain.refresh(source);
    self.thunder.refresh(source);
    self.flares.refresh(source);
  }

  /// Declares the generated WGSL its passes draw with: the structs their uniforms share and each module's bindings.
  ///
  /// # Errors
  ///
  /// Returns an error when passes sharing a module declare one of its bindings otherwise.
  pub fn declare_shaders(modules: &mut GeneratedModules) -> XrfResult<()> {
    modules
      .declare::<GlassUniform>()
      .declare::<DistortUniform>()
      .declare::<FlareUniform>()
      .declare::<RainUniform>()
      .declare::<ThunderUniform>();
    modules
      .add::<ParticleParameters<'_>>("generated/frame/particles")?
      .add::<DistortParameters<'_>>("generated/frame/distort")?
      .add::<CompositedParameters<'_>>("generated/static/composited")?
      .add::<GlassParameters<'_>>("generated/static/glass")?
      .add::<GlassCopyDispatchParameters>("generated/frame/glass_copy_dispatch")?
      .add::<GlassCopyParameters>("generated/frame/glass_copy")?
      .add::<FlareDrawParameters<'_>>("generated/frame/flare")?
      .add::<FlareTextureParameters>("generated/frame/flare")?
      .add::<FlareMeasureParameters>("generated/frame/flare_visibility")?
      .add::<RainParameters<'_>>("generated/frame/rain")?
      .add::<RainLookParameters<'_>>("generated/frame/rain")?
      .add::<RainBlurParameters<'_>>("generated/frame/rain_blur")?
      .add::<RainRingsReduceParameters>("generated/frame/rain_rings")?
      .add::<ThunderParameters<'_>>("generated/frame/thunder")?;

    Ok(())
  }
}
