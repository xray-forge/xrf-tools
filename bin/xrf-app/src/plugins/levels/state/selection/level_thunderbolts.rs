use std::collections::HashMap;

use serde::Serialize;
use xrf_chunk::XRayByteOrder;
use xrf_environment::{Thunderbolt, ThunderboltCollection, ThunderboltKey};
use xrf_light_anim::LightAnimFile;
use xrf_material::{XraySurfaceDraw, XraySurfaceResolver};
use xrf_visual::LightAnimatorDescription;

use crate::core::assets::read_located_asset;
use crate::plugins::levels::lights::ANIMATIONS_FILE;
use crate::plugins::levels::state::selection::level_thunderbolt::LevelThunderbolt;
use crate::plugins::levels::state::selection::level_thunderbolt_gradient::LevelThunderboltGradient;
use crate::plugins::levels::state::selection::level_thunderbolt_model::LevelThunderboltModel;
use crate::plugins::levels::state::selection::level_thunderbolt_settings::LevelThunderboltSettings;
use crate::plugins::levels::state::selection::level_weather_model::LevelWeatherModel;
use crate::plugins::levels::state::selection::level_weather_source::LevelWeatherSource;

/// What the game's weather strikes with: every collection, the bolts they name as the engine loads them, the models and
/// colour animations those share, and where bolts strike.
#[cfg_attr(feature = "typescript-bindings", derive(specta::Type))]
#[derive(Clone, Debug, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct LevelThunderbolts {
  /// Every collection of the game, which a keyframe set by hand may strike with as well as the level's own.
  pub collections: Vec<ThunderboltCollection>,
  /// Every bolt the collections name that the game has.
  pub bolts: Vec<LevelThunderbolt>,
  pub models: Vec<LevelThunderboltModel>,
  pub animators: Vec<LightAnimatorDescription>,
  /// None where the game has neither `[environment]` nor `[thunderbolt_common]`, which strikes nothing.
  pub settings: Option<LevelThunderboltSettings>,
}

impl LevelThunderbolts {
  /// Every collection and bolt of the catalog, their models read and their textures found in the level's probe.
  pub fn read(source: &LevelWeatherSource) -> Self {
    let catalog = source.catalog;
    let mut reader: BoltReader = BoltReader::open(source);
    let mut bolts: Vec<LevelThunderbolt> = Vec::new();

    for name in catalog
      .thunderbolt_collections
      .iter()
      .flat_map(|collection| &collection.thunderbolts)
    {
      if bolts.iter().any(|it| it.name == *name) {
        continue;
      }

      if let Some(bolt) = catalog.find_thunderbolt(name) {
        bolts.push(reader.read_bolt(name, bolt));
      }
    }

    Self {
      animators: reader.animators,
      bolts,
      collections: catalog.thunderbolt_collections.clone(),
      models: reader.models,
      settings: catalog
        .thunderbolt_settings
        .as_ref()
        .map(|settings| LevelThunderboltSettings::of(settings, catalog.engine)),
    }
  }
}

/// Reads bolts, adding each model and colour animation once however many bolts name it, a missing one looked for once.
struct BoltReader<'s, 'a, 'p> {
  source: &'s LevelWeatherSource<'a, 'p>,
  resolver: XraySurfaceResolver<'a, 'p>,
  animations: Option<LightAnimFile>,
  models: Vec<LevelThunderboltModel>,
  model_indices: HashMap<String, Option<u32>>,
  animators: Vec<LightAnimatorDescription>,
  animator_indices: HashMap<String, Option<u32>>,
}

impl<'s, 'a, 'p> BoltReader<'s, 'a, 'p> {
  fn open(source: &'s LevelWeatherSource<'a, 'p>) -> Self {
    Self {
      animations: read_located_asset(source.probe, ANIMATIONS_FILE)
        .and_then(LightAnimFile::read_from_bytes::<XRayByteOrder>)
        .inspect_err(|error| log::warn!("Thunderbolts are not coloured, '{ANIMATIONS_FILE}' does not read: {error}"))
        .ok(),
      animator_indices: HashMap::new(),
      animators: Vec::new(),
      model_indices: HashMap::new(),
      models: Vec::new(),
      resolver: XraySurfaceResolver::open(source.probe, source.scope.clone()),
      source,
    }
  }

  fn read_bolt(&mut self, name: &str, bolt: &Thunderbolt) -> LevelThunderbolt {
    let engine = self.source.catalog.engine;

    LevelThunderbolt {
      center: self.read_gradient(bolt, GradientKeys::CENTER),
      color: self.find_animator(bolt.get_text(ThunderboltKey::ColorAnim, engine)),
      model: self.find_model(bolt.get_text(ThunderboltKey::LightningModel, engine)),
      name: name.to_owned(),
      top: self.read_gradient(bolt, GradientKeys::TOP),
    }
  }

  /// `CreateModel`: the model under `meshes`.
  fn find_model(&mut self, name: &str) -> Option<u32> {
    if name.is_empty() {
      return None;
    }

    if let Some(index) = self.model_indices.get(name) {
      return *index;
    }

    let found: Option<u32> = self
      .source
      .read_model(&format!("meshes\\{name}"))
      .inspect_err(|error| log::warn!("Thunderbolt model '{name}' is not drawn: {error}"))
      .ok()
      .map(|model| {
        self.models.push(LevelThunderboltModel {
          draw: self.describe_draw(&model.shader, &model.texture),
          mesh: LevelWeatherModel::of(&model, self.source.locate(&model.texture)),
          name: name.to_owned(),
        });

        (self.models.len() - 1) as u32
      });

    self.model_indices.insert(name.to_owned(), found);

    found
  }

  /// `LALib.FindItem`: the animation by its exact name.
  fn find_animator(&mut self, name: &str) -> Option<u32> {
    if name.is_empty() {
      return None;
    }

    if let Some(index) = self.animator_indices.get(name) {
      return *index;
    }

    let found: Option<u32> = self.animations.as_ref().and_then(|file| {
      let item = file.items.iter().find(|item| item.name == name)?;

      self.animators.push(LightAnimatorDescription::of(item, file.is_bgr()));

      Some((self.animators.len() - 1) as u32)
    });

    self.animator_indices.insert(name.to_owned(), found);

    found
  }

  fn read_gradient(&self, bolt: &Thunderbolt, keys: GradientKeys) -> LevelThunderboltGradient {
    let engine = self.source.catalog.engine;
    let texture: &str = bolt.get_text(keys.texture, engine);

    LevelThunderboltGradient {
      draw: self.describe_draw(bolt.get_text(keys.shader, engine), texture),
      opacity: bolt.get_number(keys.opacity, engine),
      radius: bolt.get_vector(keys.radius, engine),
      texture: self.source.locate(texture),
    }
  }

  fn describe_draw(&self, shader: &str, texture: &str) -> XraySurfaceDraw {
    self.resolver.describe(shader, &[texture.to_owned()]).draw
  }
}

/// The keys one of a bolt's two gradients is written under.
#[derive(Clone, Copy)]
struct GradientKeys {
  opacity: ThunderboltKey,
  radius: ThunderboltKey,
  shader: ThunderboltKey,
  texture: ThunderboltKey,
}

impl GradientKeys {
  const TOP: Self = Self {
    opacity: ThunderboltKey::GradientTopOpacity,
    radius: ThunderboltKey::GradientTopRadius,
    shader: ThunderboltKey::GradientTopShader,
    texture: ThunderboltKey::GradientTopTexture,
  };
  const CENTER: Self = Self {
    opacity: ThunderboltKey::GradientCenterOpacity,
    radius: ThunderboltKey::GradientCenterRadius,
    shader: ThunderboltKey::GradientCenterShader,
    texture: ThunderboltKey::GradientCenterTexture,
  };
}
