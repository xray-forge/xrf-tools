use xrf_environment::ThunderboltCollection;
use xrf_renderer::{
  RenderThunder, RenderThunderSettings, RenderThunderbolt, RenderThunderboltGradient, RenderThunderboltModel,
};
use xrf_visual::LightAnimatorDescription;

use crate::weather::LevelWeatherSource;
use crate::weather::render::thunderbolt::level_bolt_reader::LevelBoltReader;
use crate::weather::render::{
  LevelThunderbolt, LevelThunderboltGradient, LevelThunderboltModel, LevelThunderboltSettings,
};

/// What the game's weather strikes with: every collection, the bolts they name as the engine loads them, the models and
/// colour animations those share, and where bolts strike.
#[derive(Clone, Debug, PartialEq)]
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
  /// Every collection and bolt of the catalog, their models read and their textures found in the level's search.
  pub fn read(source: &LevelWeatherSource) -> Self {
    let catalog = source.catalog;
    let mut reader: LevelBoltReader = LevelBoltReader::open(source);
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

impl From<LevelThunderbolts> for RenderThunder {
  /// What the level's weather strikes with as the renderer takes it, every texture by reference.
  fn from(thunder: LevelThunderbolts) -> Self {
    let gradient = |gradient: LevelThunderboltGradient| RenderThunderboltGradient {
      opacity: gradient.opacity,
      radius: gradient.radius,
      texture: gradient.texture.reference,
      draw: gradient.draw,
    };

    Self {
      settings: thunder.settings.map(|settings| RenderThunderSettings {
        altitude: settings.altitude,
        delta_longitude: settings.delta_longitude,
        min_distance: settings.min_distance,
        tilt: settings.tilt,
        second_probability: settings.second_probability,
        sky_color: settings.sky_color,
        sun_color: settings.sun_color,
        fog_color: settings.fog_color,
      }),
      collections: thunder
        .collections
        .into_iter()
        .map(|collection| (collection.name, collection.thunderbolts))
        .collect(),
      bolts: thunder
        .bolts
        .into_iter()
        .map(|bolt| {
          (
            bolt.name,
            RenderThunderbolt {
              model: bolt.model.map(|index| index as usize),
              color: bolt.color.map(|index| index as usize),
              top: gradient(bolt.top),
              center: gradient(bolt.center),
            },
          )
        })
        .collect(),
      models: thunder
        .models
        .into_iter()
        .map(|model| RenderThunderboltModel {
          mesh: model.mesh.into(),
          draw: model.draw,
        })
        .collect(),
      animators: thunder.animators,
    }
  }
}
