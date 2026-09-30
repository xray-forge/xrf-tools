use serde::Serialize;
use xrf_engine_target::XrayEngine;
use xrf_ltx::Section;

use crate::finding::EnvironmentRule;
use crate::key::{EnvironmentKey, EnvironmentValue};
use crate::section::{EnvironmentSection, EnvironmentSectionReader};
use crate::weather::weather_key::WeatherKey;
use crate::weather::weather_time::WeatherTime;

/// One keyframe of a weather cycle or effect, as authored: a section named for the time of day it applies at.
#[cfg_attr(feature = "typescript-bindings", derive(specta::Type))]
#[derive(Clone, Debug, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct WeatherKeyframe {
  /// When it applies; none for a section name the engine refuses.
  pub time: Option<WeatherTime>,
  pub section: EnvironmentSection<WeatherKey>,
}

impl WeatherKeyframe {
  /// The colours `CEnvDescriptor::load` checks with `C_CHECK`, which logs `! Invalid` for a component out of range.
  const CHECKED_COLORS: [WeatherKey; 8] = [
    WeatherKey::CloudsColor,
    WeatherKey::SkyColor,
    WeatherKey::FogColor,
    WeatherKey::RainColor,
    WeatherKey::AmbientColor,
    WeatherKey::HemisphereColor,
    WeatherKey::HemiColor,
    WeatherKey::SunColor,
  ];

  /// Values a negative one of makes no sense for: a distance, a speed, a period.
  const NON_NEGATIVE: [WeatherKey; 5] = [
    WeatherKey::FarPlane,
    WeatherKey::FogDistance,
    WeatherKey::WindVelocity,
    WeatherKey::ThunderboltPeriod,
    WeatherKey::ThunderboltDuration,
  ];

  /// Reads one keyframe and says everything its engine would refuse, misread or warn about.
  pub(crate) fn read(reader: &mut EnvironmentSectionReader, name: &str, section: &Section) -> Self {
    let engine: XrayEngine = reader.get_engine();
    let keyframe: Self = Self {
      section: reader.read::<WeatherKey>(name, section),
      time: WeatherTime::parse(name, engine),
    };

    keyframe.judge_time(reader);
    keyframe.judge_conditional_keys(reader);
    keyframe.judge_values(reader);

    keyframe
  }

  fn judge_time(&self, reader: &mut EnvironmentSectionReader) {
    let name: &str = &self.section.name;

    if self.time.is_none() {
      let message: String = format!(
        "{} is not a time of day the engine reads, which is HH:MM:SS",
        reader.describe(name)
      );

      reader.report(EnvironmentRule::Engine, name, None, message);
    } else if !Self::is_canonical_time(name) {
      let message: String = format!(
        "{} is read as {} but is not written HH:MM:SS",
        reader.describe(name),
        self.time.map_or(0, WeatherTime::get_seconds)
      );

      reader.report(EnvironmentRule::Convention, name, None, message);
    }
  }

  fn judge_conditional_keys(&self, reader: &mut EnvironmentSectionReader) {
    let section: &EnvironmentSection<WeatherKey> = &self.section;
    let engine: XrayEngine = reader.get_engine();

    if engine == XrayEngine::Vanilla {
      // `read_if_exists(hemi_color, "hemisphere_color", "hemi_color", true)`: one of the two, or the load fails.
      if !section.has(WeatherKey::HemiColor) {
        reader.require(section, WeatherKey::HemisphereColor);
      }

      // `sun_dir` stands in for the two angles.
      if !section.has(WeatherKey::SunDir) {
        reader.require(section, WeatherKey::SunAltitude);
        reader.require(section, WeatherKey::SunLongitude);
      }
    }

    if !section.get_text(WeatherKey::ThunderboltCollection, engine).is_empty() {
      reader.require(section, WeatherKey::ThunderboltPeriod);
      reader.require(section, WeatherKey::ThunderboltDuration);
    }
  }

  fn judge_values(&self, reader: &mut EnvironmentSectionReader) {
    self.judge_colors(reader);
    self.judge_signs(reader);
    self.judge_clamped(reader);

    let name: &str = &self.section.name;

    if self.section.has(WeatherKey::SkyTexture)
      && self
        .section
        .get_text(WeatherKey::SkyTexture, reader.get_engine())
        .is_empty()
    {
      let message: String = format!("{} names no [sky_texture]", reader.describe(name));

      reader.report(EnvironmentRule::Convention, name, Some("sky_texture"), message);
    }
  }

  /// `C_CHECK`: a colour component past what the engine takes for valid, which it warns about.
  fn judge_colors(&self, reader: &mut EnvironmentSectionReader) {
    let section: &EnvironmentSection<WeatherKey> = &self.section;
    let engine: XrayEngine = reader.get_engine();
    let name: &str = &section.name;
    // OpenXRay warns past two, Monolith past five.
    let color_limit: f32 = match engine {
      XrayEngine::Vanilla => 2.0,
      XrayEngine::Extended => 5.0,
    };

    for key in Self::CHECKED_COLORS {
      let Some(components) = section.get(key).and_then(EnvironmentValue::as_vector) else {
        continue;
      };

      if key.get_use(engine).is_read()
        && components
          .iter()
          .take(3)
          .any(|component| !(0.0..=color_limit).contains(component))
      {
        let message: String = format!(
          "{} has [{}] outside 0..{color_limit}, which the engine warns about as invalid",
          reader.describe(name),
          key.get_name()
        );

        reader.report(EnvironmentRule::Convention, name, Some(key.get_name()), message);
      }
    }
  }

  /// A value that means nothing below zero.
  fn judge_signs(&self, reader: &mut EnvironmentSectionReader) {
    let name: &str = &self.section.name;

    for key in Self::NON_NEGATIVE {
      if let Some(value) = self.section.get(key).and_then(EnvironmentValue::as_number)
        && value < 0.0
      {
        let message: String = format!("{} has a negative [{}]", reader.describe(name), key.get_name());

        reader.report(EnvironmentRule::Convention, name, Some(key.get_name()), message);
      }
    }
  }

  /// A value the engine clamps into its range as it loads the keyframe.
  fn judge_clamped(&self, reader: &mut EnvironmentSectionReader) {
    let section: &EnvironmentSection<WeatherKey> = &self.section;
    let name: &str = &section.name;

    if let Some(density) = section
      .get(WeatherKey::RainDensity)
      .and_then(EnvironmentValue::as_number)
      && !(0.0..=1.0).contains(&density)
    {
      let message: String = format!(
        "{} has [rain_density] = {density}, which the engine clamps to 0..1",
        reader.describe(name)
      );

      reader.report(EnvironmentRule::Convention, name, Some("rain_density"), message);
    }

    if reader.get_engine() == XrayEngine::Vanilla
      && let Some(azimuth) = section
        .get(WeatherKey::SunAzimuth)
        .and_then(EnvironmentValue::as_number)
      && !(0.0..=360.0).contains(&azimuth)
    {
      let message: String = format!(
        "{} has [sun_azimuth] = {azimuth}, which the engine clamps to 0..360",
        reader.describe(name)
      );

      reader.report(EnvironmentRule::Convention, name, Some("sun_azimuth"), message);
    }
  }

  /// Whether a section name is spelled `HH:MM:SS`, two digits a field.
  fn is_canonical_time(name: &str) -> bool {
    let bytes: &[u8] = name.as_bytes();

    bytes.len() == 8
      && bytes.iter().enumerate().all(|(index, byte)| match index {
        2 | 5 => *byte == b':',
        _ => byte.is_ascii_digit(),
      })
  }
}
