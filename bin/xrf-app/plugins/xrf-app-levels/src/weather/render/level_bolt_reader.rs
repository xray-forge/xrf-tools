use std::collections::HashMap;

use xrf_app_core::assets::read_located_asset;
use xrf_chunk::XRayByteOrder;
use xrf_environment::{Thunderbolt, ThunderboltKey};
use xrf_light_anim::LightAnimFile;
use xrf_material::{XraySurfaceDraw, XraySurfaceResolver};
use xrf_visual::LightAnimatorDescription;

use crate::parts::LevelLightsPart;
use crate::report::LevelReport;
use crate::weather::render::{
  LevelThunderbolt, LevelThunderboltGradient, LevelThunderboltModel, LevelWeatherModel, LevelWeatherSource,
};

/// Reads bolts, adding each model and colour animation once however many bolts name it, a missing one looked for once.
pub(crate) struct LevelBoltReader<'s, 'a> {
  source: &'s LevelWeatherSource<'a>,
  resolver: XraySurfaceResolver,
  animations: Option<LightAnimFile>,
  pub(crate) models: Vec<LevelThunderboltModel>,
  model_indices: HashMap<String, Option<u32>>,
  pub(crate) animators: Vec<LightAnimatorDescription>,
  animator_indices: HashMap<String, Option<u32>>,
}

/// The keys one of a bolt's two gradients is written under.
#[derive(Clone, Copy)]
struct GradientKeys {
  opacity: ThunderboltKey,
  radius: ThunderboltKey,
  shader: ThunderboltKey,
  texture: ThunderboltKey,
}

impl<'s, 'a> LevelBoltReader<'s, 'a> {
  /// The keys a bolt's top gradient is written under.
  const TOP_KEYS: GradientKeys = GradientKeys {
    opacity: ThunderboltKey::GradientTopOpacity,
    radius: ThunderboltKey::GradientTopRadius,
    shader: ThunderboltKey::GradientTopShader,
    texture: ThunderboltKey::GradientTopTexture,
  };

  /// The keys a bolt's centre gradient is written under.
  const CENTER_KEYS: GradientKeys = GradientKeys {
    opacity: ThunderboltKey::GradientCenterOpacity,
    radius: ThunderboltKey::GradientCenterRadius,
    shader: ThunderboltKey::GradientCenterShader,
    texture: ThunderboltKey::GradientCenterTexture,
  };

  /// A reader over the level's weather source, its colour animations read once.
  pub(crate) fn open(source: &'s LevelWeatherSource<'a>) -> Self {
    Self {
      animations: read_located_asset(source.search, LevelLightsPart::ANIMATIONS_FILE)
        .and_then(LightAnimFile::read_from_bytes::<XRayByteOrder>)
        .inspect_err(|error| LevelReport::uncoloured_thunderbolts(LevelLightsPart::ANIMATIONS_FILE, error))
        .ok(),
      animator_indices: HashMap::new(),
      animators: Vec::new(),
      model_indices: HashMap::new(),
      models: Vec::new(),
      resolver: XraySurfaceResolver::open(source.search, source.scope.clone()),
      source,
    }
  }

  /// One bolt as the engine loads it, its model and colour animation added once.
  pub(crate) fn read_bolt(&mut self, name: &str, bolt: &Thunderbolt) -> LevelThunderbolt {
    let engine = self.source.catalog.engine;

    LevelThunderbolt {
      center: self.read_gradient(bolt, Self::CENTER_KEYS),
      color: self.find_animator(bolt.get_text(ThunderboltKey::ColorAnim, engine)),
      model: self.find_model(bolt.get_text(ThunderboltKey::LightningModel, engine)),
      name: name.to_owned(),
      top: self.read_gradient(bolt, Self::TOP_KEYS),
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
      .inspect_err(|error| LevelReport::undrawn_thunderbolt_model(name, error))
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
