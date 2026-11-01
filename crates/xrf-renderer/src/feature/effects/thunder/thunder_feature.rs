use xrf_material::XraySurfaceDraw;
use xrf_renderer_core::{GraphDeclaration, StorageArray, UniformBinding};

use crate::feature::effects::over_scene::add_over_scene;
use crate::feature::effects::thunder::thunder_models::ThunderModels;
use crate::feature::effects::thunder::thunder_parameters::ThunderParameters;
use crate::feature::effects::thunder::thunder_strike::ThunderStrike;
use crate::feature::effects::thunder::thunder_uniform::ThunderUniform;
use crate::feature::effects::weather_built::WeatherBuilt;
use crate::feature::effects::weather_model_buffers::WeatherModelBuffers;
use crate::feature::view::view_feature::ViewFeature;
use crate::pass::frame_lighting::lighting_uniform::LightingUniform;
use crate::scene::level::view::record::view_record::ViewRecord;
use crate::scene::level::view::view_prepare::ViewPrepare;
use crate::scene::texture::weather_texture_kind::WeatherTextureKind;

/// The strike, while a bolt strikes and the view shows it: every bolt model built for the level's weather, the uniform
/// where the strike stands, and what its model and its two glows bind and draw.
#[derive(Default)]
pub struct ThunderFeature {
  models: Option<WeatherBuilt<ThunderModels>>,
  uniform: ThunderUniform,
  /// None while no bolt strikes.
  strike: Option<ThunderStrike>,
}

impl ThunderFeature {
  /// Takes this frame's strike, while a bolt strikes and the view is lit: every bolt model built for the level's
  /// weather, the uniform where the strike stands, and what its model and glows bind.
  pub fn prepare(
    &mut self,
    &ViewPrepare {
      device,
      options,
      lighting,
      weather,
      weather_textures,
      ..
    }: &ViewPrepare<'_>,
  ) {
    self.strike = None;

    let (Some(weather), Some(strike)) = (weather, &lighting.thunderbolt) else {
      return;
    };
    let Some(thunder) = weather.thunder.as_ref().filter(|_| options.mode.is_lit) else {
      return;
    };
    let Some(bolt) = thunder.bolts.get(&strike.bolt) else {
      return;
    };

    let models: &ThunderModels = WeatherBuilt::follow(&mut self.models, weather, || ThunderModels {
      bolts: thunder
        .models
        .iter()
        .map(|model| WeatherModelBuffers::new(device, Some(&model.mesh)))
        .collect(),
      empty: WeatherModelBuffers::new(device, None),
    });
    let model = bolt
      .model
      .and_then(|index| Some((models.bolts.get(index)?, thunder.models.get(index)?)));
    let flat = |reference: &str| {
      weather_textures
        .get_view(Some(reference), WeatherTextureKind::Flat)
        .clone()
    };
    let model_texture: &str = model.map_or("", |(_, model)| model.mesh.texture.as_str());

    self.uniform = ThunderUniform::new(strike);
    self.strike = Some(ThunderStrike {
      textures: [flat(model_texture), flat(&bolt.top.texture), flat(&bolt.center.texture)],
      draws: [
        model.map_or(XraySurfaceDraw::Opaque, |(_, model)| model.draw),
        bolt.top.draw,
        bolt.center.draw,
      ],
      model: bolt.model.filter(|_| model.is_some()),
      model_indices: model.map_or(0, |(buffers, _)| buffers.index_count),
    });
  }

  /// Adds the bolt's model with its texture, then each glow's quad with its own, over the scene, through the frame's
  /// `lighting`; nothing while no bolt strikes.
  pub fn add_passes<'a>(
    &'a self,
    GraphDeclaration {
      graph,
      bindings,
      runtime,
    }: GraphDeclaration<'_, 'a>,
    ViewRecord {
      passes,
      view,
      handles,
      lit,
      ..
    }: ViewRecord<'a>,
  ) {
    let lighting: UniformBinding<LightingUniform> = lit.lighting;
    let (Some(strike), Some(models)) = (&self.strike, &self.models) else {
      return;
    };
    let models: &ThunderModels = models.get();
    let thunder: UniformBinding<ThunderUniform> = runtime.push_uniform(&self.uniform);
    let entries: [ThunderParameters; 3] =
      [(0, models.get(strike.model)), (1, &models.empty), (2, &models.empty)].map(|(entry, buffers)| {
        ThunderParameters {
          thunder,
          thunder_texture: bindings.import_view(&mut *graph, "thunder texture", &strike.textures[entry]),
          thunder_sampler: passes.effects.thunder.get_sampler(),
          model_vertices: StorageArray::new(bindings.import_buffer(&mut *graph, "thunder vertices", &buffers.vertices)),
          model_indices: StorageArray::new(bindings.import_buffer(&mut *graph, "thunder indices", &buffers.indices)),
          lighting,
        }
      });
    let (draws, model_indices) = (strike.draws, strike.model_indices);

    entries
      .iter()
      .fold(
        add_over_scene(&mut *graph, handles, "thunder"),
        |builder, parameters| builder.parameters(parameters),
      )
      .record(move |context| {
        passes
          .effects
          .thunder
          .record(context, view, &entries, draws, model_indices)
      });
  }
}

impl ViewFeature for ThunderFeature {
  /// The bolt models and the empty model, kept for the level's weather.
  fn get_memory(&self) -> u64 {
    self.models.as_ref().map_or(0, |models| models.get().get_memory())
  }

  fn release(&mut self) {
    *self = Self::default();
  }
}
