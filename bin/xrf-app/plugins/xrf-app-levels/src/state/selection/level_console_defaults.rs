use std::path::PathBuf;

use serde::Serialize;
use xrf_error::XrfResult;
use xrf_utils::{decode_bytes_to_string, new_windows1251_encoder};
use xrf_vfs::{FsgameFile, XrayRoots, XraySearch};

/// What a game's console sets of how its levels are lit, exposed and corrected: its installation's `user.ltx` over its
/// shipped defaults (Anomaly's `default_controls.ltx`); each `None` it leaves to the engine.
#[cfg_attr(feature = "typescript-bindings", derive(specta::Type))]
#[derive(Clone, Debug, Default, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct LevelConsoleDefaults {
  /// Whether the game ships console defaults at all.
  pub is_shipped: bool,
  /// `r2_tonemap`.
  pub is_tonemapped: Option<bool>,
  /// `r2_tonemap_amount`, `r2_tonemap_middlegray`, `r2_tonemap_lowlum` and `r2_tonemap_adaptation`.
  pub tonemap_amount: Option<f32>,
  pub tonemap_middle_gray: Option<f32>,
  pub tonemap_low_luminance: Option<f32>,
  pub tonemap_adaptation: Option<f32>,
  /// `r2_sun_lumscale`, `r2_sun_lumscale_hemi` and `r2_sun_lumscale_amb`.
  pub sun_scale: Option<f32>,
  pub hemi_scale: Option<f32>,
  pub ambient_scale: Option<f32>,
  /// `r__exposure`, `r__gamma`, `r__saturation` and `r__color_grading`.
  pub image_exposure: Option<f32>,
  pub image_gamma: Option<f32>,
  pub image_saturation: Option<f32>,
  pub color_grading: Option<[f32; 3]>,
  /// `r2_ls_bloom_threshold`, `r2_ls_bloom_kernel_g` and `r2_ls_bloom_kernel_scale`.
  pub bloom_threshold: Option<f32>,
  pub bloom_radius: Option<f32>,
  pub bloom_strength: Option<f32>,
}

impl LevelConsoleDefaults {
  /// The shipped console defaults, one command a line: Anomaly's, which its first run writes into `user.ltx`.
  const SHIPPED: &'static str = "configs/default_controls.ltx";

  /// The console an installation runs with, in its user data folder.
  const USER_CONSOLE: &'static str = "user.ltx";

  /// The alias `fsgame.ltx` names the user data folder by.
  const APP_DATA_ROOT: &'static str = "$app_data_root$";

  /// The console the game runs with: its installation's `user.ltx` over its shipped defaults, each command `user.ltx`
  /// leaves out taken from the shipped ones.
  ///
  /// # Errors
  ///
  /// Returns an error when the shipped file is there and cannot be read.
  pub fn read(search: &XraySearch, roots: &XrayRoots) -> XrfResult<Self> {
    let shipped: Self = Self::read_shipped(search)?;

    let Some((path, text)) = Self::read_user_console(roots) else {
      return Ok(shipped);
    };

    log::info!("Reading the game's console from {}", path.display());

    Ok(Self::parse(&text).over(shipped))
  }

  /// The game's shipped console defaults; nothing shipped where it has no such file.
  ///
  /// # Errors
  ///
  /// Returns an error when the file is there and cannot be read.
  pub fn read_shipped(search: &XraySearch) -> XrfResult<Self> {
    let Some(asset) = search.find(Self::SHIPPED)? else {
      return Ok(Self::default());
    };
    let bytes: Vec<u8> = search.read_asset(&asset)?;

    Ok(Self {
      is_shipped: true,
      ..Self::parse(&decode_bytes_to_string(&bytes, new_windows1251_encoder())?)
    })
  }

  /// The commands of a console file this viewer draws by, `r2_sun_lumscale 2.0` and its kin; a command it cannot read
  /// is left to the engine. Whether the game ships such a file is its reader's to say.
  pub fn parse(text: &str) -> Self {
    let mut defaults: Self = Self::default();

    for line in text.lines() {
      let command: &str = line.split(';').next().unwrap_or_default().trim();
      let Some((name, value)) = command.split_once(char::is_whitespace) else {
        continue;
      };
      let value: &str = value.trim();
      let number = || value.parse::<f32>().ok();

      match name.to_ascii_lowercase().as_str() {
        "r2_tonemap" => defaults.is_tonemapped = Self::parse_switch(value),
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
        "r__color_grading" => defaults.color_grading = Self::parse_vector(value),
        "r2_ls_bloom_threshold" => defaults.bloom_threshold = number(),
        "r2_ls_bloom_kernel_g" => defaults.bloom_radius = number(),
        "r2_ls_bloom_kernel_scale" => defaults.bloom_strength = number(),
        _ => {}
      }
    }

    defaults
  }

  /// The `user.ltx` of the first installation the roots sit in, found where its `fsgame.ltx` puts the user data folder;
  /// none for a bare data folder or one never run.
  fn read_user_console(roots: &XrayRoots) -> Option<(PathBuf, String)> {
    // todo: Read an Enhanced Edition's `user.ltx`, which sits at its installation root while its fsgame file leaves
    // `$app_data_root$` for the engine to declare.
    roots
      .list_installations()
      .into_iter()
      .next()
      .and_then(|installation| FsgameFile::read(installation).ok())
      .and_then(|fsgame| fsgame.resolve(Self::APP_DATA_ROOT))
      .map(|folder| folder.join(Self::USER_CONSOLE))
      .and_then(|path| {
        let bytes: Vec<u8> = std::fs::read(&path).ok()?;
        let text: String = decode_bytes_to_string(&bytes, new_windows1251_encoder()).ok()?;

        Some((path, text))
      })
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

  /// These commands over `base`'s: each one these leave unset taken from it.
  pub fn over(self, base: Self) -> Self {
    Self {
      is_shipped: self.is_shipped || base.is_shipped,
      is_tonemapped: self.is_tonemapped.or(base.is_tonemapped),
      tonemap_amount: self.tonemap_amount.or(base.tonemap_amount),
      tonemap_middle_gray: self.tonemap_middle_gray.or(base.tonemap_middle_gray),
      tonemap_low_luminance: self.tonemap_low_luminance.or(base.tonemap_low_luminance),
      tonemap_adaptation: self.tonemap_adaptation.or(base.tonemap_adaptation),
      sun_scale: self.sun_scale.or(base.sun_scale),
      hemi_scale: self.hemi_scale.or(base.hemi_scale),
      ambient_scale: self.ambient_scale.or(base.ambient_scale),
      image_exposure: self.image_exposure.or(base.image_exposure),
      image_gamma: self.image_gamma.or(base.image_gamma),
      image_saturation: self.image_saturation.or(base.image_saturation),
      color_grading: self.color_grading.or(base.color_grading),
      bloom_threshold: self.bloom_threshold.or(base.bloom_threshold),
      bloom_radius: self.bloom_radius.or(base.bloom_radius),
      bloom_strength: self.bloom_strength.or(base.bloom_strength),
    }
  }
}
