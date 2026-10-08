use crate::scene::level::feature::bitmask_feature::BitmaskFeature;
use crate::scene::level::feature::bloom_feature::BloomFeature;
use crate::scene::level::feature::exposure_feature::ExposureFeature;
use crate::scene::level::feature::falling_rain_feature::FallingRainFeature;
use crate::scene::level::feature::flares_feature::FlaresFeature;
use crate::scene::level::feature::fog_feature::FogFeature;
use crate::scene::level::feature::reflection_feature::ReflectionFeature;
use crate::scene::level::feature::resolve_feature::ResolveFeature;
use crate::scene::level::feature::smoothing_feature::SmoothingFeature;
use crate::scene::level::feature::thunder_feature::ThunderFeature;
use crate::scene::level::feature::view_feature::ViewFeature;
use crate::scene::level::feature::water_feature::WaterFeature;

/// A view's feature units, each reading its own settings, keeping its own state and adding its own passes.
pub struct SceneFeatures {
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
}

impl SceneFeatures {
  pub fn new(device: &wgpu::Device, queue: &wgpu::Queue) -> Self {
    Self {
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
    }
  }

  /// Every unit, as what each answers alike.
  pub fn list(&self) -> [&dyn ViewFeature; 11] {
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
    ]
  }

  pub fn list_mut(&mut self) -> [&mut dyn ViewFeature; 11] {
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
    ]
  }

  /// Bytes the units hold on the GPU between frames.
  pub fn get_memory(&self) -> u64 {
    self.list().iter().map(|feature| feature.get_memory()).sum()
  }
}
