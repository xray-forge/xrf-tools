use xrf_engine_target::XrayEngine;

use super::fixtures::{EnvironmentFixture, messages, vanilla_keyframe};
use crate::{AmbientKey, EnvironmentRule};

fn clean_day() -> EnvironmentFixture {
  EnvironmentFixture::new().with(
    "environment\\weathers\\test.ltx",
    &(vanilla_keyframe("00:00:00") + &vanilla_keyframe("12:00:00")),
  )
}

#[test]
fn says_which_definition_a_keyframe_names_is_missing() {
  let keyframe: String = vanilla_keyframe("00:00:00")
    .replace("ambient = day", "ambient = absent_ambient")
    .replace("sun = sun", "sun = absent_sun")
    .replace(
      "thunderbolt_collection = bolts",
      "thunderbolt_collection = absent_bolts",
    );
  let fixture: EnvironmentFixture = EnvironmentFixture::new().with(
    "environment\\weathers\\test.ltx",
    &(keyframe + &vanilla_keyframe("12:00:00")),
  );

  assert_eq!(
    messages(&fixture.read(XrayEngine::Vanilla), EnvironmentRule::Reference),
    vec![
      "Weather [00:00:00] references missing ambient [absent_ambient]",
      "Weather [00:00:00] references missing sun [absent_sun]",
      "Weather [00:00:00] references missing thunderbolt collection [absent_bolts]",
    ]
  );
}

// OpenXRay looks in `system.ltx` for a sun or a collection its own configs lack; Monolith does not.
#[test]
fn finds_a_legacy_definition_in_system_ltx_on_openxray_alone() {
  let keyframe: String = vanilla_keyframe("00:00:00").replace("sun = sun", "sun = legacy_sun");
  let fixture: EnvironmentFixture = EnvironmentFixture::new()
    .with(
      "environment\\weathers\\test.ltx",
      &(keyframe + &vanilla_keyframe("12:00:00")),
    )
    .with(
      "system.ltx",
      "[legacy_sun]\nblend_down_time = 60\nblend_rise_time = 60\nflares = off\ngradient = off\nsun = off\n",
    );
  let catalog = fixture.read(XrayEngine::Vanilla);

  assert_eq!(catalog.findings, Vec::new());
  assert_eq!(
    catalog.find_sun("legacy_sun").map(|sun| sun.file.as_str()),
    Some("system.ltx")
  );
  assert_eq!(
    messages(&fixture.read(XrayEngine::Extended), EnvironmentRule::Reference),
    vec!["Weather [00:00:00] references missing sun [legacy_sun]"]
  );
}

#[test]
fn requires_what_a_lens_flares_switches_turn_on() {
  let fixture: EnvironmentFixture = clean_day().with(
    "environment\\suns.ltx",
    "[sun]\nblend_down_time = 60\nblend_rise_time = 60\nflares = on\nflare_shader = effects\\flare\n\
     flare_textures = a.tga, b.tga\nflare_radius = 0.1\nflare_opacity = 0.1, 0.2\nflare_position = 1, 2\n\
     gradient = off\nsun = off\n",
  );
  let catalog = fixture.read(XrayEngine::Vanilla);

  assert_eq!(
    messages(&catalog, EnvironmentRule::Convention),
    vec!["Sun [sun] has 1 items in [flare_radius] for 2 flare textures; the engine reads the rest as zero"]
  );
  assert_eq!(messages(&catalog, EnvironmentRule::Engine), Vec::<&str>::new());

  let unfinished: EnvironmentFixture = clean_day().with(
    "environment\\suns.ltx",
    "[sun]\nblend_down_time = 60\nblend_rise_time = 60\nflares = off\ngradient = on\nsun = off\n",
  );

  assert_eq!(
    messages(&unfinished.read(XrayEngine::Vanilla), EnvironmentRule::Engine),
    vec![
      "Sun [sun] is missing required field [gradient_opacity]",
      "Sun [sun] is missing required field [gradient_radius]",
      "Sun [sun] is missing required field [gradient_shader]",
      "Sun [sun] is missing required field [gradient_texture]",
    ]
  );
}

#[test]
fn says_which_bolt_a_collection_names_is_missing() {
  let fixture: EnvironmentFixture = clean_day().with(
    "environment\\thunderbolt_collections.ltx",
    "[bolts]\nbolt =\nabsent_bolt =\n",
  );

  assert_eq!(
    messages(&fixture.read(XrayEngine::Vanilla), EnvironmentRule::Reference),
    vec!["Thunderbolt collection [bolts] references missing thunderbolt [absent_bolt]"]
  );
}

#[test]
fn needs_the_strike_settings_from_either_config_on_openxray() {
  let fallback: EnvironmentFixture = clean_day().without("environment\\environment.ltx").with(
    "system.ltx",
    "[thunderbolt_common]\naltitude = 20, 30\ndelta_longitude = 30\nfog_color = 0.1\nmin_dist_factor = 0.9\n\
     second_propability = 0.5\nsky_color = 0.1\nsun_color = 0.9\ntilt = 17\n",
  );
  let catalog = fallback.read(XrayEngine::Vanilla);

  assert_eq!(catalog.findings, Vec::new());
  assert_eq!(
    catalog.thunderbolt_settings.map(|settings| settings.name),
    Some(String::from("thunderbolt_common"))
  );

  let missing: EnvironmentFixture = clean_day()
    .without("environment\\environment.ltx")
    .with("system.ltx", "");

  assert_eq!(
    messages(&missing.read(XrayEngine::Vanilla), EnvironmentRule::Engine),
    vec!["There is no [thunderbolt_common] section, where the engine reads where thunderbolts strike from"]
  );
}

#[test]
fn says_which_sound_channel_or_effect_an_ambient_names_is_missing() {
  let fixture: EnvironmentFixture = clean_day().with(
    "environment\\ambients.ltx",
    "[day]\nsound_channels = wind, absent_channel\neffects = gust, absent_effect\nmin_effect_period = 30\n\
     max_effect_period = 60\n",
  );

  assert_eq!(
    messages(&fixture.read(XrayEngine::Vanilla), EnvironmentRule::Reference),
    vec![
      "Ambient [day] references missing effect [absent_effect]",
      "Ambient [day] references missing sound channel [absent_channel]",
    ]
  );
}

#[test]
fn checks_a_sound_channel_as_the_engine_asserts_on_it() {
  let fixture: EnvironmentFixture = clean_day().with(
    "environment\\sound_channels.ltx",
    "[wind]\nmin_distance = 40\nmax_distance = 20\nperiod0 = 30000\nperiod1 = 25000\nperiod2 = 5000\n\
     period3 = 20000\nsounds =\n",
  );

  assert_eq!(
    messages(&fixture.read(XrayEngine::Vanilla), EnvironmentRule::Engine),
    vec![
      "Sound channel [wind] has a period whose least exceeds its most (30000..25000, 5000..20000), which the engine \
       asserts on",
      "Sound channel [wind] has [max_distance] 20 not beyond [min_distance] 40, which the engine asserts on",
      "Sound channel [wind] has no [sounds], which the engine asserts on",
    ]
  );
}

// A Shadow of Chernobyl ambient is its own channel, and OpenXRay still reads one.
#[test]
fn reads_a_shadow_of_chernobyl_ambient_as_its_own_channel() {
  let fixture: EnvironmentFixture = clean_day().with(
    "environment\\ambients.ltx",
    "[day]\nsounds = ambient\\wind_1\nsound_dist = 20, 40\nsound_period = 25, 30\neffect_period = 30, 60\n",
  );
  let catalog = fixture.read(XrayEngine::Vanilla);

  assert_eq!(catalog.findings, Vec::new());
  assert!(catalog.find_sound_channel("day").is_some());
}

#[test]
fn reads_a_levels_own_ambients() {
  let fixture: EnvironmentFixture = clean_day().with(
    "environment\\ambients\\zaton.ltx",
    "[day]\nsound_channels = absent_channel\nmin_effect_period = 30\nmax_effect_period = 60\n",
  );
  let catalog = fixture.read(XrayEngine::Vanilla);

  assert_eq!(
    catalog.find_level_ambients("zaton").map(|level| level.ambients.len()),
    Some(1)
  );
  assert_eq!(
    messages(&catalog, EnvironmentRule::Reference),
    vec!["Ambient [day] references missing sound channel [absent_channel]"]
  );
}

// The engine loads a sound channel only for an ambient that plays it, so one nothing names cannot fail it.
#[test]
fn demotes_a_problem_in_a_definition_the_engine_never_loads() {
  let fixture: EnvironmentFixture = clean_day().with(
    "environment\\sound_channels.ltx",
    "[wind]\nmin_distance = 20\nmax_distance = 40\nperiod0 = 1\nperiod1 = 2\nperiod2 = 3\nperiod3 = 4\n\
     sounds = ambient\\wind_1\n\n[crows]\nmin_distance = 60\nmax_distance = 80\nperiod0 = 15000\nperiod1 = 30000\n\
     period2 = 52500\nperiod3 = 32000\nsounds = ambient\\crow_1\n",
  );
  let catalog = fixture.read(XrayEngine::Vanilla);

  assert_eq!(messages(&catalog, EnvironmentRule::Engine), Vec::<&str>::new());
  assert_eq!(
    messages(&catalog, EnvironmentRule::Convention),
    vec![
      "Sound channel [crows] has a period whose least exceeds its most (15000..30000, 52500..32000), which the engine \
       asserts on; nothing references it, so the engine never loads it"
    ]
  );
}

// Anomaly keeps shared ambients under `ambients\` too, included by `ambients.ltx` rather than named for a level.
#[test]
fn reads_a_file_ambients_ltx_includes_as_shared_rather_than_a_levels() {
  let fixture: EnvironmentFixture = clean_day()
    .with(
      "environment\\ambients.ltx",
      "#include \"ambients\\underground.ltx\"\n\n[day]\nsound_channels = wind\nmin_effect_period = 30\n\
       max_effect_period = 60\n",
    )
    .with(
      "environment\\ambients\\underground.ltx",
      "[underground]\nsound_channels = wind\nmin_effect_period = 30\nmax_effect_period = 60\n",
    );
  let catalog = fixture.read(XrayEngine::Vanilla);

  assert!(catalog.find_ambient("underground").is_some());
  assert!(catalog.level_ambients.is_empty());
}

// `load_level_specific_ambients` reads a level's section over a shared one of its name, and adds none of its own.
#[test]
fn finds_a_levels_own_ambient_over_the_shared_one() {
  let fixture: EnvironmentFixture = clean_day().with(
    "environment\\ambients\\zaton.ltx",
    "[day]\nsound_channels = wind\nmin_effect_period = 5\nmax_effect_period = 6\n\n[night]\nsound_channels = wind\n\
     min_effect_period = 1\nmax_effect_period = 2\n",
  );
  let catalog = fixture.read(XrayEngine::Vanilla);
  let period = |level: &str| {
    catalog
      .find_level_ambient(level, "day")
      .map(|ambient| AmbientKey::get_effect_period(ambient, XrayEngine::Vanilla))
  };

  assert_eq!(period("zaton"), Some([5.0, 6.0]));
  assert_eq!(period("jupiter"), Some([30.0, 60.0]));
  assert!(catalog.find_level_ambient("zaton", "night").is_none());
}
