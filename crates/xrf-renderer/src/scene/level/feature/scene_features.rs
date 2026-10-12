use crate::scene::level::feature::ambient_occlusion_feature::AmbientOcclusionFeature;
use crate::scene::level::feature::bitmask_feature::BitmaskFeature;
use crate::scene::level::feature::bloom_feature::BloomFeature;
use crate::scene::level::feature::composited_feature::CompositedFeature;
use crate::scene::level::feature::contact_shadow_feature::ContactShadowFeature;
use crate::scene::level::feature::exposure_feature::ExposureFeature;
use crate::scene::level::feature::falling_rain_feature::FallingRainFeature;
use crate::scene::level::feature::flares_feature::FlaresFeature;
use crate::scene::level::feature::fog_feature::FogFeature;
use crate::scene::level::feature::lights_feature::LightsFeature;
use crate::scene::level::feature::reflection_feature::ReflectionFeature;
use crate::scene::level::feature::resolve_feature::ResolveFeature;
use crate::scene::level::feature::sky_feature::SkyFeature;
use crate::scene::level::feature::smoothing_feature::SmoothingFeature;
use crate::scene::level::feature::thunder_feature::ThunderFeature;
use crate::scene::level::feature::view_feature::ViewFeature;
use crate::scene::level::feature::volumetric_feature::VolumetricFeature;
use crate::scene::level::feature::water_feature::WaterFeature;
use crate::scene::level::feature::wet_feature::WetFeature;
use crate::scene::level::level_shadows::LevelShadows;
use crate::scene::level::level_surface::LevelSurface;
use crate::scene::level::rain_cover::RainCover;
use crate::scene::level::surroundings_cube::SurroundingsCube;

/// A view's feature units, each reading its own settings, keeping its own state and adding its own passes; and what it
/// draws from views of its own: the sun's shadows, the rain cover and the level's surface seen from overhead, and the
/// level around the camera as a cube.
pub struct SceneFeatures {
  pub shadows: LevelShadows,
  pub rain_cover: RainCover,
  /// The level's surface around the camera without its trees, which the puddles are seated on.
  pub level_surface: LevelSurface,
  /// The level around the camera as a cube, a face a frame, which a reflection meeting nothing on the screen reads.
  pub surroundings: SurroundingsCube,
  /// The level's local lights as the view lights its frame with them.
  pub lights: LightsFeature,
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
  pub wet: WetFeature,
  pub sky: SkyFeature,
}

impl SceneFeatures {
  /// The units of a view whose views bind by `view_layout`, the scene's cull arguments `args_size` bytes.
  pub fn new(
    (device, queue): (&wgpu::Device, &wgpu::Queue),
    view_layout: &wgpu::BindGroupLayout,
    args_size: u64,
  ) -> Self {
    Self {
      shadows: LevelShadows::new(device),
      rain_cover: RainCover::new(device, view_layout, args_size),
      level_surface: LevelSurface::new(device, view_layout, args_size),
      surroundings: SurroundingsCube::new(device, view_layout, args_size),
      lights: LightsFeature::new(device),
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
      wet: WetFeature::default(),
      sky: SkyFeature::default(),
    }
  }

  /// Every unit, as what each answers alike.
  pub fn list(&self) -> [&dyn ViewFeature; 17] {
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
      &self.wet,
      &self.sky,
    ]
  }

  pub fn list_mut(&mut self) -> [&mut dyn ViewFeature; 17] {
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
      &mut self.wet,
      &mut self.sky,
    ]
  }

  /// Bytes the units hold on the GPU between frames.
  pub fn get_memory(&self) -> u64 {
    self.list().iter().map(|feature| feature.get_memory()).sum()
  }
}
