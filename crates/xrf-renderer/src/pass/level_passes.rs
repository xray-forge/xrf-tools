use crate::feature::effects::effect_passes::EffectPasses;
use crate::feature::image::image_passes::ImagePasses;
use crate::feature::weather::sky::sky_bindings::SkyBindings;
use crate::feature::weather::weather_passes::WeatherPasses;
use crate::pass::pipeline_source::PipelineSource;
use crate::pass::stage::lighting_passes::LightingPasses;
use crate::pass::stage::scene_passes::ScenePasses;
use crate::pass::stage::tracing_passes::TracingPasses;

/// The passes a level's frame runs, borrowed from the GPU state together, by the part of the frame they draw.
#[derive(Clone, Copy)]
pub struct LevelPasses<'a> {
  /// Where the feature units ask for their pipelines.
  pub pipelines: PipelineSource<'a>,
  /// The sky's textures, which the lit and traced passes bind.
  pub sky: &'a SkyBindings,
  pub scene: &'a ScenePasses,
  pub lighting: &'a LightingPasses,
  pub tracing: &'a TracingPasses,
  pub weather: &'a WeatherPasses,
  pub effects: &'a EffectPasses,
  pub image: &'a ImagePasses,
}
