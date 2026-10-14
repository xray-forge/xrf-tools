use crate::scene::level::feature::ambient_occlusion_feature::AmbientOcclusionFeature;
use crate::scene::level::feature::bitmask_feature::BitmaskFeature;
use crate::scene::level::feature::bloom_feature::BloomFeature;
use crate::scene::level::feature::combine_feature::CombineFeature;
use crate::scene::level::feature::composited_feature::CompositedFeature;
use crate::scene::level::feature::contact_shadow_feature::ContactShadowFeature;
use crate::scene::level::feature::exposure_feature::ExposureFeature;
use crate::scene::level::feature::falling_rain_feature::FallingRainFeature;
use crate::scene::level::feature::flares_feature::FlaresFeature;
use crate::scene::level::feature::fog_feature::FogFeature;
use crate::scene::level::feature::lights_feature::LightsFeature;
use crate::scene::level::feature::overlays_feature::OverlaysFeature;
use crate::scene::level::feature::pick_feature::PickFeature;
use crate::scene::level::feature::present_feature::PresentFeature;
use crate::scene::level::feature::reflection_feature::ReflectionFeature;
use crate::scene::level::feature::resolve_feature::ResolveFeature;
use crate::scene::level::feature::sky_feature::SkyFeature;
use crate::scene::level::feature::smoothing_feature::SmoothingFeature;
use crate::scene::level::feature::statics_feature::StaticsFeature;
use crate::scene::level::feature::sun_shafts_feature::SunShaftsFeature;
use crate::scene::level::feature::sun_trace_feature::SunTraceFeature;
use crate::scene::level::feature::thunder_feature::ThunderFeature;
use crate::scene::level::feature::view_feature::ViewFeature;
use crate::scene::level::feature::volumetric_feature::VolumetricFeature;
use crate::scene::level::feature::water_feature::WaterFeature;
use crate::scene::level::feature::wet_feature::WetFeature;
use crate::scene::level::feature::wind_feature::WindFeature;
use crate::scene::level::grass_view::GrassView;
use crate::scene::level::level_shadows::LevelShadows;
use crate::scene::level::level_surface::LevelSurface;
use crate::scene::level::particles_view::ParticlesView;
use crate::scene::level::rain_cover::RainCover;
use crate::scene::level::surroundings_cube::SurroundingsCube;

/// A view's feature units, each reading its own settings, keeping its own state and adding its own passes; and what it
/// draws from views of its own: the sun's shadows, the rain cover and the level's surface seen from overhead, and the
/// level around the camera as a cube.
pub struct SceneFeatures {
  pub shadows: LevelShadows,
  /// The rain cover, while it rains or the surfaces are wet.
  pub rain_cover: Option<RainCover>,
  /// The level's surface around the camera without its trees, which the puddles are seated on, while the enhanced
  /// wetting draws them.
  pub level_surface: Option<LevelSurface>,
  /// What the overhead maps and their views are made with when they are wanted again: the views' layout and the
  /// scene's cull arguments' bytes.
  pub overhead_layout: (wgpu::BindGroupLayout, u64),
  /// The level around the camera as a cube, a face a frame, which a reflection meeting nothing on the screen reads.
  pub surroundings: SurroundingsCube,
  /// The level's local lights as the view lights its frame with them.
  pub lights: LightsFeature,
  /// What it draws over its frame and marks in it: the page's overlays and the selection.
  pub overlays: OverlaysFeature,
  pub pick: PickFeature,
  pub wind: WindFeature,
  /// The level's grass as the view plants and draws it.
  pub grass: GrassView,
  /// The scene's particles as the view sorts and draws them.
  pub particles: ParticlesView,
  pub fog: FogFeature,
  pub bloom: BloomFeature,
  pub smoothing: SmoothingFeature,
  pub exposure: ExposureFeature,
  pub flares: FlaresFeature,
  pub rain: FallingRainFeature,
  pub thunder: ThunderFeature,
  pub resolve: ResolveFeature,
  pub bitmask: BitmaskFeature,
  pub reflections: ReflectionFeature,
  pub water: WaterFeature,
  pub volumetric: VolumetricFeature,
  pub composited: CompositedFeature,
  pub ambient_occlusion: AmbientOcclusionFeature,
  pub contact_shadows: ContactShadowFeature,
  pub sun_trace: SunTraceFeature,
  pub wet: WetFeature,
  pub sky: SkyFeature,
  pub statics: StaticsFeature,
  pub present: PresentFeature,
  pub combine: CombineFeature,
  pub sun_shafts: SunShaftsFeature,
}

impl SceneFeatures {
  /// The units of a view whose views bind by `view_layout`, the scene's cull arguments `args_size` bytes.
  pub fn new(
    (device, queue): (&wgpu::Device, &wgpu::Queue),
    view_layout: &wgpu::BindGroupLayout,
    args_size: u64,
  ) -> Self {
    Self {
      shadows: LevelShadows::default(),
      rain_cover: None,
      level_surface: None,
      overhead_layout: (view_layout.clone(), args_size),
      surroundings: SurroundingsCube::new(device, view_layout, args_size),
      lights: LightsFeature::new(device),
      overlays: OverlaysFeature::default(),
      wind: WindFeature::default(),
      grass: GrassView::new(device),
      particles: ParticlesView::new(device),
      fog: FogFeature::default(),
      bloom: BloomFeature::default(),
      smoothing: SmoothingFeature::default(),
      exposure: ExposureFeature::new(device, queue),
      flares: FlaresFeature::new(),
      rain: FallingRainFeature::default(),
      thunder: ThunderFeature::default(),
      resolve: ResolveFeature::default(),
      bitmask: BitmaskFeature::default(),
      reflections: ReflectionFeature::default(),
      water: WaterFeature::default(),
      volumetric: VolumetricFeature::default(),
      composited: CompositedFeature::default(),
      ambient_occlusion: AmbientOcclusionFeature::default(),
      contact_shadows: ContactShadowFeature::default(),
      sun_trace: SunTraceFeature::default(),
      wet: WetFeature::default(),
      sky: SkyFeature::default(),
      pick: PickFeature::default(),
      statics: StaticsFeature::new(device),
      present: PresentFeature::default(),
      combine: CombineFeature::default(),
      sun_shafts: SunShaftsFeature::default(),
    }
  }

  /// Every unit, as what each answers alike.
  pub fn list(&self) -> [&dyn ViewFeature; 25] {
    [
      &self.fog,
      &self.bloom,
      &self.smoothing,
      &self.exposure,
      &self.flares,
      &self.rain,
      &self.thunder,
      &self.resolve,
      &self.bitmask,
      &self.reflections,
      &self.water,
      &self.volumetric,
      &self.composited,
      &self.ambient_occlusion,
      &self.contact_shadows,
      &self.sun_trace,
      &self.wet,
      &self.sky,
      &self.pick,
      &self.statics,
      &self.present,
      &self.combine,
      &self.shadows,
      &self.sun_shafts,
      &self.surroundings,
    ]
  }

  pub fn list_mut(&mut self) -> [&mut dyn ViewFeature; 25] {
    [
      &mut self.fog,
      &mut self.bloom,
      &mut self.smoothing,
      &mut self.exposure,
      &mut self.flares,
      &mut self.rain,
      &mut self.thunder,
      &mut self.resolve,
      &mut self.bitmask,
      &mut self.reflections,
      &mut self.water,
      &mut self.volumetric,
      &mut self.composited,
      &mut self.ambient_occlusion,
      &mut self.contact_shadows,
      &mut self.sun_trace,
      &mut self.wet,
      &mut self.sky,
      &mut self.pick,
      &mut self.statics,
      &mut self.present,
      &mut self.combine,
      &mut self.shadows,
      &mut self.sun_shafts,
      &mut self.surroundings,
    ]
  }

  /// Bytes the units hold on the GPU between frames, with the overhead maps.
  pub fn get_memory(&self) -> u64 {
    let units: u64 = self.list().iter().map(|feature| feature.get_memory()).sum();

    units
      + self.rain_cover.as_ref().map_or(0, RainCover::get_memory)
      + self.level_surface.as_ref().map_or(0, LevelSurface::get_memory)
  }

  /// Lets go of every unit's GPU state, the overhead maps with them.
  pub fn release(&mut self) {
    for feature in self.list_mut() {
      feature.release();
    }

    self.rain_cover = None;
    self.level_surface = None;
  }
}
