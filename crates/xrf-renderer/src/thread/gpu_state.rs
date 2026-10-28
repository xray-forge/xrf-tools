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
use crate::pass::calibration_pass::CalibrationPass;
use crate::pass::combine_pass::CombinePass;
use crate::pass::composited_pass::CompositedPass;
use crate::pass::contact_shadow_pass::ContactShadowPass;
use crate::pass::depth_pyramid_pass::DepthPyramidPass;
use crate::pass::distort_pass::DistortPass;
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
use crate::pass::indirect_trace_pass::IndirectTracePass;
use crate::pass::level_passes::LevelPasses;
use crate::pass::light_volume_pass::LightVolumePass;
use crate::pass::lights_pass::LightsPass;
use crate::pass::material_table::MaterialTable;
use crate::pass::meter_pass::MeterPass;
use crate::pass::overlay_pass::OverlayPass;
use crate::pass::particle_pass::ParticlePass;
use crate::pass::pipeline_source::PipelineSource;
use crate::pass::present_pass::PresentPass;
use crate::pass::puddle_sites_pass::PuddleSitesPass;
use crate::pass::rain_pass::RainPass;
use crate::pass::ray_guides_pass::RayGuidesPass;
use crate::pass::reflection_pass::ReflectionPass;
use crate::pass::rtao_pass::RtaoPass;
use crate::pass::scene_trace_pass::SceneTracePass;
use crate::pass::sky_behind_pass::SkyBehindPass;
use crate::pass::sky_bindings::SkyBindings;
use crate::pass::sky_haze_pass::SkyHazePass;
use crate::pass::sky_irradiance_pass::SkyIrradiancePass;
use crate::pass::sky_motion_pass::SkyMotionPass;
use crate::pass::sky_prefilter_pass::SkyPrefilterPass;
use crate::pass::smaa_pass::SmaaPass;
use crate::pass::static_cull_pass::StaticCullPass;
use crate::pass::static_draw_parameters::StaticDrawParameters;
use crate::pass::static_gbuffer_pass::StaticGBufferPass;
use crate::pass::static_shadow_pass::StaticShadowPass;
use crate::pass::sun_pass::SunPass;
use crate::pass::sun_shafts_pass::SunShaftsPass;
use crate::pass::sun_trace_pass::SunTracePass;
use crate::pass::surroundings_pass::SurroundingsPass;
use crate::pass::temporal_pass::TemporalPass;
use crate::pass::thunder_pass::ThunderPass;
use crate::pass::trace_sway_pass::TraceSwayPass;
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
  pub sky_motion: SkyMotionPass,
  pub sky_irradiance: SkyIrradiancePass,
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
  pub rtao: RtaoPass,
  pub indirect_trace: IndirectTracePass,
  pub sun_trace: SunTracePass,
  pub reflections: ReflectionPass,
  pub scene_trace: SceneTracePass,
  pub trace_sway: TraceSwayPass,
  pub lights: LightsPass,
  pub combine: CombinePass,
  pub sky_behind: SkyBehindPass,
  pub ray_guides: RayGuidesPass,
  pub distort: DistortPass,
  pub exposure: ExposurePass,
  pub meter: MeterPass,
  pub calibration: CalibrationPass,
  pub present: PresentPass,
  /// The page's backdrop under every window's viewports.
  pub backdrop: BackdropPass,
  pub overlay: OverlayPass,
  pub table: MaterialTable,
  /// The grid pass for each target format drawn into.
  grids: HashMap<wgpu::TextureFormat, GridPass>,
  /// The name the driver's pipelines are kept under for this adapter and driver, none where its backend keeps none.
  driver_cache_key: Option<String>,
  /// How many bytes of them are kept now, which a save must pass to be worth writing.
  driver_cache_kept: usize,
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

    let mut state: Self = std::thread::scope(|threads| -> XrfResult<Self> {
      let static_gbuffer = threads.spawn(move || StaticGBufferPass::new(source, view, texture));
      let static_shadow = threads.spawn(move || StaticShadowPass::new(source, view, scene, texture));
      let water = threads.spawn(move || WaterPass::new(queue, source, [view, scene, texture], bundle));
      let composited = threads.spawn(move || CompositedPass::new(source, view, scene, texture, sky_layout));
      let glass = threads.spawn(move || GlassPass::new(source, view, scene, texture, sky_layout));
      let grass = threads.spawn(move || GrassPass::new(source, view, texture));
      let grass_colours = threads.spawn(move || GrassColourPass::new(source, texture));
      let rain = threads.spawn(move || RainPass::new(queue, source, view, bundle));
      let particles = threads.spawn(move || ParticlePass::new(source, view, texture));
      let wet = threads.spawn(move || WetPass::new(source, view));
      let thunder = threads.spawn(move || ThunderPass::new(source, view));
      let flares = threads.spawn(move || FlarePass::new(source, view));
      let sun_shafts = threads.spawn(move || SunShaftsPass::new(source, view));
      let light_volumes = threads.spawn(move || LightVolumePass::new(source, view, texture));
      let froxels = threads.spawn(move || FroxelPass::new(source, view));
      let bloom = threads.spawn(move || BloomPass::new(source));
      let fog_scattering = threads.spawn(move || FogScatteringPass::new(queue, source, view));
      let temporal = threads.spawn(move || TemporalPass::new(source, view));
      let fsr = threads.spawn(move || FsrPass::new(source));
      let fxaa = threads.spawn(move || FxaaPass::new(source));
      let smaa = threads.spawn(move || SmaaPass::new(queue, source, bundle));
      let upscale = threads.spawn(move || UpscalePass::new(source));
      let static_cull = threads.spawn(move || StaticCullPass::new(source, view));
      let pyramid = threads.spawn(move || DepthPyramidPass::new(source));
      let puddle_sites = threads.spawn(move || PuddleSitesPass::new(source));
      let contact_shadows = threads.spawn(move || ContactShadowPass::new(queue, source, view));
      let sun = threads.spawn(move || SunPass::new(queue, source, view));
      let sun_trace = threads.spawn(move || SunTracePass::new(source, view, texture));
      let ambient_occlusion = threads.spawn(move || AmbientOcclusionPass::new(source, view));
      let vbao = threads.spawn(move || VbaoPass::new(queue, source, view));
      let rtao = threads.spawn(move || RtaoPass::new(source, view, texture));
      let indirect_trace = threads.spawn(move || IndirectTracePass::new(source, view, sky_layout, texture));
      let reflections = threads.spawn(move || ReflectionPass::new(queue, source, view, sky_layout));
      let scene_trace = threads.spawn(move || SceneTracePass::new(source, view, sky_layout, texture));
      let trace_sway = threads.spawn(move || TraceSwayPass::new(source));
      let lights = threads.spawn(move || LightsPass::new(source, view, texture));
      let combine = threads.spawn(move || CombinePass::new(source, view, sky_layout));
      let sky_behind = threads.spawn(move || SkyBehindPass::new(source, view, sky_layout));
      let ray_guides = threads.spawn(move || RayGuidesPass::new(source, view));
      let distort = threads.spawn(move || DistortPass::new(source, view));
      let sky_haze = threads.spawn(move || SkyHazePass::new(source, sky_layout));
      let sky_motion = threads.spawn(move || SkyMotionPass::new(source, view));
      let sky_irradiance = threads.spawn(move || SkyIrradiancePass::new(source));
      let sky_prefilter = threads.spawn(move || SkyPrefilterPass::new(queue, source));
      let surroundings = threads.spawn(move || SurroundingsPass::new(source, view));
      let exposure = threads.spawn(move || ExposurePass::new(source));
      let meter = threads.spawn(move || MeterPass::new(source, view));
      let calibration = threads.spawn(move || CalibrationPass::new(source, view));

      Ok(Self {
        water: Self::join(water)?,
        composited: Self::join(composited)?,
        glass: Self::join(glass)?,
        grass: Self::join(grass)?,
        grass_colours: Self::join(grass_colours)?,
        rain: Self::join(rain)?,
        particles: Self::join(particles)?,
        wet: Self::join(wet)?,
        thunder: Self::join(thunder)?,
        flares: Self::join(flares)?,
        sun_shafts: Self::join(sun_shafts)?,
        light_volumes: Self::join(light_volumes)?,
        froxels: Self::join(froxels)?,
        bloom: Self::join(bloom)?,
        enhanced_bloom: EnhancedBloomPass::new(device, view),
        fog_scattering: Self::join(fog_scattering)?,
        temporal: Self::join(temporal)?,
        fsr: Self::join(fsr)?,
        fxaa: Self::join(fxaa)?,
        smaa: Self::join(smaa)
          .inspect_err(|error| log::error!("SMAA cannot be built, smoothing as FXAA does instead: {error}"))
          .ok(),
        upscale: Self::join(upscale)?,
        static_cull: Self::join(static_cull)?,
        static_gbuffer: Self::join(static_gbuffer)?,
        static_shadow: Self::join(static_shadow)?,
        pyramid: Self::join(pyramid)?,
        puddle_sites: Self::join(puddle_sites)?,
        contact_shadows: Self::join(contact_shadows)?,
        sun: Self::join(sun)?,
        ambient_occlusion: Self::join(ambient_occlusion)?,
        vbao: Self::join(vbao)?,
        rtao: Self::join(rtao)?,
        indirect_trace: Self::join(indirect_trace)?,
        sun_trace: Self::join(sun_trace)?,
        reflections: Self::join(reflections)?,
        scene_trace: Self::join(scene_trace)?,
        trace_sway: Self::join(trace_sway)?,
        lights: Self::join(lights)?,
        combine: Self::join(combine)?,
        sky_behind: Self::join(sky_behind)?,
        ray_guides: Self::join(ray_guides)?,
        distort: Self::join(distort)?,
        sky_haze: Self::join(sky_haze)?,
        sky_motion: Self::join(sky_motion)?,
        sky_irradiance: Self::join(sky_irradiance)?,
        sky_prefilter: Self::join(sky_prefilter)?,
        surroundings: Self::join(surroundings)?,
        exposure: Self::join(exposure)?,
        meter: Self::join(meter)?,
        calibration: Self::join(calibration)?,
        present: PresentPass::new(source, view),
        backdrop: BackdropPass::new(source),
        overlay: OverlayPass::new(source, view),
        table: MaterialTable::new(device, queue),
        weather_textures: WeatherTextureCache::new(device, queue, workers),
        windows: HashMap::new(),
        grids: HashMap::new(),
        pipelines,
        sky,
        textures,
        view_layout,
        context,
        driver_cache_key: None,
        driver_cache_kept: 0,
      })
    })?;

    state
      .get_pipeline_source(shaders)
      .warm(&state.enhanced_bloom.describe_pipelines())?;

    state.driver_cache_key = cache_key;
    state.driver_cache_kept = cached.map_or(0, |cached| cached.len());
    state.save_driver_cache(store);

    Ok(state)
  }

  /// Keeps the driver's pipelines for the next start where it holds more of them than are kept: the composition, the
  /// grids and the feature units make theirs on demand, so the cache grows long after the start.
  pub fn save_driver_cache(&mut self, store: Option<&dyn RenderCacheStore>) {
    let (Some(store), Some(key)) = (store, self.driver_cache_key.as_deref()) else {
      return;
    };
    let Some(data) = self.pipelines.get_driver_data() else {
      return;
    };

    if data.len() > self.driver_cache_kept {
      store.write_cached(key, &data);
      self.driver_cache_kept = data.len();
    }
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
      let grid: GridPass = GridPass::new(source, &self.view_layout, format)?;

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
      rtao: &self.rtao,
      indirect_trace: &self.indirect_trace,
      sun_trace: &self.sun_trace,
      reflections: &self.reflections,
      scene_trace: &self.scene_trace,
      trace_sway: &self.trace_sway,
      grass_colours: &self.grass_colours,
      lights: &self.lights,
      combine: &self.combine,
      sky_behind: &self.sky_behind,
      ray_guides: &self.ray_guides,
      distort: &self.distort,
      sky_haze: &self.sky_haze,
      sky_motion: &self.sky_motion,
      sky_irradiance: &self.sky_irradiance,
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
      meter: &self.meter,
      calibration: &self.calibration,
      table: &self.table,
    }
  }

  /// Rebuilds the pipelines made from the modules a reload `changed`, or from any module importing one; every other
  /// pipeline stays as it was made.
  pub fn refresh(&mut self, shaders: &ShaderLibrary, changed: &[&str]) {
    for module in shaders.list_dependents(changed) {
      self.pipelines.invalidate_module(module);
    }

    let device: &wgpu::Device = &self.context.device;
    let source: PipelineSource<'_> = PipelineSource {
      cache: &self.pipelines,
      device,
      shaders,
    };

    for grid in self.grids.values_mut() {
      grid.refresh(source);
    }

    self.static_cull.refresh(source);
    self.static_gbuffer.refresh(source);
    self.static_shadow.refresh(source);
    self.pyramid.refresh(source);
    self.puddle_sites.refresh(source);
    self.sky_irradiance.refresh(source);
    self.sky_prefilter.refresh(source);
    self.surroundings.refresh(source);
    self.contact_shadows.refresh(source);
    self.sun.refresh(source);
    self.ambient_occlusion.refresh(source);
    self.vbao.refresh(source);
    self.rtao.refresh(source);
    self.indirect_trace.refresh(source);
    self.sun_trace.refresh(source);
    self.reflections.refresh(source);
    self.scene_trace.refresh(source);
    self.trace_sway.refresh(source);
    self.grass_colours.refresh(source);
    self.lights.refresh(source);
    self.combine.refresh(source);
    self.sky_behind.refresh(source);
    self.ray_guides.refresh(source);
    self.distort.refresh(source);
    self.sky_haze.refresh(source);
    self.sky_motion.refresh(source);
    self.water.refresh(source);
    self.composited.refresh(source);
    self.glass.refresh(source);
    self.grass.refresh(source);
    self.rain.refresh(source);
    self.particles.refresh(source);
    self.wet.refresh(source);
    self.thunder.refresh(source);
    self.flares.refresh(source);
    self.sun_shafts.refresh(source);
    self.light_volumes.refresh(source);
    self.froxels.refresh(source);
    self.bloom.refresh(source);
    self.fog_scattering.refresh(source);
    self.temporal.refresh(source);
    self.fsr.refresh(source);
    self.fxaa.refresh(source);
    if let Some(smaa) = &mut self.smaa {
      smaa.refresh(source);
    }
    self.upscale.refresh(source);
    self.exposure.refresh(source);
    self.meter.refresh(source);
    self.calibration.refresh(source);
    self.present.refresh(source);
    self.backdrop.refresh(source);
    self.overlay.refresh(source);
  }

  /// What a pass built on a thread of its own came to; a panic there goes on here.
  fn join<T>(handle: std::thread::ScopedJoinHandle<'_, T>) -> T {
    handle.join().unwrap_or_else(|panic| std::panic::resume_unwind(panic))
  }
}
