use xrf_engine_target::XrayEngine;

use super::fixtures::{EnvironmentFixture, extended_keyframe, sun_table, vanilla_keyframe};
use crate::{EnvironmentReader, LevelWeather, LevelWeatherOption};

fn option(cycle: &str, graph: Option<&str>, state: Option<&str>) -> LevelWeatherOption {
  LevelWeatherOption {
    cycle: cycle.to_owned(),
    graph: graph.map(str::to_owned),
    state: state.map(str::to_owned),
  }
}

fn level_weather(fixture: &EnvironmentFixture, engine: XrayEngine, level: &str) -> LevelWeather {
  let project = fixture.open();
  let catalog = EnvironmentReader::read(&project, engine).unwrap();

  EnvironmentReader::read_level_weather(&project, &catalog, level).unwrap()
}

fn vanilla_tree() -> EnvironmentFixture {
  EnvironmentFixture::new()
    .with(
      "environment\\weathers\\default_clear.ltx",
      &(vanilla_keyframe("00:00:00") + &vanilla_keyframe("12:00:00")),
    )
    .with(
      "environment\\dynamic_weather_graphs.ltx",
      "[dynamic_default]\nclear = 0.4\nrain = 0.1\n",
    )
}

#[test]
fn plays_a_graphs_states_as_the_vanilla_script_does() {
  let fixture: EnvironmentFixture = vanilla_tree().with("game.ltx", "[zaton]\nweathers = dynamic_default\n");

  assert_eq!(
    level_weather(&fixture, XrayEngine::Vanilla, "zaton").options,
    vec![
      option("default_clear", Some("dynamic_default"), Some("clear")),
      option("default_rain", Some("dynamic_default"), Some("rain")),
    ]
  );
}

#[test]
fn plays_the_default_where_a_level_names_none() {
  let fixture: EnvironmentFixture = vanilla_tree().with("game.ltx", "[zaton]\n");
  let weather: LevelWeather = level_weather(&fixture, XrayEngine::Vanilla, "zaton");

  assert_eq!(weather.key, "[default]");
  assert_eq!(weather.options, vec![option("[default]", None, None)]);
}

#[test]
fn offers_every_cycle_a_condlist_can_pick() {
  let fixture: EnvironmentFixture = vanilla_tree().with(
    "game.ltx",
    "[zaton]\nweathers = {+surge_started} indoor, dynamic_default\n",
  );

  assert_eq!(
    level_weather(&fixture, XrayEngine::Vanilla, "zaton").options,
    vec![
      option("indoor", None, None),
      option("default_clear", Some("dynamic_default"), Some("clear")),
      option("default_rain", Some("dynamic_default"), Some("rain")),
    ]
  );
}

#[test]
fn plays_atmosfears_presets_on_monolith() {
  let fixture: EnvironmentFixture = EnvironmentFixture::new()
    .with(
      "environment\\weathers\\w_clear1.ltx",
      &(extended_keyframe("00:00:00") + &extended_keyframe("12:00:00")),
    )
    .with("environment\\sun_positions.ltx", &sun_table())
    .with(
      "environment\\dynamic_weather_graphs.ltx",
      "[weather_cycles]\nclear\nstorm\n\n[cycle_clear]\nw_clear1\nw_clear2\n\n[cycle_storm]\nw_storm1\n",
    )
    .with("game.ltx", "[zaton]\nweathers = atmosfear\n");

  assert_eq!(
    level_weather(&fixture, XrayEngine::Extended, "zaton").options,
    vec![
      option("w_clear1", Some("atmosfear"), Some("clear")),
      option("w_clear2", Some("atmosfear"), Some("clear")),
      option("w_storm1", Some("atmosfear"), Some("storm")),
    ]
  );

  // OpenXRay's script has no Atmosfear, and plays the name as a cycle of its own.
  assert_eq!(
    level_weather(&fixture, XrayEngine::Vanilla, "zaton").options,
    vec![option("atmosfear", None, None)]
  );
}

// Under DLTX a `mod_` file patches the cycle it is named for rather than being one.
#[test]
fn reads_a_dltx_patch_into_its_cycle_rather_than_as_one() {
  let fixture: EnvironmentFixture = EnvironmentFixture::new()
    .with(
      "environment\\weathers\\w_clear1.ltx",
      &(extended_keyframe("00:00:00") + &extended_keyframe("12:00:00")),
    )
    .with(
      "environment\\weathers\\mod_w_clear1_brighter.ltx",
      "![12:00:00]\nfar_plane = 900\n",
    )
    .with("environment\\sun_positions.ltx", &sun_table())
    .with_dltx();
  let catalog = fixture.read(XrayEngine::Extended);

  assert_eq!(
    catalog
      .cycles
      .iter()
      .map(|cycle| cycle.name.as_str())
      .collect::<Vec<_>>(),
    vec!["w_clear1"]
  );
  assert_eq!(
    catalog.cycles[0].keyframes[1]
      .section
      .get_number(crate::WeatherKey::FarPlane, XrayEngine::Extended),
    900.0
  );
}
