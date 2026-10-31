use xrf_error::XrfResult;

use crate::pass::composited_pass::CompositedPass;
use crate::pass::distort_pass::DistortPass;
use crate::pass::flare_pass::FlarePass;
use crate::pass::glass_pass::GlassPass;
use crate::pass::particle_pass::ParticlePass;
use crate::pass::pipeline_source::PipelineSource;
use crate::pass::rain_pass::RainPass;
use crate::pass::stage::pass_build::PassBuild;
use crate::pass::thunder_pass::ThunderPass;

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
}
