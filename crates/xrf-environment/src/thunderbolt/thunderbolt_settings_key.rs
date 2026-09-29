use xrf_ltx::Section;

use crate::finding::EnvironmentRule;
use crate::key::{EnvironmentValue, declare_environment_keys};
use crate::section::EnvironmentSectionReader;
use crate::thunderbolt::thunderbolt_settings::ThunderboltSettings;

declare_environment_keys! {
  /// The keys of `environment.ltx`'s `[environment]`, where every bolt of every collection is struck from
  /// (`CEffect_Thunderbolt` in OpenXRay, `CEnvironment` in Monolith); OpenXRay falls back to `system.ltx`'s
  /// `[thunderbolt_common]` without the file.
  pub enum ThunderboltSettingsKey {
    /// Degrees above the horizon; OpenXRay also reads a range of two.
    Altitude = "altitude": Vector { least: 1, most: 2 },
      vanilla: Required(Vector(&[0.0, 0.0])), extended: Required(Vector(&[0.0]));
    /// Degrees either side of the sun's heading.
    DeltaLongitude = "delta_longitude": Number, vanilla: Required(Number(0.0)), extended: Required(Number(0.0));
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
  /// Reads the strike settings and says what the engine would refuse or clamp.
  pub(crate) fn read(reader: &mut EnvironmentSectionReader, name: &str, section: &Section) -> ThunderboltSettings {
    let settings: ThunderboltSettings = reader.read::<Self>(name, section);

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
}
