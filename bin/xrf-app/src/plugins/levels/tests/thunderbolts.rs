//! What the game's weather strikes with, as the open level finds it.

use xrf_engine_target::XrayEngine;
use xrf_environment::{
  EnvironmentCatalog, EnvironmentValue, Thunderbolt, ThunderboltCollection, ThunderboltKey, WeatherGraphs,
};
use xrf_material::XrayTextureScope;
use xrf_material::fixtures::FixtureTree;
use xrf_vfs::{XrayLookupScope, XrayMountId, XrayProbe, XrayVfs};

use crate::plugins::levels::state::selection::level_thunderbolts::LevelThunderbolts;
use crate::plugins::levels::state::selection::level_weather_source::LevelWeatherSource;

const FILE: &str = "environment\\thunderbolts.ltx";

fn new_collection(name: &str, thunderbolts: &[&str]) -> ThunderboltCollection {
  ThunderboltCollection {
    file: String::from("environment\\thunderbolt_collections.ltx"),
    name: name.to_owned(),
    thunderbolts: thunderbolts.iter().map(|bolt| (*bolt).to_owned()).collect(),
  }
}

fn new_bolt(name: &str, model: &str, color: &str) -> Thunderbolt {
  let mut bolt: Thunderbolt = Thunderbolt::new(name, FILE);

  bolt.values.extend([
    (ThunderboltKey::LightningModel, EnvironmentValue::Text(model.to_owned())),
    (ThunderboltKey::ColorAnim, EnvironmentValue::Text(color.to_owned())),
  ]);

  bolt
}

fn new_catalog(collections: Vec<ThunderboltCollection>, thunderbolts: Vec<Thunderbolt>) -> EnvironmentCatalog {
  EnvironmentCatalog {
    ambient_effects: Vec::new(),
    ambients: Vec::new(),
    configs: Vec::new(),
    cycles: Vec::new(),
    effects: Vec::new(),
    engine: XrayEngine::Vanilla,
    findings: Vec::new(),
    graphs: WeatherGraphs::default(),
    level_ambients: Vec::new(),
    sound_channels: Vec::new(),
    sun_table: None,
    suns: Vec::new(),
    thunderbolt_collections: collections,
    thunderbolt_settings: None,
    thunderbolts,
  }
}

fn read(catalog: &EnvironmentCatalog) -> LevelThunderbolts {
  let tree: FixtureTree = FixtureTree::new("level_thunderbolts");
  let mut vfs: XrayVfs = XrayVfs::new();
  let id: XrayMountId = vfs.mount_directory("", tree.root()).expect("tree mounts");
  let probe: XrayProbe = vfs.probe().with_step("tree", XrayLookupScope::only([id]));
  let source: LevelWeatherSource = LevelWeatherSource {
    catalog,
    probe: &probe,
    scope: XrayTextureScope::shared(),
  };

  LevelThunderbolts::read(&source)
}

#[test]
fn reads_each_bolt_once_however_many_collections_name_it() {
  let catalog: EnvironmentCatalog = new_catalog(
    vec![
      new_collection("storm", &["bolt_a", "bolt_b"]),
      new_collection("far", &["bolt_b", "bolt_gone", "bolt_a"]),
    ],
    vec![
      new_bolt("bolt_a", "fx\\fx_lightning", "thunder_a"),
      new_bolt("bolt_b", "fx\\fx_lightning", ""),
    ],
  );
  let thunderbolts: LevelThunderbolts = read(&catalog);

  // In the order first named; a bolt the game does not have is left out.
  assert_eq!(
    thunderbolts
      .bolts
      .iter()
      .map(|bolt| bolt.name.as_str())
      .collect::<Vec<_>>(),
    vec!["bolt_a", "bolt_b"]
  );
  assert_eq!(thunderbolts.collections.len(), 2);
}

// A model not in the game, a colour animation without `lanims.xr`, and a bolt naming neither all draw without.
#[test]
fn draws_a_bolt_without_what_the_game_does_not_hold() {
  let catalog: EnvironmentCatalog = new_catalog(
    vec![new_collection("storm", &["bolt_a", "bolt_b"])],
    vec![
      new_bolt("bolt_a", "fx\\fx_lightning", "thunder_a"),
      new_bolt("bolt_b", "", ""),
    ],
  );
  let thunderbolts: LevelThunderbolts = read(&catalog);

  assert!(
    thunderbolts
      .bolts
      .iter()
      .all(|bolt| bolt.model.is_none() && bolt.color.is_none())
  );
  assert!(thunderbolts.models.is_empty());
  assert!(thunderbolts.animators.is_empty());
  assert_eq!(thunderbolts.settings, None);
}
