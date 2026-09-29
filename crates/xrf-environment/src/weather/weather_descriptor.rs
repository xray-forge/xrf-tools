use serde::Serialize;
use xrf_engine_target::XrayEngine;

use crate::key::EnvironmentValue;
use crate::section::EnvironmentSection;
use crate::weather::weather_key::WeatherKey;
use crate::weather::weather_keyframe::WeatherKeyframe;

/// One keyframe as the engine holds it once loaded, `CEnvDescriptor` after `load`: angles in radians, the clouds'
/// colour scaled by its multiplier, the sun's direction built, every key the section leaves out at its default.
#[cfg_attr(feature = "typescript-bindings", derive(specta::Type))]
#[derive(Clone, Debug, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct WeatherDescriptor {
  /// Seconds since midnight, `exec_time`.
  pub time: u32,
  pub sky_texture: String,
  /// `sky_texture` with `#small`, the irradiance cube bound beside it.
  pub sky_texture_env: String,
  pub sky_color: [f32; 3],
  /// Radians.
  pub sky_rotation: f32,
  pub clouds_texture: String,
  /// The colour scaled by half the fifth component, the alpha as written.
  pub clouds_color: [f32; 4],
  /// Radians.
  pub clouds_rotation: f32,
  pub far_plane: f32,
  pub fog_color: [f32; 3],
  pub fog_density: f32,
  pub fog_distance: f32,
  /// Clamped to a unit.
  pub rain_density: f32,
  pub rain_color: [f32; 3],
  pub wind_velocity: f32,
  /// Radians.
  pub wind_direction: f32,
  pub hemi_color: [f32; 4],
  pub sun_color: [f32; 3],
  pub ambient_color: [f32; 3],
  /// The ambient's section, none where not named.
  pub ambient: Option<String>,
  /// The lens flare's section, none where not named.
  pub sun: Option<String>,
  /// The sun's direction as the keyframe stands it; none on an engine that stands it by its sun table.
  pub sun_direction: Option<[f32; 3]>,
  /// Whether the keyframe fixes its sun, `sun_dir`, against OpenXRay's own computed one.
  pub is_sun_fixed: bool,
  /// Radians, what OpenXRay turns its computed sun by.
  pub sun_azimuth: f32,
  pub sun_shafts_intensity: f32,
  pub water_intensity: f32,
  pub tree_amplitude: f32,
  pub tree_speed: f32,
  pub tree_rotation: f32,
  pub tree_wave: [f32; 3],
  /// The collection's section, none where not named.
  pub thunderbolt_collection: Option<String>,
  /// Seconds, zero without a collection.
  pub thunderbolt_period: f32,
  /// Seconds, zero without a collection.
  pub thunderbolt_duration: f32,
  pub hemi_vibrance: f32,
  pub hemi_contrast: f32,
  pub wet_surface_factor: f32,
  pub volumetric_intensity_factor: f32,
  pub volumetric_distance_factor: f32,
  pub bloom_threshold: f32,
  pub bloom_exposure: f32,
  pub bloom_sky_intensity: f32,
}

impl WeatherDescriptor {
  /// The keyframe as the engine loads it.
  pub fn new(keyframe: &WeatherKeyframe, engine: XrayEngine) -> Self {
    let section: &EnvironmentSection<WeatherKey> = &keyframe.section;
    let number = |key: WeatherKey| section.get_number(key, engine);
    let text = |key: WeatherKey| section.get_text(key, engine).to_owned();
    let named = |key: WeatherKey| {
      Some(section.get_text(key, engine))
        .filter(|it| !it.is_empty())
        .map(str::to_owned)
    };

    let sky_texture: String = text(WeatherKey::SkyTexture);
    let sky_rotation: f32 = number(WeatherKey::SkyRotation).to_radians();
    let [red, green, blue, alpha, multiplier] = Self::read_clouds_color(section, engine);
    let thunderbolt_collection: Option<String> = named(WeatherKey::ThunderboltCollection);
    let bolt = |key: WeatherKey| {
      if thunderbolt_collection.is_some() {
        number(key)
      } else {
        0.0
      }
    };
    let (tree_speed, tree_rotation, tree_wave) = Self::read_tree_sway(section, engine);

    Self {
      ambient: named(WeatherKey::Ambient),
      ambient_color: section.get_vector(WeatherKey::AmbientColor, engine),
      bloom_exposure: number(WeatherKey::BloomExposure),
      bloom_sky_intensity: number(WeatherKey::BloomSkyIntensity),
      bloom_threshold: number(WeatherKey::BloomThreshold),
      // `clouds_color.mul(.5f * multiplier)` with the alpha saved around it: four components write no colour at all.
      clouds_color: [
        red * 0.5 * multiplier,
        green * 0.5 * multiplier,
        blue * 0.5 * multiplier,
        alpha,
      ],
      clouds_rotation: if section.has(WeatherKey::CloudsRotation) && engine == XrayEngine::Vanilla {
        number(WeatherKey::CloudsRotation).to_radians()
      } else {
        sky_rotation
      },
      clouds_texture: text(WeatherKey::CloudsTexture),
      far_plane: number(WeatherKey::FarPlane),
      fog_color: section.get_vector(WeatherKey::FogColor, engine),
      fog_density: number(WeatherKey::FogDensity),
      fog_distance: number(WeatherKey::FogDistance),
      hemi_color: Self::read_hemi_color(section, engine),
      hemi_contrast: number(WeatherKey::HemiContrast),
      hemi_vibrance: number(WeatherKey::HemiVibrance),
      is_sun_fixed: engine == XrayEngine::Vanilla && section.has(WeatherKey::SunDir),
      rain_color: section.get_vector(WeatherKey::RainColor, engine),
      rain_density: number(WeatherKey::RainDensity).clamp(0.0, 1.0),
      sky_color: section.get_vector(WeatherKey::SkyColor, engine),
      sky_rotation,
      sky_texture_env: format!("{sky_texture}#small"),
      sky_texture,
      sun: named(WeatherKey::Sun),
      sun_azimuth: number(WeatherKey::SunAzimuth).clamp(0.0, 360.0).to_radians(),
      sun_color: section.get_vector(WeatherKey::SunColor, engine),
      sun_direction: Self::read_sun_direction(section, engine),
      sun_shafts_intensity: number(WeatherKey::SunShaftsIntensity),
      thunderbolt_duration: bolt(WeatherKey::ThunderboltDuration),
      thunderbolt_period: bolt(WeatherKey::ThunderboltPeriod),
      thunderbolt_collection,
      time: keyframe.time.map_or(0, |time| time.get_seconds()),
      tree_amplitude: Self::read_tree_amplitude(section, engine),
      tree_rotation,
      tree_speed,
      tree_wave,
      volumetric_distance_factor: number(WeatherKey::VolumetricDistanceFactor),
      volumetric_intensity_factor: number(WeatherKey::VolumetricIntensityFactor),
      water_intensity: number(WeatherKey::WaterIntensity),
      wet_surface_factor: number(WeatherKey::WetSurfaceFactor),
      wind_direction: number(WeatherKey::WindDirection).to_radians(),
      wind_velocity: number(WeatherKey::WindVelocity),
    }
  }

  /// `Fvector::setHP(h, p)`: the direction a heading and a pitch point along.
  pub fn direction_of(heading: f32, pitch: f32) -> [f32; 3] {
    [-pitch.cos() * heading.sin(), pitch.sin(), pitch.cos() * heading.cos()]
  }

  /// `sscanf` into the constructed `(1, 1, 1, 1)` and a zero multiplier, so a component it does not read keeps those
  /// rather than the zero `r_fvector` would give it.
  fn read_clouds_color(section: &EnvironmentSection<WeatherKey>, engine: XrayEngine) -> [f32; 5] {
    let defaults: EnvironmentSection<WeatherKey> = EnvironmentSection::new("", "");
    let constructed: [f32; 5] = defaults.get_vector(WeatherKey::CloudsColor, engine);
    let written: &[f32] = section
      .get(WeatherKey::CloudsColor)
      .and_then(EnvironmentValue::as_vector)
      .unwrap_or(&[]);

    std::array::from_fn(|index| written.get(index).copied().unwrap_or(constructed[index]))
  }

  /// OpenXRay reads `hemisphere_color` and falls back to `hemi_color`; Monolith reads only the first.
  fn read_hemi_color(section: &EnvironmentSection<WeatherKey>, engine: XrayEngine) -> [f32; 4] {
    if engine == XrayEngine::Vanilla && !section.has(WeatherKey::HemisphereColor) && section.has(WeatherKey::HemiColor)
    {
      section.get_vector(WeatherKey::HemiColor, engine)
    } else {
      section.get_vector(WeatherKey::HemisphereColor, engine)
    }
  }

  /// `sun_dir.setHP(deg2rad(altitude), deg2rad(longitude))`, from `sun_dir` as longitude then altitude or from the two
  /// keys; the engine's names are swapped, so the altitude is the heading. None on Monolith, which reads neither.
  fn read_sun_direction(section: &EnvironmentSection<WeatherKey>, engine: XrayEngine) -> Option<[f32; 3]> {
    if engine != XrayEngine::Vanilla {
      return None;
    }

    let (longitude, altitude): (f32, f32) = if section.has(WeatherKey::SunDir) {
      let [longitude, altitude] = section.get_vector::<2>(WeatherKey::SunDir, engine);

      (longitude, altitude)
    } else {
      (
        section.get_number(WeatherKey::SunLongitude, engine),
        section.get_number(WeatherKey::SunAltitude, engine),
      )
    };

    Some(Self::direction_of(altitude.to_radians(), longitude.to_radians()))
  }

  /// The trees' sway speed, rotation and wave. Monolith reads none of them and sways by OpenXRay's defaults, which an
  /// empty section answers.
  fn read_tree_sway(section: &EnvironmentSection<WeatherKey>, engine: XrayEngine) -> (f32, f32, [f32; 3]) {
    let defaults: EnvironmentSection<WeatherKey> = EnvironmentSection::new("", "");
    let (section, engine) = match engine {
      XrayEngine::Vanilla => (section, engine),
      XrayEngine::Extended => (&defaults, XrayEngine::Vanilla),
    };

    (
      section.get_number(WeatherKey::TreesSpeed, engine),
      section.get_number(WeatherKey::TreesRotation, engine),
      section.get_vector(WeatherKey::TreesWave, engine),
    )
  }

  /// OpenXRay reads Lost Alpha's `trees_amplitude` before Call of Chernobyl's `tree_amplitude_intensity`.
  fn read_tree_amplitude(section: &EnvironmentSection<WeatherKey>, engine: XrayEngine) -> f32 {
    if engine == XrayEngine::Vanilla && section.has(WeatherKey::TreesAmplitude) {
      section.get_number(WeatherKey::TreesAmplitude, engine)
    } else {
      section.get_number(WeatherKey::TreeAmplitudeIntensity, engine)
    }
  }
}
