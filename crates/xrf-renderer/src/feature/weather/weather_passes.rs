use xrf_error::XrfResult;

use crate::feature::weather::fog::fog_scattering_pass::FogScatteringPass;
use crate::feature::weather::sky::dome::sky_behind_pass::SkyBehindPass;
use crate::feature::weather::sky::dome::sky_haze_pass::SkyHazePass;
use crate::feature::weather::sky::dome::sky_motion_pass::SkyMotionPass;
use crate::feature::weather::sky::irradiance::sky_irradiance_pass::SkyIrradiancePass;
use crate::feature::weather::sky::prefilter::sky_prefilter_pass::SkyPrefilterPass;
use crate::feature::weather::sun_shafts::sun_shafts_pass::SunShaftsPass;
use crate::feature::weather::volumetric::froxels::froxel_pass::FroxelPass;
use crate::feature::weather::volumetric::light_volumes::light_volume_pass::LightVolumePass;
use crate::feature::weather::water::water_pass::WaterPass;
use crate::feature::weather::wet::puddles::puddle_sites_pass::PuddleSitesPass;
use crate::feature::weather::wet::wet_pass::WetPass;
use crate::pass::pipeline_source::PipelineSource;
use crate::pass::stage::pass_build::PassBuild;

/// The passes drawing the weather: the sky, its haze, motion and light, the sun's shafts, the fog and the volumetric
/// light, the water, and the wet surfaces with their puddles.
pub struct WeatherPasses {
  pub sky_haze: SkyHazePass,
  pub sky_motion: SkyMotionPass,
  pub sky_irradiance: SkyIrradiancePass,
  pub sky_prefilter: SkyPrefilterPass,
  pub sky_behind: SkyBehindPass,
  pub sun_shafts: SunShaftsPass,
  pub fog_scattering: FogScatteringPass,
  pub light_volumes: LightVolumePass,
  pub froxels: FroxelPass,
  pub water: WaterPass,
  pub wet: WetPass,
  pub puddle_sites: PuddleSitesPass,
}

impl WeatherPasses {
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
      let sky_haze = threads.spawn(move || SkyHazePass::new(source, sky));
      let sky_motion = threads.spawn(move || SkyMotionPass::new(source, view));
      let sky_irradiance = threads.spawn(move || SkyIrradiancePass::new(source));
      let sky_prefilter = threads.spawn(move || SkyPrefilterPass::new(queue, source));
      let sky_behind = threads.spawn(move || SkyBehindPass::new(source, view, sky));
      let sun_shafts = threads.spawn(move || SunShaftsPass::new(source, view));
      let fog_scattering = threads.spawn(move || FogScatteringPass::new(queue, source, view));
      let light_volumes = threads.spawn(move || LightVolumePass::new(source, view, texture));
      let froxels = threads.spawn(move || FroxelPass::new(source, view));
      let water = threads.spawn(move || WaterPass::new(queue, source, [view, scene, texture], bundle));
      let wet = threads.spawn(move || WetPass::new(source, view));
      let puddle_sites = threads.spawn(move || PuddleSitesPass::new(source));

      Ok(Self {
        sky_haze: PassBuild::join(sky_haze)?,
        sky_motion: PassBuild::join(sky_motion)?,
        sky_irradiance: PassBuild::join(sky_irradiance)?,
        sky_prefilter: PassBuild::join(sky_prefilter)?,
        sky_behind: PassBuild::join(sky_behind)?,
        sun_shafts: PassBuild::join(sun_shafts)?,
        fog_scattering: PassBuild::join(fog_scattering)?,
        light_volumes: PassBuild::join(light_volumes)?,
        froxels: PassBuild::join(froxels)?,
        water: PassBuild::join(water)?,
        wet: PassBuild::join(wet)?,
        puddle_sites: PassBuild::join(puddle_sites)?,
      })
    })
  }

  /// Rebuilds the pipelines a shader reload made stale, each pass keeping its last good ones where a rebuild fails.
  pub fn refresh(&mut self, source: PipelineSource<'_>) {
    self.sky_haze.refresh(source);
    self.sky_motion.refresh(source);
    self.sky_irradiance.refresh(source);
    self.sky_prefilter.refresh(source);
    self.sky_behind.refresh(source);
    self.sun_shafts.refresh(source);
    self.fog_scattering.refresh(source);
    self.light_volumes.refresh(source);
    self.froxels.refresh(source);
    self.water.refresh(source);
    self.wet.refresh(source);
    self.puddle_sites.refresh(source);
  }
}
