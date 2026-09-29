use crate::key::declare_environment_keys;

declare_environment_keys! {
  /// The keys of a weather keyframe, `CEnvDescriptor::load` in `xray-16` and `xray-monolith`
  /// (`xrEngine/Environment_misc.cpp`); a required key's default is `CEnvDescriptor`'s constructed value.
  pub enum WeatherKey {
    /// A cube; the engine binds its `#small` twin beside it as the irradiance.
    SkyTexture = "sky_texture": Text, vanilla: Required(Text("")), extended: Required(Text(""));
    SkyColor = "sky_color": Vector { least: 3, most: 3 },
      vanilla: Required(Vector(&[1.0, 1.0, 1.0])), extended: Required(Vector(&[1.0, 1.0, 1.0]));
    /// Degrees.
    SkyRotation = "sky_rotation": Number, vanilla: Optional(Number(0.0)), extended: Optional(Number(0.0));
    CloudsTexture = "clouds_texture": Text, vanilla: Required(Text("")), extended: Required(Text(""));
    /// Four components and an optional fifth that scales the colour by half of it; the colour is lost without it.
    CloudsColor = "clouds_color": Vector { least: 4, most: 5 },
      vanilla: Required(Vector(&[1.0, 1.0, 1.0, 1.0, 0.0])), extended: Required(Vector(&[1.0, 1.0, 1.0, 1.0, 0.0]));
    /// Degrees; the sky's rotation where not written.
    CloudsRotation = "clouds_rotation": Number, vanilla: Optional(Number(0.0)), extended: Unread;
    /// Metres.
    FarPlane = "far_plane": Number, vanilla: Required(Number(400.0)), extended: Required(Number(400.0));
    FogColor = "fog_color": Vector { least: 3, most: 3 },
      vanilla: Required(Vector(&[1.0, 1.0, 1.0])), extended: Required(Vector(&[1.0, 1.0, 1.0]));
    FogDensity = "fog_density": Number, vanilla: Required(Number(0.0)), extended: Required(Number(0.0));
    /// Metres.
    FogDistance = "fog_distance": Number, vanilla: Required(Number(400.0)), extended: Required(Number(400.0));
    /// Clamped to a unit.
    RainDensity = "rain_density": Number, vanilla: Required(Number(0.0)), extended: Required(Number(0.0));
    RainColor = "rain_color": Vector { least: 3, most: 3 },
      vanilla: Required(Vector(&[0.0, 0.0, 0.0])), extended: Required(Vector(&[0.0, 0.0, 0.0]));
    WindVelocity = "wind_velocity": Number, vanilla: Required(Number(0.0)), extended: Required(Number(0.0));
    /// Degrees.
    WindDirection = "wind_direction": Number, vanilla: Required(Number(0.0)), extended: Required(Number(0.0));
    /// OpenXRay reads this or `hemi_color`, one of them required.
    HemisphereColor = "hemisphere_color": Vector { least: 4, most: 4 },
      vanilla: Conditional(Vector(&[1.0, 1.0, 1.0, 1.0])), extended: Required(Vector(&[1.0, 1.0, 1.0, 1.0]));
    HemiColor = "hemi_color": Vector { least: 4, most: 4 },
      vanilla: Conditional(Vector(&[1.0, 1.0, 1.0, 1.0])), extended: Unread;
    SunColor = "sun_color": Vector { least: 3, most: 3 },
      vanilla: Required(Vector(&[1.0, 1.0, 1.0])), extended: Required(Vector(&[1.0, 1.0, 1.0]));
    AmbientColor = "ambient_color": Vector { least: 3, most: 3 },
      vanilla: Required(Vector(&[0.0, 0.0, 0.0])), extended: Required(Vector(&[0.0, 0.0, 0.0]));
    /// An `ambients.ltx` section: the sounds and particles, none where not written.
    Ambient = "ambient": Text, vanilla: Optional(Text("")), extended: Optional(Text(""));
    /// A `suns.ltx` section: the sun's sprite and lens flare, not its direction.
    Sun = "sun": Text, vanilla: Required(Text("")), extended: Required(Text(""));
    /// Longitude then altitude, in degrees; in place of the two keys, and it fixes the sun against OpenXRay's own.
    SunDir = "sun_dir": Vector { least: 2, most: 2 }, vanilla: Conditional(Vector(&[0.0, 0.0])), extended: Unread;
    /// Degrees. Monolith stands the sun by its sun table instead.
    SunAltitude = "sun_altitude": Number, vanilla: Conditional(Number(0.0)), extended: Unread;
    /// Degrees.
    SunLongitude = "sun_longitude": Number, vanilla: Conditional(Number(0.0)), extended: Unread;
    /// Degrees, clamped to a turn; `openxray.ltx`'s `sun_dir_azimuth` where not written.
    SunAzimuth = "sun_azimuth": Number, vanilla: Optional(Number(0.0)), extended: Unread;
    SunShaftsIntensity = "sun_shafts_intensity": Number,
      vanilla: Optional(Number(0.0)), extended: Optional(Number(0.0));
    WaterIntensity = "water_intensity": Number, vanilla: Optional(Number(1.0)), extended: Optional(Number(1.0));
    /// Lost Alpha's spelling, which OpenXRay reads before Call of Chernobyl's.
    TreesAmplitude = "trees_amplitude": Number, vanilla: Optional(Number(0.005)), extended: Unread;
    /// Call of Chernobyl's spelling.
    TreeAmplitudeIntensity = "tree_amplitude_intensity": Number,
      vanilla: Optional(Number(0.005)), extended: Optional(Number(0.01));
    TreesSpeed = "trees_speed": Number, vanilla: Optional(Number(1.0)), extended: Unread;
    TreesRotation = "trees_rotation": Number, vanilla: Optional(Number(10.0)), extended: Unread;
    TreesWave = "trees_wave": Vector { least: 3, most: 3 }, vanilla: Optional(Vector(&[0.1, 0.01, 0.11])), extended: Unread;
    /// A `thunderbolt_collections.ltx` section, empty for none.
    ThunderboltCollection = "thunderbolt_collection": Text, vanilla: Required(Text("")), extended: Required(Text(""));
    /// Seconds; read only with a collection to strike from.
    ThunderboltPeriod = "thunderbolt_period": Number,
      vanilla: Conditional(Number(0.0)), extended: Conditional(Number(0.0));
    /// Seconds; read only with a collection to strike from.
    ThunderboltDuration = "thunderbolt_duration": Number,
      vanilla: Conditional(Number(0.0)), extended: Conditional(Number(0.0));
    HemiVibrance = "hemi_vibrance": Number, vanilla: Unread, extended: Optional(Number(1.0));
    HemiContrast = "hemi_contrast": Number, vanilla: Unread, extended: Optional(Number(1.0));
    /// Extra gloss on wet surfaces; none by default.
    WetSurfaceFactor = "wet_surface_factor": Number, vanilla: Unread, extended: Optional(Number(0.0));
    VolumetricIntensityFactor = "volumetric_intensity_factor": Number,
      vanilla: Unread, extended: Optional(Number(1.0));
    VolumetricDistanceFactor = "volumetric_distance_factor": Number,
      vanilla: Unread, extended: Optional(Number(1.0));
    BloomThreshold = "bloom_threshold": Number, vanilla: Unread, extended: Optional(Number(3.5));
    BloomExposure = "bloom_exposure": Number, vanilla: Unread, extended: Optional(Number(3.0));
    BloomSkyIntensity = "bloom_sky_intensity": Number, vanilla: Unread, extended: Optional(Number(0.6));
  }
}
