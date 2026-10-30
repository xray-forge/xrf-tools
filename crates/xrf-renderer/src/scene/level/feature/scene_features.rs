use crate::scene::level::feature::ambient_occlusion_feature::AmbientOcclusionFeature;
use crate::scene::level::feature::bitmask_feature::BitmaskFeature;
use crate::scene::level::feature::bloom_feature::BloomFeature;
use crate::scene::level::feature::calibration_feature::CalibrationFeature;
use crate::scene::level::feature::camera_feature::CameraFeature;
use crate::scene::level::feature::combine_feature::CombineFeature;
use crate::scene::level::feature::composited_feature::CompositedFeature;
use crate::scene::level::feature::contact_shadow_feature::ContactShadowFeature;
use crate::scene::level::feature::exposure_feature::ExposureFeature;
use crate::scene::level::feature::falling_rain_feature::FallingRainFeature;
use crate::scene::level::feature::flares_feature::FlaresFeature;
use crate::scene::level::feature::fog_feature::FogFeature;
use crate::scene::level::feature::helpers_feature::HelpersFeature;
use crate::scene::level::feature::indirect_trace_feature::IndirectTraceFeature;
use crate::scene::level::feature::lights_feature::LightsFeature;
use crate::scene::level::feature::meter_feature::MeterFeature;
use crate::scene::level::feature::overlays_feature::OverlaysFeature;
use crate::scene::level::feature::pick_feature::PickFeature;
use crate::scene::level::feature::present_feature::PresentFeature;
use crate::scene::level::feature::reflection_feature::ReflectionFeature;
use crate::scene::level::feature::resolve_feature::ResolveFeature;
use crate::scene::level::feature::sky_feature::SkyFeature;
use crate::scene::level::feature::smoothing_feature::SmoothingFeature;
use crate::scene::level::feature::statics_feature::StaticsFeature;
use crate::scene::level::feature::sun_feature::SunFeature;
use crate::scene::level::feature::sun_shafts_feature::SunShaftsFeature;
use crate::scene::level::feature::sun_trace_feature::SunTraceFeature;
use crate::scene::level::feature::thunder_feature::ThunderFeature;
use crate::scene::level::feature::view_feature::ViewFeature;
use crate::scene::level::feature::view_trace_feature::ViewTraceFeature;
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
  pub fog: FogFeature,
  pub bloom: BloomFeature,
  pub smoothing: SmoothingFeature,
  pub exposure: ExposureFeature,
  /// The camera the resolved frame is shown through.
  pub camera: CameraFeature,
  pub meter: MeterFeature,
  pub calibration: CalibrationFeature,
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
  pub sun: SunFeature,
  pub sun_trace: SunTraceFeature,
  pub indirect_trace: IndirectTraceFeature,
  pub view_trace: ViewTraceFeature,
  pub wet: WetFeature,
  pub sky: SkyFeature,
  pub statics: StaticsFeature,
  pub present: PresentFeature,
  pub combine: CombineFeature,
  pub sun_shafts: SunShaftsFeature,
}

impl SceneFeatures {
  /// The units of a view whose views bind by `view_layout`.
  pub fn new((device, queue): (&wgpu::Device, &wgpu::Queue), view_layout: &wgpu::BindGroupLayout) -> Self {
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
      particles: ParticlesView::new(device),
      fog: FogFeature::default(),
      bloom: BloomFeature::default(),
      smoothing: SmoothingFeature::default(),
      exposure: ExposureFeature::new(device, queue),
      camera: CameraFeature::default(),
      meter: MeterFeature::default(),
      calibration: CalibrationFeature::default(),
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
      sun: SunFeature,
      sun_trace: SunTraceFeature::default(),
      indirect_trace: IndirectTraceFeature::default(),
      view_trace: ViewTraceFeature::default(),
      wet: WetFeature::default(),
      sky: SkyFeature::default(),
      pick: PickFeature::default(),
      statics: StaticsFeature::new(device),
      present: PresentFeature::default(),
      combine: CombineFeature::default(),
      sun_shafts: SunShaftsFeature::default(),
    }
  }

  /// Bytes the units hold on the GPU between frames, with the overhead maps.
  pub fn get_memory(&self) -> u64 {
    let units: u64 = self.list().iter().map(|feature| feature.get_memory()).sum();

    units
      + self.rain_cover.as_ref().map_or(0, RainCover::get_memory)
      + self.level_surface.as_ref().map_or(0, LevelSurface::get_memory)
  }
}

/// The units' two lists made from one, so neither names a unit the other leaves out.
macro_rules! scene_units {
  ($($unit:ident),+ $(,)?) => {
    impl SceneFeatures {
      /// How many units a view has.
      pub const UNIT_COUNT: usize = [$(stringify!($unit)),+].len();

      /// Every unit, as what each answers alike.
      pub fn list(&self) -> [&dyn ViewFeature; Self::UNIT_COUNT] {
        [$(&self.$unit),+]
      }

      /// Every unit, to let go of or change alike.
      pub fn list_mut(&mut self) -> [&mut dyn ViewFeature; Self::UNIT_COUNT] {
        [$(&mut self.$unit),+]
      }
    }
  };
}

scene_units!(
  lights,
  overlays,
  helpers,
  wind,
  grass,
  particles,
  fog,
  bloom,
  smoothing,
  exposure,
  camera,
  meter,
  calibration,
  flares,
  rain,
  thunder,
  resolve,
  bitmask,
  reflections,
  water,
  volumetric,
  composited,
  ambient_occlusion,
  contact_shadows,
  sun,
  sun_trace,
  indirect_trace,
  view_trace,
  wet,
  sky,
  pick,
  statics,
  present,
  combine,
  shadows,
  sun_shafts,
  surroundings,
);
