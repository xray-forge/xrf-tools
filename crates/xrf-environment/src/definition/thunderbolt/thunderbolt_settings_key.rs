use xrf_engine_target::XrayEngine;
use xrf_ltx::Section;

use crate::definition::thunderbolt::thunderbolt_settings::ThunderboltSettings;
use crate::section::EnvironmentSectionReader;
use crate::section::finding::EnvironmentRule;
use crate::section::key::{EnvironmentValue, declare_environment_keys};

declare_environment_keys! {
  /// The keys of `environment.ltx`'s `[environment]`, where every bolt of every collection is struck from
  /// (`CEffect_Thunderbolt` in OpenXRay, `CEnvironment` in Monolith); OpenXRay falls back to `system.ltx`'s
  /// `[thunderbolt_common]` without the file.
  pub enum ThunderboltSettingsKey {
    /// Degrees above the horizon; OpenXRay also reads a range of two, where Monolith reads the first as a number.
    Altitude = "altitude": Vector { least: 1, most: 2 },
      vanilla: Required(Vector(&[0.0, 0.0])), extended: Required(Vector(&[0.0]));
    /// Degrees either side of the sun's heading.
    DeltaLongitude = "delta_longitude": Number, vanilla: Required(Number(0.0)), extended: Required(Number(0.0));
    /// The nearest a bolt strikes, of the far plane; at most `MAX_DIST_FACTOR`.
    MinDistFactor = "min_dist_factor": Number, vanilla: Required(Number(0.0)), extended: Required(Number(0.0));
    /// Degrees.
    Tilt = "tilt": Number, vanilla: Required(Number(0.0)), extended: Required(Number(0.0));
    /// The chance of a second bolt, clamped to a unit; the engine's spelling.
    SecondPropability = "second_propability": Number,
      vanilla: Required(Number(0.0)), extended: Required(Number(0.0));
    SkyColor = "sky_color": Number, vanilla: Required(Number(0.0)), extended: Required(Number(0.0));
    SunColor = "sun_color": Number, vanilla: Required(Number(0.0)), extended: Required(Number(0.0));
    FogColor = "fog_color": Number, vanilla: Required(Number(0.0)), extended: Required(Number(0.0));
  }
}

impl ThunderboltSettingsKey {
  /// `MAX_DIST_FACTOR`: the farthest a bolt strikes, of the far plane, and so the most `min_dist_factor` is taken as.
  pub const MAX_DIST_FACTOR: f32 = 0.95;

  /// Reads the strike settings and says what the engine would refuse, misread or clamp.
  pub(crate) fn read(reader: &mut EnvironmentSectionReader, name: &str, section: &Section) -> ThunderboltSettings {
    let settings: ThunderboltSettings = reader.read::<Self>(name, section);

    // `r_float` takes the first number of a range, and ignores the rest.
    if reader.get_engine() == XrayEngine::Extended
      && let Some([least, _, ..]) = settings.get(Self::Altitude).and_then(EnvironmentValue::as_vector)
    {
      let message: String = format!(
        "{} has [altitude] written as a range, of which the engine reads only {least}",
        reader.describe(name)
      );

      reader.report(EnvironmentRule::Convention, name, Some("altitude"), message);
    }

    if let Some(factor) = settings.get(Self::MinDistFactor).and_then(EnvironmentValue::as_number)
      && factor > Self::MAX_DIST_FACTOR
    {
      let message: String = format!(
        "{} has [min_dist_factor] = {factor}, which the engine takes as {}",
        reader.describe(name),
        Self::MAX_DIST_FACTOR
      );

      reader.report(EnvironmentRule::Convention, name, Some("min_dist_factor"), message);
    }

    if let Some(chance) = settings
      .get(Self::SecondPropability)
      .and_then(EnvironmentValue::as_number)
      && !(0.0..=1.0).contains(&chance)
    {
      let message: String = format!(
        "{} has [second_propability] = {chance}, which the engine clamps to 0..1",
        reader.describe(name)
      );

      reader.report(EnvironmentRule::Convention, name, Some("second_propability"), message);
    }

    settings
  }

  /// Degrees above the horizon a bolt strikes from, the least and the most: OpenXRay's range, or one number as both
  /// where it fails to read two; Monolith's one number as both.
  pub fn get_altitude(settings: &ThunderboltSettings, engine: XrayEngine) -> [f32; 2] {
    let written: &[f32] = settings
      .get(Self::Altitude)
      .and_then(EnvironmentValue::as_vector)
      .unwrap_or(&[]);
    let least: f32 = written.first().copied().unwrap_or(0.0);

    match (engine, written) {
      (XrayEngine::Vanilla, [_, most, ..]) => [least, *most],
      _ => [least, least],
    }
  }
}
