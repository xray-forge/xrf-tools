use std::ptr;
use std::sync::{Arc, Weak};

use xrf_material::XraySurfaceDraw;
use xrf_renderer_core::{GraphDeclaration, StorageArray, UniformBinding};

use crate::contract::render_view_options::RenderViewOptions;
use crate::frame::view_target_handles::ViewTargetHandles;
use crate::host::render_level_weather::RenderLevelWeather;
use crate::lighting::render_lighting::RenderLighting;
use crate::pass::level_passes::LevelPasses;
use crate::pass::lighting_uniform::LightingUniform;
use crate::pass::thunder_parameters::ThunderParameters;
use crate::pass::thunder_uniform::ThunderUniform;
use crate::pass::view_binding::ViewBinding;
use crate::scene::level::feature::over_scene::add_over_scene;
use crate::scene::level::feature::view_feature::ViewFeature;
use crate::scene::level::weather_model_buffers::WeatherModelBuffers;
use crate::scene::texture::weather_texture_kind::WeatherTextureKind;
use crate::scene::texture::weather_textures::WeatherTextures;

/// The strike, while a bolt strikes and the view shows it: every bolt model built for the level's weather, the uniform
/// where the strike stands, and what its model and its two glows bind and draw.
#[derive(Default)]
pub struct ThunderFeature {
  /// Every bolt model of the level's weather, with the weather they were built for (held weakly, so its address is
  /// never another's), and an empty model, which the glows' quads and a bolt without a model bind.
  models: Option<(Weak<RenderLevelWeather>, Vec<WeatherModelBuffers>, WeatherModelBuffers)>,
  uniform: ThunderUniform,
  /// The strike's model's, its top glow's and its centre glow's textures, and the bolt model it draws, by its index
  /// among the weather's.
  views: Option<([wgpu::TextureView; 3], Option<usize>)>,
  /// How the model and the two glows draw, and the model's indices, none while no bolt strikes.
  draw: Option<([XraySurfaceDraw; 3], u32)>,
}

impl ThunderFeature {
  /// Takes this frame's strike, while a bolt strikes and the view is lit: every bolt model built for the level's
  /// weather, the uniform where the strike stands, and what its model and glows bind.
  pub fn prepare(
    &mut self,
    device: &wgpu::Device,
    (lighting, weather): (&RenderLighting, Option<&Arc<RenderLevelWeather>>),
    options: &RenderViewOptions,
    weather_textures: WeatherTextures<'_>,
  ) {
    self.draw = None;
    self.views = None;

    let (Some(weather), Some(strike)) = (weather, &lighting.thunderbolt) else {
      return;
    };
    let Some(thunder) = weather.thunder.as_ref().filter(|_| options.mode.is_lit) else {
      return;
    };
    let Some(bolt) = thunder.bolts.get(&strike.bolt) else {
      return;
    };

    if self
      .models
      .as_ref()
      .is_none_or(|(built, ..)| !ptr::eq(built.as_ptr(), Arc::as_ptr(weather)))
    {
      let models: Vec<WeatherModelBuffers> = thunder
        .models
        .iter()
        .map(|model| WeatherModelBuffers::new(device, Some(&model.mesh)))
        .collect();

      self.models = Some((Arc::downgrade(weather), models, WeatherModelBuffers::new(device, None)));
    }

    let Some((_, models, _)) = &self.models else {
      return;
    };
    let model = bolt
      .model
      .and_then(|index| Some((models.get(index)?, thunder.models.get(index)?)));
    let flat = |reference: &str| {
      weather_textures
        .get_view(Some(reference), WeatherTextureKind::Flat)
        .clone()
    };
    let model_texture: &str = model.map_or("", |(_, model)| model.mesh.texture.as_str());

    self.uniform = ThunderUniform::new(strike);
    self.views = Some((
      [flat(model_texture), flat(&bolt.top.texture), flat(&bolt.center.texture)],
      bolt.model.filter(|_| model.is_some()),
    ));
    self.draw = Some((
      [
        model.map_or(XraySurfaceDraw::Opaque, |(_, model)| model.draw),
        bolt.top.draw,
        bolt.center.draw,
      ],
      model.map_or(0, |(buffers, _)| buffers.index_count),
    ));
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
    passes: LevelPasses<'a>,
    view: &'a ViewBinding,
    (handles, lighting): (ViewTargetHandles, UniformBinding<LightingUniform>),
  ) {
    let (Some((draws, model_indices)), Some((textures, model)), Some((_, models, no_model))) =
      (self.draw, &self.views, &self.models)
    else {
      return;
    };
    let thunder: UniformBinding<ThunderUniform> = runtime.push_uniform(&self.uniform);
    let bolt: &WeatherModelBuffers = model.and_then(|index| models.get(index)).unwrap_or(no_model);
    let entries: [ThunderParameters; 3] =
      [(0, bolt), (1, no_model), (2, no_model)].map(|(entry, buffers)| ThunderParameters {
        thunder,
        thunder_texture: bindings.import_view(&mut *graph, "thunder texture", &textures[entry]),
        thunder_sampler: passes.thunder.get_sampler(),
        model_vertices: StorageArray::new(bindings.import_buffer(&mut *graph, "thunder vertices", &buffers.vertices)),
        model_indices: StorageArray::new(bindings.import_buffer(&mut *graph, "thunder indices", &buffers.indices)),
        lighting,
      });

    entries
      .iter()
      .fold(
        add_over_scene(&mut *graph, handles, "thunder"),
        |builder, parameters| builder.parameters(parameters),
      )
      .record(move |context| passes.thunder.record(context, view, &entries, draws, model_indices));
  }
}

impl ViewFeature for ThunderFeature {
  /// The bolt models and the empty model, kept for the level's weather.
  fn get_memory(&self) -> u64 {
    self.models.as_ref().map_or(0, |(_, models, no_model)| {
      models
        .iter()
        .chain([no_model])
        .map(|model| model.vertices.size() + model.indices.size())
        .sum()
    })
  }

  fn release(&mut self) {
    *self = Self::default();
  }
}
