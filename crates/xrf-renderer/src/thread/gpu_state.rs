use std::collections::HashMap;

use xrf_error::XrfResult;

use crate::context::gpu_context::GpuContext;
use crate::host::render_bundle::RenderBundle;
use crate::pass::ambient_occlusion_pass::AmbientOcclusionPass;
use crate::pass::backdrop_pass::BackdropPass;
use crate::pass::combine_pass::CombinePass;
use crate::pass::composited_pass::CompositedPass;
use crate::pass::depth_pyramid_pass::DepthPyramidPass;
use crate::pass::exposure_pass::ExposurePass;
use crate::pass::flare_pass::FlarePass;
use crate::pass::fsr_pass::FsrPass;
use crate::pass::fxaa_pass::FxaaPass;
use crate::pass::grass_pass::GrassPass;
use crate::pass::grid_pass::GridPass;
use crate::pass::level_passes::LevelPasses;
use crate::pass::lights_pass::LightsPass;
use crate::pass::material_table::MaterialTable;
use crate::pass::overlay_pass::OverlayPass;
use crate::pass::particle_pass::ParticlePass;
use crate::pass::present_pass::PresentPass;
use crate::pass::rain_pass::RainPass;
use crate::pass::sky_bindings::SkyBindings;
use crate::pass::sky_haze_pass::SkyHazePass;
use crate::pass::smaa_pass::SmaaPass;
use crate::pass::static_cull_pass::StaticCullPass;
use crate::pass::static_gbuffer_pass::StaticGBufferPass;
use crate::pass::static_shadow_pass::StaticShadowPass;
use crate::pass::sun_pass::SunPass;
use crate::pass::sun_shafts_pass::SunShaftsPass;
use crate::pass::temporal_pass::TemporalPass;
use crate::pass::thunder_pass::ThunderPass;
use crate::pass::upscale_pass::UpscalePass;
use crate::pass::view_binding::ViewBinding;
use crate::pass::water_pass::WaterPass;
use crate::pass::wet_pass::WetPass;
use crate::scene::texture::texture_cache::TextureCache;
use crate::scene::texture::weather_texture_cache::WeatherTextureCache;
use crate::shader::shader_library::ShaderLibrary;
use crate::thread::render_workers::RenderWorkers;
use crate::window::render_window::RenderWindow;

/// Everything made on the GPU, which goes as a whole when the GPU stops or is lost.
pub struct GpuState {
  pub context: GpuContext,
  pub view_layout: wgpu::BindGroupLayout,
  /// Each window's swapchain, by its host's key.
  pub windows: HashMap<u64, RenderWindow>,
  /// Every texture any viewport's scene samples, uploaded once however many draw it.
  pub textures: TextureCache,
  /// The weather's skies and clouds, by reference, kept while any viewport asks for them.
  pub weather_textures: WeatherTextureCache,
  pub sky: SkyBindings,
  pub sky_haze: SkyHazePass,
  pub water: WaterPass,
  pub composited: CompositedPass,
  pub grass: GrassPass,
  pub rain: RainPass,
  pub particles: ParticlePass,
  pub wet: WetPass,
  pub thunder: ThunderPass,
  pub flares: FlarePass,
  pub sun_shafts: SunShaftsPass,
  pub temporal: TemporalPass,
  pub fsr: FsrPass,
  pub fxaa: FxaaPass,
  /// None where its lookup textures could not be read.
  pub smaa: Option<SmaaPass>,
  pub upscale: UpscalePass,
  pub static_cull: StaticCullPass,
  pub static_gbuffer: StaticGBufferPass,
  pub static_shadow: StaticShadowPass,
  pub pyramid: DepthPyramidPass,
  pub sun: SunPass,
  pub ambient_occlusion: AmbientOcclusionPass,
  pub lights: LightsPass,
  pub combine: CombinePass,
  pub exposure: ExposurePass,
  pub present: PresentPass,
  /// The page's backdrop under every window's viewports.
  pub backdrop: BackdropPass,
  pub overlay: OverlayPass,
  pub table: MaterialTable,
  /// The grid pass for each target format drawn into.
  grids: HashMap<wgpu::TextureFormat, GridPass>,
}

impl GpuState {
  /// # Errors
  ///
  /// Returns an error when a pass's shaders do not compose or compile.
  pub fn new(
    context: GpuContext,
    shaders: &ShaderLibrary,
    workers: &RenderWorkers,
    bundle: &dyn RenderBundle,
  ) -> XrfResult<Self> {
    let device: &wgpu::Device = &context.device;
    let view_layout: wgpu::BindGroupLayout = ViewBinding::create_layout(device);
    let textures: TextureCache = TextureCache::new(device, &context.queue, workers);
    let sky: SkyBindings = SkyBindings::new(device);
    let static_gbuffer: StaticGBufferPass =
      StaticGBufferPass::new(device, shaders, &view_layout, textures.get_layout())?;
    let static_shadow: StaticShadowPass = StaticShadowPass::new(
      device,
      shaders,
      &view_layout,
      static_gbuffer.get_layout(),
      textures.get_layout(),
    )?;

    let water: WaterPass = WaterPass::new(
      device,
      shaders,
      &view_layout,
      static_gbuffer.get_layout(),
      textures.get_layout(),
    )?;
    let composited: CompositedPass = CompositedPass::new(
      device,
      shaders,
      &view_layout,
      static_gbuffer.get_layout(),
      textures.get_layout(),
      sky.get_layout(),
    )?;

    Ok(Self {
      water,
      composited,
      grass: GrassPass::new(device, shaders, &view_layout, textures.get_layout())?,
      rain: RainPass::new(device, shaders, &view_layout)?,
      particles: ParticlePass::new(device, shaders, &view_layout, textures.get_layout())?,
      wet: WetPass::new(device, shaders, &view_layout)?,
      thunder: ThunderPass::new(device, shaders, &view_layout)?,
      flares: FlarePass::new(device, shaders, &view_layout)?,
      sun_shafts: SunShaftsPass::new(device, shaders, &view_layout)?,
      temporal: TemporalPass::new(device, shaders, &view_layout)?,
      fsr: FsrPass::new(device, shaders)?,
      fxaa: FxaaPass::new(device, shaders)?,
      smaa: SmaaPass::new(device, &context.queue, shaders, bundle)
        .inspect_err(|error| log::error!("SMAA cannot be built, smoothing as FXAA does instead: {error}"))
        .ok(),
      upscale: UpscalePass::new(device, shaders)?,
      static_cull: StaticCullPass::new(device, shaders, &view_layout)?,
      static_gbuffer,
      static_shadow,
      pyramid: DepthPyramidPass::new(device, shaders)?,
      sun: SunPass::new(device, shaders, &view_layout)?,
      ambient_occlusion: AmbientOcclusionPass::new(device, shaders, &view_layout)?,
      lights: LightsPass::new(device, shaders, &view_layout, textures.get_layout())?,
      combine: CombinePass::new(device, shaders, &view_layout, sky.get_layout())?,
      sky_haze: SkyHazePass::new(device, shaders, sky.get_layout())?,
      weather_textures: WeatherTextureCache::new(device, &context.queue, workers),
      sky,
      exposure: ExposurePass::new(device, shaders)?,
      present: PresentPass::new(device, shaders, &view_layout),
      backdrop: BackdropPass::new(device, shaders),
      overlay: OverlayPass::new(device, shaders, &view_layout),
      table: MaterialTable::new(device, &context.queue),
      windows: HashMap::new(),
      grids: HashMap::new(),
      textures,
      view_layout,
      context,
    })
  }

  /// Builds the grid pass drawing into a format, unless it is built.
  ///
  /// # Errors
  ///
  /// Returns an error when its shader does not compose or compile.
  pub fn ensure_grid(&mut self, shaders: &ShaderLibrary, format: wgpu::TextureFormat) -> XrfResult {
    if !self.grids.contains_key(&format) {
      let grid: GridPass = GridPass::new(&self.context.device, shaders, &self.view_layout, format)?;

      self.grids.insert(format, grid);
    }

    Ok(())
  }

  /// The grid pass drawing into a format, once [`GpuState::ensure_grid`] built it.
  pub fn get_grid(&self, format: wgpu::TextureFormat) -> Option<&GridPass> {
    self.grids.get(&format)
  }

  pub fn get_level_passes(&self) -> LevelPasses<'_> {
    LevelPasses {
      cull: &self.static_cull,
      gbuffer: &self.static_gbuffer,
      shadow: &self.static_shadow,
      pyramid: &self.pyramid,
      sun: &self.sun,
      ambient_occlusion: &self.ambient_occlusion,
      lights: &self.lights,
      combine: &self.combine,
      sky_haze: &self.sky_haze,
      sky: &self.sky,
      water: &self.water,
      composited: &self.composited,
      grass: &self.grass,
      rain: &self.rain,
      particles: &self.particles,
      wet: &self.wet,
      thunder: &self.thunder,
      flares: &self.flares,
      sun_shafts: &self.sun_shafts,
      temporal: &self.temporal,
      fsr: &self.fsr,
      fxaa: &self.fxaa,
      smaa: self.smaa.as_ref(),
      upscale: &self.upscale,
      exposure: &self.exposure,
      present: &self.present,
      overlay: &self.overlay,
      table: &self.table,
    }
  }

  /// Rebuilds pipelines from a shader library reloaded since they were built.
  pub fn refresh(&mut self, shaders: &ShaderLibrary) {
    let device: &wgpu::Device = &self.context.device;

    for grid in self.grids.values_mut() {
      grid.refresh(device, shaders);
    }

    self.static_cull.refresh(device, shaders);
    self.static_gbuffer.refresh(device, shaders);
    self.static_shadow.refresh(device, shaders);
    self.pyramid.refresh(device, shaders);
    self.sun.refresh(device, shaders);
    self.ambient_occlusion.refresh(device, shaders);
    self.lights.refresh(device, shaders);
    self.combine.refresh(device, shaders);
    self.sky_haze.refresh(device, shaders);
    self.water.refresh(device, shaders);
    self.composited.refresh(device, shaders);
    self.grass.refresh(device, shaders);
    self.rain.refresh(device, shaders);
    self.particles.refresh(device, shaders);
    self.wet.refresh(device, shaders);
    self.thunder.refresh(device, shaders);
    self.flares.refresh(device, shaders);
    self.sun_shafts.refresh(device, shaders);
    self.temporal.refresh(device, shaders);
    self.fsr.refresh(device, shaders);
    self.fxaa.refresh(device, shaders);
    if let Some(smaa) = &mut self.smaa {
      smaa.refresh(device, shaders);
    }
    self.upscale.refresh(device, shaders);
    self.exposure.refresh(device, shaders);
    self.present.refresh(shaders);
    self.backdrop.refresh(shaders);
    self.overlay.refresh(shaders);
  }
}
