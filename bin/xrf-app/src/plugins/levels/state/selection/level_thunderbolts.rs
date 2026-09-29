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
    let resolver: XraySurfaceResolver = XraySurfaceResolver::open(source.probe, source.scope.clone());
    let animations: Option<LightAnimFile> = read_located_asset(source.probe, ANIMATIONS_FILE)
      .and_then(LightAnimFile::read_from_bytes::<XRayByteOrder>)
      .inspect_err(|error| log::warn!("Thunderbolts are not coloured, '{ANIMATIONS_FILE}' does not read: {error}"))
      .ok();
    let mut read: Self = Self {
      animators: Vec::new(),
      bolts: Vec::new(),
      collections: catalog.thunderbolt_collections.clone(),
      models: Vec::new(),
      settings: catalog
        .thunderbolt_settings
        .as_ref()
        .map(|settings| LevelThunderboltSettings::of(settings, catalog.engine)),
    };
    let mut animator_names: Vec<String> = Vec::new();

    for name in catalog
      .thunderbolt_collections
      .iter()
      .flat_map(|collection| &collection.thunderbolts)
    {
      if read.bolts.iter().any(|it| it.name == *name) {
        continue;
      }

      let Some(bolt) = catalog.find_thunderbolt(name) else {
        continue;
      };
      let text = |key: ThunderboltKey| bolt.get_text(key, catalog.engine);
      let model: Option<u32> = read.find_model(source, &resolver, text(ThunderboltKey::LightningModel));
      let color: Option<u32> = Self::find_animator(
        &mut read.animators,
        &mut animator_names,
        animations.as_ref(),
        text(ThunderboltKey::ColorAnim),
      );

      read.bolts.push(LevelThunderbolt {
        center: Self::read_gradient(source, &resolver, bolt, GradientKeys::CENTER),
        color,
        model,
        name: name.clone(),
        top: Self::read_gradient(source, &resolver, bolt, GradientKeys::TOP),
      });
    }

    read
  }

  /// `CreateModel`: the model under `meshes`, read once however many bolts name it.
  fn find_model(&mut self, source: &LevelWeatherSource, resolver: &XraySurfaceResolver, name: &str) -> Option<u32> {
    if let Some(index) = self.models.iter().position(|it| it.name == name) {
      return Some(index as u32);
    }

    let model = source
      .read_model(&format!("meshes\\{name}"))
      .inspect_err(|error| log::warn!("Thunderbolt model '{name}' is not drawn: {error}"))
      .ok()?;

    self.models.push(LevelThunderboltModel {
      draw: Self::describe_draw(resolver, &model.shader, &model.texture),
      mesh: LevelWeatherModel::of(&model, source.locate(&model.texture)),
      name: name.to_owned(),
    });

    Some((self.models.len() - 1) as u32)
  }

  /// `LALib.FindItem`: the animation by its exact name, added once however many bolts name it.
  fn find_animator(
    animators: &mut Vec<LightAnimatorDescription>,
    names: &mut Vec<String>,
    animations: Option<&LightAnimFile>,
    name: &str,
  ) -> Option<u32> {
    if let Some(index) = names.iter().position(|it| it == name) {
      return Some(index as u32);
    }

    let file: &LightAnimFile = animations?;
    let item = file.items.iter().find(|item| item.name == name)?;

    animators.push(LightAnimatorDescription::of(item, file.is_bgr()));
    names.push(name.to_owned());

    Some((animators.len() - 1) as u32)
  }

  fn read_gradient(
    source: &LevelWeatherSource,
    resolver: &XraySurfaceResolver,
    bolt: &Thunderbolt,
    keys: GradientKeys,
  ) -> LevelThunderboltGradient {
    let engine = source.catalog.engine;
    let texture: &str = bolt.get_text(keys.texture, engine);

    LevelThunderboltGradient {
      draw: Self::describe_draw(resolver, bolt.get_text(keys.shader, engine), texture),
      opacity: bolt.get_number(keys.opacity, engine),
      radius: bolt.get_vector(keys.radius, engine),
      texture: source.locate(texture),
    }
  }

  fn describe_draw(resolver: &XraySurfaceResolver, shader: &str, texture: &str) -> XraySurfaceDraw {
    resolver.describe(shader, &[texture.to_owned()]).draw
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
