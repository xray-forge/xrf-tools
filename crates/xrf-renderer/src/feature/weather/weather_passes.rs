use xrf_error::XrfResult;

use crate::feature::weather::fog::fog_scattering_pass::FogScatteringPass;
use crate::feature::weather::sky::deband::sky_deband_pass::SkyDebandPass;
use crate::feature::weather::sky::dome::sky_behind_pass::SkyBehindPass;
use crate::feature::weather::sky::dome::sky_haze_pass::SkyHazePass;
use crate::feature::weather::sky::dome::sky_motion_pass::SkyMotionPass;
use crate::feature::weather::sky::irradiance::sky_irradiance_pass::SkyIrradiancePass;
use crate::feature::weather::sky::prefilter::sky_prefilter_pass::SkyPrefilterPass;
use crate::feature::weather::sun_shafts::sun_shafts_pass::SunShaftsPass;
use crate::feature::weather::volumetric::froxels::froxel_pass::FroxelPass;
use crate::feature::weather::volumetric::light_volumes::light_volume_pass::LightVolumePass;
use crate::feature::weather::water::draw::water_pass::WaterPass;
use crate::feature::weather::wet::draw::wet_pass::WetPass;
use crate::feature::weather::wet::puddles::puddle_sites_pass::PuddleSitesPass;
use crate::pass::build::pass_build::PassBuild;
use crate::pass::build::pipeline_source::PipelineSource;
#[cfg(test)]
use crate::shader::generated_modules::GeneratedModules;

/// The passes drawing the weather: the sky, its haze, motion, light and debanding, the sun's shafts, the fog and the
/// volumetric light, the water, and the wet surfaces with their puddles.
pub struct WeatherPasses {
  pub sky_haze: SkyHazePass,
  pub sky_motion: SkyMotionPass,
  pub sky_irradiance: SkyIrradiancePass,
  pub sky_prefilter: SkyPrefilterPass,
  pub sky_behind: SkyBehindPass,
  pub sky_deband: SkyDebandPass,
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
      let sky_deband = threads.spawn(move || SkyDebandPass::new(source));
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
        sky_deband: PassBuild::join(sky_deband)?,
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
    self.sky_deband.refresh(source);
    self.sun_shafts.refresh(source);
    self.fog_scattering.refresh(source);
    self.light_volumes.refresh(source);
    self.froxels.refresh(source);
    self.water.refresh(source);
    self.wet.refresh(source);
    self.puddle_sites.refresh(source);
  }

  /// Declares the generated WGSL its passes draw with: the structs their uniforms share and each module's bindings, the
  /// sky's among them, which every pass reading the sky binds.
  ///
  /// # Errors
  ///
  /// Returns an error when passes sharing a module declare one of its bindings otherwise.
  #[cfg(test)]
  pub fn declare_shaders(modules: &mut GeneratedModules) -> XrfResult<()> {
    use crate::feature::weather::fog::fog_scattering_parameters::FogScatteringParameters;
    use crate::feature::weather::fog::fog_scattering_uniform::FogScatteringUniform;
    use crate::feature::weather::sky::deband::sky_deband_parameters::{
      SkyDebandCubeParameters, SkyDebandFlatParameters,
    };
    use crate::feature::weather::sky::dome::sky_behind_parameters::SkyBehindParameters;
    use crate::feature::weather::sky::dome::sky_haze_parameters::SkyHazeParameters;
    use crate::feature::weather::sky::dome::sky_motion_parameters::SkyMotionParameters;
    use crate::feature::weather::sky::irradiance::sky_harmonics::SkyHarmonics;
    use crate::feature::weather::sky::irradiance::sky_irradiance_parameters::SkyIrradianceParameters;
    use crate::feature::weather::sky::prefilter::sky_prefilter_parameters::SkyPrefilterParameters;
    use crate::feature::weather::sky::prefilter::sky_prefilter_uniform::SkyPrefilterUniform;
    use crate::feature::weather::sky::sky_parameters::SkyParameters;
    use crate::feature::weather::sun_shafts::sun_shafts_parameters::SunShaftsParameters;
    use crate::feature::weather::volumetric::froxels::froxel_inject_parameters::FroxelInjectParameters;
    use crate::feature::weather::volumetric::froxels::froxel_integrate_parameters::FroxelIntegrateParameters;
    use crate::feature::weather::volumetric::froxels::froxel_uniform::FroxelUniform;
    use crate::feature::weather::volumetric::light_volumes::light_volume_parameters::LightVolumeParameters;
    use crate::feature::weather::water::draw::water_depth_parameters::WaterDepthParameters;
    use crate::feature::weather::water::draw::water_surface_parameters::WaterSurfaceParameters;
    use crate::feature::weather::water::enhanced::enhanced_water_parameters::EnhancedWaterParameters;
    use crate::feature::weather::water::enhanced::enhanced_water_uniform::EnhancedWaterUniform;
    use crate::feature::weather::water::reflection::water_blur_parameters::WaterBlurParameters;
    use crate::feature::weather::water::reflection::water_blur_uniform::WaterBlurUniform;
    use crate::feature::weather::water::reflection::water_reflection_parameters::WaterReflectionParameters;
    use crate::feature::weather::water::water_uniform::WaterUniform;
    use crate::feature::weather::wet::draw::wet_apply_parameters::WetApplyParameters;
    use crate::feature::weather::wet::draw::wet_patch_parameters::WetPatchParameters;
    use crate::feature::weather::wet::puddles::grass_puddles_uniform::GrassPuddlesUniform;
    use crate::feature::weather::wet::puddles::lowest_heights_parameters::LowestHeightsParameters;
    use crate::feature::weather::wet::puddles::puddle_keep_parameters::PuddleKeepParameters;
    use crate::feature::weather::wet::puddles::puddle_sites_parameters::PuddleSitesParameters;
    use crate::feature::weather::wet::puddles::puddle_sites_uniform::PuddleSitesUniform;
    use crate::feature::weather::wet::wet_uniform::WetUniform;

    modules
      .declare::<FroxelUniform>()
      .declare::<GrassPuddlesUniform>()
      .declare::<WaterUniform>()
      .declare::<EnhancedWaterUniform>()
      .declare::<WaterBlurUniform>()
      .declare::<FogScatteringUniform>()
      .declare::<PuddleSitesUniform>()
      .declare::<SkyPrefilterUniform>()
      .declare::<SkyHarmonics>()
      .declare::<WetUniform>();
    modules
      .add::<WaterDepthParameters>("generated/static/water")?
      .add::<WaterSurfaceParameters<'_>>("generated/static/water")?
      .add::<EnhancedWaterParameters<'_>>("generated/static/water_enhanced")?
      .add::<WaterReflectionParameters<'_>>("generated/static/water_reflection")?
      .add::<WaterBlurParameters<'_>>("generated/frame/water_blur")?
      .add::<FogScatteringParameters<'_>>("generated/frame/fog_scattering")?
      .add::<SkyParameters<'_>>("generated/common/sky")?
      .add::<SkyMotionParameters>("generated/frame/sky_motion")?
      .add::<SkyBehindParameters>("generated/frame/sky_behind")?
      .add::<SkyHazeParameters>("generated/frame/sky_haze")?
      .add::<SunShaftsParameters<'_>>("generated/frame/sun_shafts")?
      .add::<LightVolumeParameters<'_>>("generated/frame/light_volumes")?
      .add::<FroxelInjectParameters<'_>>("generated/frame/froxel_inject")?
      .add::<FroxelIntegrateParameters>("generated/frame/froxel_integrate")?
      .add::<WetPatchParameters<'_>>("generated/frame/wet_patch")?
      .add::<WetApplyParameters>("generated/frame/wet_apply")?
      .add::<LowestHeightsParameters>("generated/frame/puddle_sites")?
      .add::<PuddleSitesParameters>("generated/frame/puddle_sites")?
      .add::<PuddleKeepParameters<'_>>("generated/frame/puddle_sites")?
      .add::<SkyIrradianceParameters<'_>>("generated/frame/sky_irradiance")?
      .add::<SkyPrefilterParameters<'_>>("generated/frame/sky_prefilter")?
      .add::<SkyDebandCubeParameters<'_>>("generated/frame/sky_deband_cube")?
      .add::<SkyDebandFlatParameters<'_>>("generated/frame/sky_deband_flat")?;

    Ok(())
  }
}
