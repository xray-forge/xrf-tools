use std::collections::HashMap;
use std::sync::Arc;

use xrf_error::XrfResult;
use xrf_renderer_core::{PassParameters, PipelineCache};

use crate::context::gpu_context::GpuContext;
use crate::feature::effects::effect_passes::EffectPasses;
use crate::feature::image::image_passes::ImagePasses;
use crate::feature::image::present::present_pass::PresentPass;
use crate::feature::lighting::lighting_passes::LightingPasses;
use crate::feature::scene::scene_passes::ScenePasses;
use crate::feature::scene::statics::draw::static_draw_parameters::StaticDrawParameters;
use crate::feature::tracing::tracing_passes::TracingPasses;
use crate::feature::weather::sky::sky_bindings::SkyBindings;
use crate::feature::weather::weather_passes::WeatherPasses;
use crate::host::render_bundle::RenderBundle;
use crate::host::render_cache_store::RenderCacheStore;
use crate::pass::build::pass_build::PassBuild;
use crate::pass::build::pipeline_source::PipelineSource;
use crate::pass::level_passes::LevelPasses;
use crate::pass::view::view_binding::ViewBinding;
use crate::pass::window::backdrop_pass::BackdropPass;
use crate::pass::window::grid_pass::GridPass;
use crate::pass::window::overlay_pass::OverlayPass;
use crate::scene::texture::texture_cache::TextureCache;
use crate::scene::texture::weather_texture_cache::WeatherTextureCache;
use crate::shader::shader_library::ShaderLibrary;
use crate::thread::queue::render_workers::RenderWorkers;
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
  /// The sky's textures, which the lit and traced passes bind.
  pub sky: SkyBindings,
  /// The pipelines the feature units ask for, made once and forgotten when a shader reloads.
  pub pipelines: Arc<PipelineCache>,
  /// The passes a level's frame runs, by the part of the frame they draw, shared by every view.
  pub scene: ScenePasses,
  pub lighting: LightingPasses,
  pub tracing: TracingPasses,
  pub weather: WeatherPasses,
  pub effects: EffectPasses,
  pub image: ImagePasses,
  /// Each viewport's picture put into its window's rectangle.
  pub present: PresentPass,
  /// The page's backdrop under every window's viewports.
  pub backdrop: BackdropPass,
  pub overlay: OverlayPass,
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
    bundle: &dyn RenderBundle,
    store: Option<&dyn RenderCacheStore>,
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
    // Every stage builds its passes at once, each on a thread of its own, from handles of their own: the state takes
    // the device, the layouts and the caches whole once every pass is built.
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
    let build: PassBuild<'_> = PassBuild {
      source,
      queue: &queue,
      bundle,
      view: &view,
      scene: &scene,
      texture: &texture,
      sky: &sky_layout,
    };

    let mut state: Self = std::thread::scope(|threads| -> XrfResult<Self> {
      let scene = threads.spawn(move || ScenePasses::new(build));
      let lighting = threads.spawn(move || LightingPasses::new(build));
      let tracing = threads.spawn(move || TracingPasses::new(build));
      let weather = threads.spawn(move || WeatherPasses::new(build));
      let effects = threads.spawn(move || EffectPasses::new(build));
      let image = threads.spawn(move || ImagePasses::new(build));

      Ok(Self {
        scene: PassBuild::join(scene)?,
        lighting: PassBuild::join(lighting)?,
        tracing: PassBuild::join(tracing)?,
        weather: PassBuild::join(weather)?,
        effects: PassBuild::join(effects)?,
        image: PassBuild::join(image)?,
        present: PresentPass::new(source, &view),
        backdrop: BackdropPass::new(source),
        overlay: OverlayPass::new(&queue, source, &view, bundle),
        weather_textures: WeatherTextureCache::new(&device, &queue, workers),
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
      .warm(&state.image.enhanced_bloom.describe_pipelines())?;

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
      sky: &self.sky,
      scene: &self.scene,
      lighting: &self.lighting,
      tracing: &self.tracing,
      weather: &self.weather,
      effects: &self.effects,
      image: &self.image,
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

    self.scene.refresh(source);
    self.lighting.refresh(source);
    self.tracing.refresh(source);
    self.weather.refresh(source);
    self.effects.refresh(source);
    self.image.refresh(source);
    self.present.refresh(source);
    self.backdrop.refresh(source);
    self.overlay.refresh(source);
  }
}
