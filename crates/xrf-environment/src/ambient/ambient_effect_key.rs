use xrf_engine_target::XrayEngine;
use xrf_ltx::Section;

use crate::ambient::ambient_effect::AmbientEffect;
use crate::finding::EnvironmentRule;
use crate::key::declare_environment_keys;
use crate::section::EnvironmentSectionReader;

declare_environment_keys! {
  /// The keys of an `effects.ltx` section, `CEnvAmbient::create_effect`: a particle effect an ambient plays near the
  /// camera, with a sound and a gust of wind.
  pub enum AmbientEffectKey {
    /// Seconds.
    LifeTime = "life_time": Number, vanilla: Required(Number(0.0)), extended: Required(Number(0.0));
    /// A `particles.xr` effect.
    Particles = "particles": Text, vanilla: Required(Text("")), extended: Required(Text(""));
    /// Metres from the camera.
    Offset = "offset": Vector { least: 3, most: 3 },
      vanilla: Required(Vector(&[0.0, 0.0, 0.0])), extended: Required(Vector(&[0.0, 0.0, 0.0]));
    WindGustFactor = "wind_gust_factor": Number, vanilla: Required(Number(0.0)), extended: Required(Number(0.0));
    /// A sound, none where not written.
    Sound = "sound": Text, vanilla: Optional(Text("")), extended: Optional(Text(""));
    /// A blast of wind with the effect; the three keys after it are read only where it is written.
    WindBlastStrength = "wind_blast_strength": Number, vanilla: Optional(Number(0.0)), extended: Optional(Number(0.0));
    /// Degrees.
    WindBlastLongitude = "wind_blast_longitude": Number,
      vanilla: Conditional(Number(0.0)), extended: Conditional(Number(0.0));
    /// Seconds.
    WindBlastInTime = "wind_blast_in_time": Number,
      vanilla: Conditional(Number(0.0)), extended: Conditional(Number(0.0));
    /// Seconds.
    WindBlastOutTime = "wind_blast_out_time": Number,
      vanilla: Conditional(Number(0.0)), extended: Conditional(Number(0.0));
  }
}

impl AmbientEffectKey {
  /// Reads one effect and says what its engine would refuse.
  pub(crate) fn read(reader: &mut EnvironmentSectionReader, name: &str, section: &Section) -> AmbientEffect {
    let engine: XrayEngine = reader.get_engine();
    let effect: AmbientEffect = reader.read::<Self>(name, section);

    if effect.has(Self::WindBlastStrength) {
      for key in [Self::WindBlastLongitude, Self::WindBlastInTime, Self::WindBlastOutTime] {
        reader.require(&effect, key);
      }
    }

    // `VERIFY(result->particles.size())`: a debug build stops on it, a release one plays nothing.
    if effect.has(Self::Particles) && effect.get_text(Self::Particles, engine).is_empty() {
      let message: String = format!("{} names no [particles]", reader.describe(name));

      reader.report(EnvironmentRule::Convention, name, Some("particles"), message);
    }

    effect
  }
}
