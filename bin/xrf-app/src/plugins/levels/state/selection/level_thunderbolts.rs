use serde::Serialize;
use xrf_environment::{
  EnvironmentCatalog, Thunderbolt, ThunderboltCollection, ThunderboltSettings, WeatherCycle, WeatherKey,
};

/// What a level's weather strikes with: the collections its cycles and effects name, their bolts, and where every
/// bolt is struck from.
#[cfg_attr(feature = "typescript-bindings", derive(specta::Type))]
#[derive(Clone, Debug, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct LevelThunderbolts {
  pub collections: Vec<ThunderboltCollection>,
  pub thunderbolts: Vec<Thunderbolt>,
  pub settings: Option<ThunderboltSettings>,
}

impl LevelThunderbolts {
  /// The collections some cycles strike with, each once, their bolts, and the catalog's strike settings.
  pub fn of<'a>(catalog: &EnvironmentCatalog, cycles: impl Iterator<Item = &'a WeatherCycle>) -> Self {
    let mut collections: Vec<ThunderboltCollection> = Vec::new();

    for keyframe in cycles.flat_map(|cycle| &cycle.keyframes) {
      let name: &str = keyframe
        .section
        .get_text(WeatherKey::ThunderboltCollection, catalog.engine);

      if !name.is_empty()
        && !collections.iter().any(|it| it.name == name)
        && let Some(collection) = catalog.find_thunderbolt_collection(name)
      {
        collections.push(collection.clone());
      }
    }

    let mut thunderbolts: Vec<Thunderbolt> = Vec::new();

    for bolt in collections.iter().flat_map(|collection| &collection.thunderbolts) {
      if !thunderbolts.iter().any(|it| it.name == *bolt)
        && let Some(thunderbolt) = catalog.find_thunderbolt(bolt)
      {
        thunderbolts.push(thunderbolt.clone());
      }
    }

    Self {
      collections,
      settings: catalog.thunderbolt_settings.clone(),
      thunderbolts,
    }
  }
}
