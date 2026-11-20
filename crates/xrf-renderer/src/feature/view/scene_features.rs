use crate::feature::effects::composited::composited_feature::CompositedFeature;
use crate::feature::effects::particles::particles_view::ParticlesView;
use crate::feature::effects::rain::cover::rain_cover::RainCover;
use crate::feature::effects::rain::falling_rain_feature::FallingRainFeature;
use crate::feature::image::bloom::bloom_feature::BloomFeature;
use crate::feature::image::exposure::exposure_feature::ExposureFeature;
use crate::feature::image::present::present_feature::PresentFeature;
use crate::feature::image::resolve::resolve_feature::ResolveFeature;
use crate::feature::lighting::lights::lights_feature::LightsFeature;
use crate::feature::lighting::shadows::level_shadows::LevelShadows;
use crate::feature::page::helpers_feature::HelpersFeature;
use crate::feature::page::overlays_feature::OverlaysFeature;
use crate::feature::page::pick_feature::PickFeature;
use crate::feature::scene::grass::grass_view::GrassView;
use crate::feature::scene::statics::statics_feature::StaticsFeature;
use crate::feature::scene::wind::wind_feature::WindFeature;
use crate::feature::tracing::reflections::reflection_feature::ReflectionFeature;
use crate::feature::tracing::surroundings::surroundings_cube::SurroundingsCube;
use crate::feature::tracing::view_trace::view_trace_feature::ViewTraceFeature;
use crate::feature::view::view_units::view_units;
use crate::feature::weather::sky::sky_feature::SkyFeature;
use crate::feature::weather::water::water_feature::WaterFeature;
use crate::feature::weather::wet::puddles::level_surface::LevelSurface;
use crate::feature::weather::wet::wet_feature::WetFeature;

/// A view's units shared across its frame's parts: those recording in more than one of its phases (the resolve, the
/// statics, the grass, the lights, the sky, the water, the wet surfaces, the rain and the reflections), those every
/// part reads (the exposure, the bloom, the composited surfaces, the wind, the scene's trace and the particles), the
/// present every part's decision reaches, and the page's overlays, helpers and picks; and what it draws from views of
/// its own: the sun's shadows, the rain cover and the level's surface seen from overhead, and the level around the
/// camera as a cube. The units drawing in one part alone are its stages' (`ViewStages`).
pub(crate) struct SceneFeatures {
  pub shadows: LevelShadows,
  /// The rain cover, while it rains or the surfaces are wet.
  pub rain_cover: Option<RainCover>,
  /// The level's surface around the camera without its trees, which the puddles are seated on, while the enhanced
  /// wetting draws them.
  pub level_surface: Option<LevelSurface>,
  /// The layout the overhead maps' views are made with when they are wanted again.
  pub overhead_layout: wgpu::BindGroupLayout,
  /// The level around the camera as a cube, a face a frame, which a reflection meeting nothing on the screen reads.
  pub surroundings: SurroundingsCube,
  /// The level's local lights as the view lights its frame with them.
  pub lights: LightsFeature,
  /// What it draws over its frame and marks in it: the page's overlays and the selection.
  pub overlays: OverlaysFeature,
  /// The level objects' helpers it draws over its frame and picks.
  pub helpers: HelpersFeature,
  pub pick: PickFeature,
  pub wind: WindFeature,
  /// The level's grass as the view plants and draws it.
  pub grass: GrassView,
  /// The scene's particles as the view sorts and draws them.
  pub particles: ParticlesView,
  pub bloom: BloomFeature,
  pub exposure: ExposureFeature,
  pub rain: FallingRainFeature,
  pub resolve: ResolveFeature,
  pub reflections: ReflectionFeature,
  pub water: WaterFeature,
  pub composited: CompositedFeature,
  /// The scene as the view's rays meet it, which the lighting, the reflections and the water trace through.
  pub view_trace: ViewTraceFeature,
  pub wet: WetFeature,
  pub sky: SkyFeature,
  pub statics: StaticsFeature,
  pub present: PresentFeature,
}

impl SceneFeatures {
  /// The units of a view whose views bind by `view_layout`.
  pub(crate) fn new(device: &wgpu::Device, queue: &wgpu::Queue, view_layout: &wgpu::BindGroupLayout) -> Self {
    Self {
      shadows: LevelShadows::default(),
      rain_cover: None,
      level_surface: None,
      overhead_layout: view_layout.clone(),
      surroundings: SurroundingsCube::new(device, view_layout),
      lights: LightsFeature::new(device),
      overlays: OverlaysFeature::default(),
      helpers: HelpersFeature::default(),
      wind: WindFeature::default(),
      grass: GrassView::new(device),
      particles: ParticlesView::default(),
      bloom: BloomFeature::default(),
      exposure: ExposureFeature::new(device, queue),
      rain: FallingRainFeature::default(),
      resolve: ResolveFeature::default(),
      reflections: ReflectionFeature::default(),
      water: WaterFeature::default(),
      composited: CompositedFeature::default(),
      view_trace: ViewTraceFeature::default(),
      wet: WetFeature::default(),
      sky: SkyFeature::default(),
      pick: PickFeature::default(),
      statics: StaticsFeature::new(device),
      present: PresentFeature::default(),
    }
  }
}

view_units!(SceneFeatures {
  lights,
  overlays,
  helpers,
  wind,
  grass,
  particles,
  bloom,
  exposure,
  rain,
  resolve,
  reflections,
  water,
  composited,
  view_trace,
  wet,
  sky,
  pick,
  statics,
  present,
  shadows,
  surroundings,
});
