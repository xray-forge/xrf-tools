use xrf_ltx::Ltx;
use xrf_renderer::{RenderAntialiasing, RenderDisplayTransform, RenderViewFeatures};

use crate::game_preset::GamePreset;
use crate::game_setting_override::GameSettingOverride;
use crate::game_settings::GameSettings;
use crate::game_settings_load::GameSettingsFile;
use crate::game_window_mode::GameWindowMode;

fn over(path: &str, value: &str) -> GameSettingOverride {
  GameSettingOverride {
    path: path.to_owned(),
    value: value.to_owned(),
    origin: String::from("test"),
  }
}

fn high() -> GameSettings {
  GamePreset::High.get_settings()
}

#[test]
fn starts_the_high_preset_from_the_renderers_defaults() {
  let settings: GameSettings = high();

  assert_eq!(settings.view.antialiasing, RenderAntialiasing::Taa);
  assert_eq!(settings.view.shadows.resolution, 4096);
  // What the preset leaves alone stays the renderer's own.
  assert_eq!(settings.view.bloom, RenderViewFeatures::default().bloom);
  assert_eq!(settings.window.mode, GameWindowMode::Fullscreen);
}

#[test]
fn changes_a_number_a_flag_and_a_variant_over_the_preset() {
  let settings: GameSettings = high().merge(&[
    over("view.shadows.resolution", "2048"),
    over("view.grass.is_shadowed", "off"),
    over("view.display_transform", "pbr_neutral"),
    over("view.grass.radius", "120.5"),
    over("window.mode", "windowed"),
  ]);

  assert_eq!(settings.view.shadows.resolution, 2048);
  assert!(!settings.view.grass.is_shadowed);
  assert_eq!(settings.view.display_transform, RenderDisplayTransform::PbrNeutral);
  assert!((settings.view.grass.radius - 120.5).abs() < 1e-6);
  assert_eq!(settings.window.mode, GameWindowMode::Windowed);
}

#[test]
fn replaces_a_list_whole() {
  let settings: GameSettings = high().merge(&[over("view.shadows.cascades", "10, 40")]);

  assert_eq!(settings.view.shadows.cascades.len(), 2);
  assert!((settings.view.shadows.cascades[1] - 40.0).abs() < 1e-6);
}

#[test]
fn leaves_out_what_names_nothing_or_takes_no_such_value_and_keeps_the_rest() {
  let settings: GameSettings = high().merge(&[
    over("view.shadows.no_such_setting", "1"),
    over("view.no_such_group.steps", "1"),
    over("view.shadows", "1"),
    over("view.shadows.resolution", "lots"),
    over("view.antialiasing", "nonsense"),
    over("view.shadows.resolution", "1024"),
  ]);

  assert_eq!(settings.view.antialiasing, RenderAntialiasing::Taa);
  assert_eq!(settings.view.shadows.resolution, 1024);
}

#[test]
fn reads_a_settings_file_by_dotted_sections_with_its_preset() {
  let ltx: Ltx =
    Ltx::read_from_str("[settings]\npreset = high\n\n[view.shadows.contact]\nsteps = 16\n\n[window]\nwidth = 1280\n")
      .expect("the file reads");
  let file: GameSettingsFile = GameSettings::read_settings_ltx(&ltx, "settings.ltx");

  assert_eq!(file.preset, Some(GamePreset::High));

  let settings: GameSettings = high().merge(&file.overrides);

  assert_eq!(settings.view.shadows.contact.steps, 16);
  assert_eq!(settings.window.width, 1280);
}

#[test]
fn reads_an_argument_as_a_path_and_a_value() {
  assert_eq!(
    GameSettingOverride::parse_argument("view.shadows.resolution = 2048"),
    Some(GameSettingOverride {
      path: String::from("view.shadows.resolution"),
      value: String::from("2048"),
      origin: String::from("--set"),
    })
  );
  assert_eq!(GameSettingOverride::parse_argument("view.shadows.resolution"), None);
  assert_eq!(GameSettingOverride::parse_argument("=2048"), None);
}

#[test]
fn names_a_preset_in_any_case_and_refuses_one_the_game_lacks() {
  assert_eq!("high".parse::<GamePreset>(), Ok(GamePreset::High));
  assert_eq!(" HIGH ".parse::<GamePreset>(), Ok(GamePreset::High));
  assert!("editing".parse::<GamePreset>().is_err());
}

#[test]
fn spells_a_name_as_the_settings_hold_it() {
  assert_eq!(GameSettings::format_camel_case("shadow_radius"), "shadowRadius");
  assert_eq!(GameSettings::format_camel_case("ssa_a"), "ssaA");
  assert_eq!(GameSettings::format_camel_case("isTraced"), "isTraced");
}
