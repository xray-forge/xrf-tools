use serde::Serialize;
use xrf_texture::{AlbedoVerdict, TextureAlbedo};

use crate::commands::level::albedo::albedo_texture::AlbedoTexture;
use crate::commands::level::albedo::level_albedo_survey::LevelAlbedoSurvey;

/// What `level albedo` measured: the level's albedo at a glance, then every texture.
#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct LevelAlbedoReport<'a> {
  level: String,
  summary: LevelAlbedoSummary,
  textures: &'a [AlbedoTexture],
}

/// The level's textures taken together, each counted once whatever area it covers.
#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct LevelAlbedoSummary {
  /// The range the verdicts judge by, linear luminance.
  pub floor: f32,
  pub ceiling: f32,
  pub surfaces: usize,
  pub unlit_surfaces: usize,
  pub textures: usize,
  pub measured: usize,
  pub dark: usize,
  pub within: usize,
  pub bright: usize,
  /// The median of the measured textures' medians.
  pub median_albedo: f32,
  /// The mean over the measured textures of their texels' share past each bound.
  pub dark_share: f32,
  pub bright_share: f32,
  /// The median of the measured textures' coarse spreads, in stops.
  pub median_coarse_spread: f32,
}

impl<'a> LevelAlbedoReport<'a> {
  pub fn new(level: String, survey: &'a LevelAlbedoSurvey, summary: LevelAlbedoSummary) -> Self {
    Self {
      level,
      summary,
      textures: &survey.textures,
    }
  }
}

impl LevelAlbedoSummary {
  pub fn of(survey: &LevelAlbedoSurvey) -> Self {
    let measured: Vec<&TextureAlbedo> = survey.textures.iter().filter_map(AlbedoTexture::get_albedo).collect();
    let count = |verdict: AlbedoVerdict| measured.iter().filter(|it| it.verdict == verdict).count();
    let mean = |share: fn(&TextureAlbedo) -> f32| {
      measured.iter().map(|it| share(it)).sum::<f32>() / measured.len().max(1) as f32
    };

    Self {
      floor: AlbedoVerdict::FLOOR,
      ceiling: AlbedoVerdict::CEILING,
      surfaces: survey.surfaces,
      unlit_surfaces: survey.unlit_surfaces,
      textures: survey.textures.len(),
      measured: measured.len(),
      dark: count(AlbedoVerdict::Dark),
      within: count(AlbedoVerdict::Within),
      bright: count(AlbedoVerdict::Bright),
      median_albedo: Self::median(measured.iter().map(|it| it.median).collect()),
      dark_share: mean(|it| it.dark_share),
      bright_share: mean(|it| it.bright_share),
      median_coarse_spread: Self::median(measured.iter().map(|it| it.coarse_spread).collect()),
    }
  }

  fn median(mut values: Vec<f32>) -> f32 {
    if values.is_empty() {
      return 0.0;
    }

    values.sort_unstable_by(f32::total_cmp);

    values[values.len() / 2]
  }
}
