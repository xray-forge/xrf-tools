use std::collections::HashMap;
use std::sync::Arc;

use xrf_error::XrfResult;
use xrf_renderer_core::{PassParameters, PipelineCache};

use crate::context::gpu_context::GpuContext;
use crate::host::render_bundle::RenderBundle;
use crate::host::render_cache_store::RenderCacheStore;
use crate::pass::ambient_occlusion_pass::AmbientOcclusionPass;
use crate::pass::backdrop_pass::BackdropPass;
use crate::pass::bloom_pass::BloomPass;
use crate::pass::combine_pass::CombinePass;
use crate::pass::composited_pass::CompositedPass;
use crate::pass::contact_shadow_pass::ContactShadowPass;
use crate::pass::depth_pyramid_pass::DepthPyramidPass;
use crate::pass::enhanced_bloom_pass::EnhancedBloomPass;
use crate::pass::exposure_pass::ExposurePass;
use crate::pass::flare_pass::FlarePass;
use crate::pass::fog_scattering_pass::FogScatteringPass;
use crate::pass::froxel_pass::FroxelPass;
use crate::pass::fsr_pass::FsrPass;
use crate::pass::fxaa_pass::FxaaPass;
use crate::pass::glass_pass::GlassPass;
use crate::pass::grass_colour_pass::GrassColourPass;
use crate::pass::grass_pass::GrassPass;
use crate::pass::grid_pass::GridPass;
use crate::pass::level_passes::LevelPasses;
use crate::pass::light_volume_pass::LightVolumePass;
use crate::pass::lights_pass::LightsPass;
use crate::pass::material_table::MaterialTable;
use crate::pass::overlay_pass::OverlayPass;
use crate::pass::particle_pass::ParticlePass;
use crate::pass::pipeline_source::PipelineSource;
use crate::pass::present_pass::PresentPass;
use crate::pass::puddle_sites_pass::PuddleSitesPass;
use crate::pass::rain_pass::RainPass;
use crate::pass::reflection_pass::ReflectionPass;
use crate::pass::scene_trace_pass::SceneTracePass;
use crate::pass::sky_bindings::SkyBindings;
use crate::pass::sky_haze_pass::SkyHazePass;
use crate::pass::sky_prefilter_pass::SkyPrefilterPass;
use crate::pass::smaa_pass::SmaaPass;
use crate::pass::static_cull_pass::StaticCullPass;
use crate::pass::static_draw_parameters::StaticDrawParameters;
use crate::pass::static_gbuffer_pass::StaticGBufferPass;
use crate::pass::static_shadow_pass::StaticShadowPass;
use crate::pass::sun_pass::SunPass;
use crate::pass::sun_shafts_pass::SunShaftsPass;
use crate::pass::surroundings_pass::SurroundingsPass;
use crate::pass::temporal_pass::TemporalPass;
use crate::pass::thunder_pass::ThunderPass;
use crate::pass::upscale_pass::UpscalePass;
use crate::pass::vbao_pass::VbaoPass;
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
  pub sky_prefilter: SkyPrefilterPass,
  pub surroundings: SurroundingsPass,
  pub water: WaterPass,
  pub composited: CompositedPass,
  pub glass: GlassPass,
  pub grass: GrassPass,
  pub grass_colours: GrassColourPass,
  pub rain: RainPass,
  pub particles: ParticlePass,
  pub wet: WetPass,
  pub thunder: ThunderPass,
  pub flares: FlarePass,
  pub sun_shafts: SunShaftsPass,
  pub light_volumes: LightVolumePass,
  pub froxels: FroxelPass,
  pub bloom: BloomPass,
  pub enhanced_bloom: EnhancedBloomPass,
  /// The pipelines the feature units ask for, made once and forgotten when a shader reloads.
  pub pipelines: Arc<PipelineCache>,
  pub fog_scattering: FogScatteringPass,
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
  pub puddle_sites: PuddleSitesPass,
  pub contact_shadows: ContactShadowPass,
  pub sun: SunPass,
  pub ambient_occlusion: AmbientOcclusionPass,
  pub vbao: VbaoPass,
  pub reflections: ReflectionPass,
  pub scene_trace: SceneTracePass,
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
    (bundle, store): (&dyn RenderBundle, Option<&dyn RenderCacheStore>),
  ) -> XrfResult<Self> {
    let device: &wgpu::Device = &context.device;
    // The driver's pipelines kept from the last run on this adapter and driver, where its backend keeps them.
    let cache_key: Option<String> = store.and(wgpu::util::pipeline_cache_key(&context.adapter.get_info()));
    let cached: Option<Vec<u8>> = store
      .zip(cache_key.as_deref())
      .and_then(|(store, key)| store.read_cached(key));
    let pipelines: Arc<PipelineCache> = Arc::new(PipelineCache::with_driver_cache(device, cached.as_deref()));
    let view_layout: wgpu::BindGroupLayout = ViewBinding::create_layout(device);
    let textures: TextureCache = TextureCache::new(device, &context.queue, workers);
    let sky: SkyBindings = SkyBindings::new(device);
    // What every static draw binds its clusters by, made from the same parameters the draws bind.
    let scene_layout: wgpu::BindGroupLayout = StaticDrawParameters::create_layout(device);
    // The passes compile their pipelines at once, each on a thread of its own, from handles of their own: the state
    // takes the device, the layouts and the caches whole once every pass is built.
    let (device, queue): (wgpu::Device, wgpu::Queue) = (context.device.clone(), context.queue.clone());
    let cache: Arc<PipelineCache> = Arc::clone(&pipelines);
    let (view, scene, texture, sky_layout) = (
      view_layout.clone(),
      scene_layout.clone(),
      textures.get_layout().clone(),
      sky.get_layout().clone(),
    );
    let source: PipelineSource<'_> = PipelineSource {
      cache: &cache,
      device: &device,
      shaders,
    };
    let (device, queue, view, scene, texture, sky_layout) = (&device, &queue, &view, &scene, &texture, &sky_layout);

    let state: Self = std::thread::scope(|threads| -> XrfResult<Self> {
      let static_gbuffer = threads.spawn(move || StaticGBufferPass::new(device, source, view, texture));
      let static_shadow = threads.spawn(move || StaticShadowPass::new(device, source, view, scene, texture));
      let water = threads.spawn(move || WaterPass::new(device, queue, source, [view, scene, texture], bundle));
      let composited = threads.spawn(move || CompositedPass::new(device, source, view, scene, texture, sky_layout));
      let glass = threads.spawn(move || GlassPass::new(device, source, view, scene, texture, sky_layout));
      let grass = threads.spawn(move || GrassPass::new(device, source, view, texture));
      let grass_colours = threads.spawn(move || GrassColourPass::new(device, source, texture));
      let rain = threads.spawn(move || RainPass::new(device, source, view));
      let particles = threads.spawn(move || ParticlePass::new(device, source, view, texture));
      let wet = threads.spawn(move || WetPass::new(device, source, view));
      let thunder = threads.spawn(move || ThunderPass::new(device, source, view));
      let flares = threads.spawn(move || FlarePass::new(device, source, view));
      let sun_shafts = threads.spawn(move || SunShaftsPass::new(device, source, view));
      let light_volumes = threads.spawn(move || LightVolumePass::new(device, source, view, texture));
      let froxels = threads.spawn(move || FroxelPass::new(device, source, view));
      let bloom = threads.spawn(move || BloomPass::new(device, source));
      let fog_scattering = threads.spawn(move || FogScatteringPass::new(device, queue, source, view));
      let temporal = threads.spawn(move || TemporalPass::new(device, source, view));
      let fsr = threads.spawn(move || FsrPass::new(device, source));
      let fxaa = threads.spawn(move || FxaaPass::new(device, source));
      let smaa = threads.spawn(move || SmaaPass::new(device, queue, source, bundle));
      let upscale = threads.spawn(move || UpscalePass::new(device, source));
      let static_cull = threads.spawn(move || StaticCullPass::new(device, source, view));
      let pyramid = threads.spawn(move || DepthPyramidPass::new(device, source));
      let puddle_sites = threads.spawn(move || PuddleSitesPass::new(device, source));
      let contact_shadows = threads.spawn(move || ContactShadowPass::new(device, queue, source, view));
      let sun = threads.spawn(move || SunPass::new(device, source, view));
      let ambient_occlusion = threads.spawn(move || AmbientOcclusionPass::new(device, source, view));
      let vbao = threads.spawn(move || VbaoPass::new(device, queue, source, view));
      let reflections = threads.spawn(move || ReflectionPass::new(device, queue, source, view, sky_layout));
      let scene_trace = threads.spawn(move || SceneTracePass::new(device, source, view, sky_layout, texture));
      let lights = threads.spawn(move || LightsPass::new(device, source, view, texture));
      let combine = threads.spawn(move || CombinePass::new(device, source, view, sky_layout));
      let sky_haze = threads.spawn(move || SkyHazePass::new(device, source, sky_layout));
      let sky_prefilter = threads.spawn(move || SkyPrefilterPass::new(device, queue, source));
      let surroundings = threads.spawn(move || SurroundingsPass::new(device, source, view));
      let exposure = threads.spawn(move || ExposurePass::new(device, source));

      Ok(Self {
        water: join(water)?,
        composited: join(composited)?,
        glass: join(glass)?,
        grass: join(grass)?,
        grass_colours: join(grass_colours)?,
        rain: join(rain)?,
        particles: join(particles)?,
        wet: join(wet)?,
        thunder: join(thunder)?,
        flares: join(flares)?,
        sun_shafts: join(sun_shafts)?,
        light_volumes: join(light_volumes)?,
        froxels: join(froxels)?,
        bloom: join(bloom)?,
        enhanced_bloom: EnhancedBloomPass::new(device, view),
        fog_scattering: join(fog_scattering)?,
        temporal: join(temporal)?,
        fsr: join(fsr)?,
        fxaa: join(fxaa)?,
        smaa: join(smaa)
          .inspect_err(|error| log::error!("SMAA cannot be built, smoothing as FXAA does instead: {error}"))
          .ok(),
        upscale: join(upscale)?,
        static_cull: join(static_cull)?,
        static_gbuffer: join(static_gbuffer)?,
        static_shadow: join(static_shadow)?,
        pyramid: join(pyramid)?,
        puddle_sites: join(puddle_sites)?,
        contact_shadows: join(contact_shadows)?,
        sun: join(sun)?,
        ambient_occlusion: join(ambient_occlusion)?,
        vbao: join(vbao)?,
        reflections: join(reflections)?,
        scene_trace: join(scene_trace)?,
        lights: join(lights)?,
        combine: join(combine)?,
        sky_haze: join(sky_haze)?,
        sky_prefilter: join(sky_prefilter)?,
        surroundings: join(surroundings)?,
        exposure: join(exposure)?,
        present: PresentPass::new(device, source, view),
        backdrop: BackdropPass::new(device, source),
        overlay: OverlayPass::new(device, source, view),
        table: MaterialTable::new(device, queue),
        weather_textures: WeatherTextureCache::new(device, queue, workers),
        windows: HashMap::new(),
        grids: HashMap::new(),
        pipelines,
        sky,
        textures,
        view_layout,
        context,
      })
    })?;

    state
      .get_pipeline_source(shaders)
      .warm(&state.enhanced_bloom.describe_pipelines())?;

    // Every pipeline is made now: kept for the next start.
    if let (Some(store), Some(key), Some(data)) = (store, cache_key, state.pipelines.get_driver_data()) {
      store.write_cached(&key, &data);
    }

    Ok(state)
  }

  /// Where the feature units ask for their pipelines, made from `shaders`.
  pub fn get_pipeline_source<'a>(&'a self, shaders: &'a ShaderLibrary) -> PipelineSource<'a> {
    PipelineSource {
      cache: &self.pipelines,
      device: &self.context.device,
      shaders,
    }
  }

  /// Builds the grid pass drawing into a format, unless it is built.
  ///
  /// # Errors
  ///
  /// Returns an error when its shader does not compose or compile.
  pub fn ensure_grid(&mut self, shaders: &ShaderLibrary, format: wgpu::TextureFormat) -> XrfResult {
    if !self.grids.contains_key(&format) {
      let source: PipelineSource<'_> = self.get_pipeline_source(shaders);
      let grid: GridPass = GridPass::new(&self.context.device, source, &self.view_layout, format)?;

      self.grids.insert(format, grid);
    }

    Ok(())
  }

  /// The grid pass drawing into a format, once [`GpuState::ensure_grid`] built it.
  pub fn get_grid(&self, format: wgpu::TextureFormat) -> Option<&GridPass> {
    self.grids.get(&format)
  }

  pub fn get_level_passes<'a>(&'a self, shaders: &'a ShaderLibrary) -> LevelPasses<'a> {
    LevelPasses {
      pipelines: self.get_pipeline_source(shaders),
      cull: &self.static_cull,
      gbuffer: &self.static_gbuffer,
      shadow: &self.static_shadow,
      pyramid: &self.pyramid,
      puddle_sites: &self.puddle_sites,
      contact_shadows: &self.contact_shadows,
      sun: &self.sun,
      ambient_occlusion: &self.ambient_occlusion,
      vbao: &self.vbao,
      reflections: &self.reflections,
      scene_trace: &self.scene_trace,
      grass_colours: &self.grass_colours,
      lights: &self.lights,
      combine: &self.combine,
      sky_haze: &self.sky_haze,
      sky_prefilter: &self.sky_prefilter,
      surroundings: &self.surroundings,
      sky: &self.sky,
      water: &self.water,
      composited: &self.composited,
      glass: &self.glass,
      grass: &self.grass,
      rain: &self.rain,
      particles: &self.particles,
      wet: &self.wet,
      thunder: &self.thunder,
      flares: &self.flares,
      sun_shafts: &self.sun_shafts,
      light_volumes: &self.light_volumes,
      froxels: &self.froxels,
      bloom: &self.bloom,
      enhanced_bloom: &self.enhanced_bloom,
      fog_scattering: &self.fog_scattering,
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
    self.pipelines.invalidate_all();

    let device: &wgpu::Device = &self.context.device;
    let source: PipelineSource<'_> = PipelineSource {
      cache: &self.pipelines,
      device,
      shaders,
    };

    for grid in self.grids.values_mut() {
      grid.refresh(device, source);
    }

    self.static_cull.refresh(device, source);
    self.static_gbuffer.refresh(device, source);
    self.static_shadow.refresh(device, source);
    self.pyramid.refresh(device, source);
    self.puddle_sites.refresh(device, source);
    self.sky_prefilter.refresh(device, source);
    self.surroundings.refresh(device, source);
    self.contact_shadows.refresh(device, source);
    self.sun.refresh(device, source);
    self.ambient_occlusion.refresh(device, source);
    self.vbao.refresh(device, source);
    self.reflections.refresh(device, source);
    self.scene_trace.refresh(device, source);
    self.grass_colours.refresh(device, source);
    self.lights.refresh(device, source);
    self.combine.refresh(device, source);
    self.sky_haze.refresh(device, source);
    self.water.refresh(device, source);
    self.composited.refresh(device, source);
    self.glass.refresh(device, source);
    self.grass.refresh(device, source);
    self.rain.refresh(device, source);
    self.particles.refresh(device, source);
    self.wet.refresh(device, source);
    self.thunder.refresh(device, source);
    self.flares.refresh(device, source);
    self.sun_shafts.refresh(device, source);
    self.light_volumes.refresh(device, source);
    self.froxels.refresh(device, source);
    self.bloom.refresh(device, source);
    self.fog_scattering.refresh(device, source);
    self.temporal.refresh(device, source);
    self.fsr.refresh(device, source);
    self.fxaa.refresh(device, source);
    if let Some(smaa) = &mut self.smaa {
      smaa.refresh(device, source);
    }
    self.upscale.refresh(device, source);
    self.exposure.refresh(device, source);
    self.present.refresh(source);
    self.backdrop.refresh(source);
    self.overlay.refresh(source);
  }
}

/// What a pass built on a thread of its own came to; a panic there goes on here.
fn join<T>(handle: std::thread::ScopedJoinHandle<'_, T>) -> T {
  handle.join().unwrap_or_else(|panic| std::panic::resume_unwind(panic))
}
