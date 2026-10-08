//! What a game's console says about how its levels are lit and exposed: the installation's own `user.ltx`, the
//! commands it runs with, over Anomaly's shipped `default_controls.ltx`, the commands its first run writes there.

use std::path::PathBuf;

use xrf_error::XrfResult;
use xrf_utils::{decode_bytes_to_string, new_windows1251_encoder};
use xrf_vfs::{FsgameFile, XrayProbe, XrayRoots};

use crate::plugins::levels::state::LevelConsoleDefaults;

/// The shipped console defaults, one command a line.
const DEFAULTS: &str = "configs/default_controls.ltx";

/// The console an installation runs with, in its user data folder.
const USER_CONSOLE: &str = "user.ltx";

/// The alias `fsgame.ltx` names the user data folder by.
const APP_DATA_ROOT: &str = "$app_data_root$";

/// The console the game runs with: its installation's `user.ltx` over its shipped defaults, each command `user.ltx`
/// leaves out taken from the shipped ones.
///
/// # Errors
///
/// Returns an error when the shipped file is there and cannot be read.
pub fn read_level_console(probe: &XrayProbe, roots: &XrayRoots) -> XrfResult<LevelConsoleDefaults> {
  let shipped: LevelConsoleDefaults = read_console_defaults(probe)?;

  let Some((path, text)) = read_user_console(roots) else {
    return Ok(shipped);
  };

  log::info!("Reading the game's console from {}", path.display());

  Ok(parse_console_defaults(&text).over(shipped))
}

/// The `user.ltx` of the first installation the roots sit in, found where its `fsgame.ltx` puts the user data folder;
/// none for a bare data folder or one never run.
fn read_user_console(roots: &XrayRoots) -> Option<(PathBuf, String)> {
  roots
    .list_installations()
    .into_iter()
    .next()
    .and_then(|installation| FsgameFile::read(installation).ok())
    .and_then(|fsgame| fsgame.resolve(APP_DATA_ROOT))
    .map(|folder| folder.join(USER_CONSOLE))
    .and_then(|path| {
      let bytes: Vec<u8> = std::fs::read(&path).ok()?;
      let text: String = decode_bytes_to_string(&bytes, new_windows1251_encoder()).ok()?;

      Some((path, text))
    })
}

/// The game's console defaults; nothing shipped where it has no such file.
///
/// # Errors
///
/// Returns an error when the file is there and cannot be read.
pub fn read_console_defaults(probe: &XrayProbe) -> XrfResult<LevelConsoleDefaults> {
  let resolution = probe.find(DEFAULTS)?;
  let Some(asset) = resolution.get_asset() else {
    return Ok(LevelConsoleDefaults::default());
  };
  let bytes: Vec<u8> = probe.read_asset_bytes(asset)?;

  Ok(parse_console_defaults(&decode_bytes_to_string(
    &bytes,
    new_windows1251_encoder(),
  )?))
}

/// The commands of a console defaults file this viewer draws by, `r2_sun_lumscale 2.0` and its kin; a command it
/// cannot read is left to the engine.
pub fn parse_console_defaults(text: &str) -> LevelConsoleDefaults {
  let mut defaults: LevelConsoleDefaults = LevelConsoleDefaults {
    is_shipped: true,
    ..LevelConsoleDefaults::default()
  };

  for line in text.lines() {
    let command: &str = line.split(';').next().unwrap_or_default().trim();
    let Some((name, value)) = command.split_once(char::is_whitespace) else {
      continue;
    };
    let value: &str = value.trim();
    let number = || value.parse::<f32>().ok();

    match name.to_ascii_lowercase().as_str() {
      "r2_tonemap" => defaults.is_tonemapped = parse_switch(value),
      "r2_tonemap_amount" => defaults.tonemap_amount = number(),
      "r2_tonemap_middlegray" => defaults.tonemap_middle_gray = number(),
      "r2_tonemap_lowlum" => defaults.tonemap_low_luminance = number(),
      "r2_tonemap_adaptation" => defaults.tonemap_adaptation = number(),
      "r2_sun_lumscale" => defaults.sun_scale = number(),
      "r2_sun_lumscale_hemi" => defaults.hemi_scale = number(),
      "r2_sun_lumscale_amb" => defaults.ambient_scale = number(),
      "r__exposure" => defaults.image_exposure = number(),
      "r__gamma" => defaults.image_gamma = number(),
      "r__saturation" => defaults.image_saturation = number(),
      "r__color_grading" => defaults.color_grading = parse_vector(value),
      "r2_ls_bloom_threshold" => defaults.bloom_threshold = number(),
      "r2_ls_bloom_kernel_g" => defaults.bloom_radius = number(),
      "r2_ls_bloom_kernel_scale" => defaults.bloom_strength = number(),
      _ => {}
    }
  }

  defaults
}

/// A console switch: `on` or `off`, `1` or `0`.
fn parse_switch(value: &str) -> Option<bool> {
  match value.to_ascii_lowercase().as_str() {
    "on" | "1" | "true" => Some(true),
    "off" | "0" | "false" => Some(false),
    _ => None,
  }
}

/// A console vector: `(r, g, b)`.
fn parse_vector(value: &str) -> Option<[f32; 3]> {
  let parts: Vec<f32> = value
    .trim_matches(|it| it == '(' || it == ')')
    .split(',')
    .map(|it| it.trim().parse::<f32>())
    .collect::<Result<_, _>>()
    .ok()?;

  parts.try_into().ok()
}
