// Auto-generated rust bindings. Do not edit it manually.

/** One problem with a game's environment configs, placed at the file, section and key it is about. */
export type EnvironmentFinding = {
  rule: EnvironmentRule;
  /** The config it is in, as a logical path. */
  file: string;
  /** The section it is in, where it is about one. */
  section: string | null;
  /** The key it is about, where it is about one. */
  key: string | null;
  message: string;
};

/** What kind of problem a finding reports. */
export enum EEnvironmentRule {
  /** The engine refuses to load it, or loads it as something other than what is written. */
  ENGINE = "engine",
  /** A section names another the engine cannot find. */
  REFERENCE = "reference",
  /**
   * The engine loads it, but against a convention strong enough to be a mistake: the engine's own `! Invalid`
   * warnings, a value it clamps, a list it pairs item by item written unevenly.
   */
  CONVENTION = "convention",
}

/** Every `EEnvironmentRule` as the spelling it crosses IPC as, for a value no member has narrowed. */
export type EnvironmentRule = `${EEnvironmentRule}`;

/**
 * The cycles a level can be lit under, as its game's weather script chooses between them: the `weathers` key of its
 * `game.ltx` section, a condlist, read against `dynamic_weather_graphs.ltx`.
 */
export type LevelWeather = {
  level: string;
  /** The `weathers` key as written, `[default]` where the level writes none. */
  key: string;
  /** Every cycle the key can play, in the order its branches and graphs list them; the first is where a viewer starts. */
  options: Array<LevelWeatherOption>;
};

/** One cycle a level can be lit under, and what in its weather picks it. */
export type LevelWeatherOption = {
  /** The cycle, under `environment\weathers`. */
  cycle: string;
  /** The graph that plays it, or `atmosfear`; none for a cycle the level names itself. */
  graph: string | null;
  /** The graph's or Atmosfear's state that plays it, `clear` or `rain`. */
  state: string | null;
};

/** Where the sun stands at one hour of Monolith's sun table, in degrees, named as the table names them. */
export type SunPosition = {
  /** `sun_altitude`, which `setHP` makes the heading. */
  altitude: number | null;
  /** `sun_longitude`, which `setHP` makes the pitch. */
  longitude: number | null;
};

/**
 * A `thunderbolt_collections.ltx` section, `SThunderboltCollection`: the bolts a keyframe strikes with, one of which
 * the engine picks at random each strike. Each line's key names a bolt; its value is not read.
 */
export type ThunderboltCollection = {
  name: string;
  /** The config, as a logical path. */
  file: string;
  /** The bolts' `thunderbolts.ltx` sections, in the order written. */
  thunderbolts: Array<string>;
};

/** Which cycle or effect: the list the engine keeps it in, and its name there. */
export type WeatherCycleId = {
  kind: WeatherCycleKind;
  /** The file's name without its extension. */
  name: string;
};

/** Which list the engine keeps a cycle in. */
export enum EWeatherCycleKind {
  /** A day, under `environment\weathers`, which `set_weather` plays: `load_weathers`. */
  CYCLE = "cycle",
  /**
   * An effect, under `environment\weather_effects`, which `start_weather_fx` plays over a cycle, bracketed by the
   * engine with keyframes of its own at midnight either end: `load_weather_effects`.
   */
  EFFECT = "effect",
}

/** Every `EWeatherCycleKind` as the spelling it crosses IPC as, for a value no member has narrowed. */
export type WeatherCycleKind = `${EWeatherCycleKind}`;

/**
 * One keyframe as the engine holds it once loaded, `CEnvDescriptor` after `load`: angles in radians, the clouds'
 * colour scaled by its multiplier, the sun's direction built, every key the section leaves out at its default.
 */
export type WeatherDescriptor = {
  /** Seconds since midnight, `exec_time`. */
  time: number;
  skyTexture: string;
  /** `sky_texture` with `#small`, the irradiance cube bound beside it. */
  skyTextureEnv: string;
  skyColor: [number | null, number | null, number | null];
  /** Radians. */
  skyRotation: number | null;
  cloudsTexture: string;
  /** The colour scaled by half the fifth component, the alpha as written. */
  cloudsColor: [number | null, number | null, number | null, number | null];
  /** Radians. */
  cloudsRotation: number | null;
  farPlane: number | null;
  fogColor: [number | null, number | null, number | null];
  fogDensity: number | null;
  fogDistance: number | null;
  /** Clamped to a unit. */
  rainDensity: number | null;
  rainColor: [number | null, number | null, number | null];
  windVelocity: number | null;
  /** Radians. */
  windDirection: number | null;
  hemiColor: [number | null, number | null, number | null, number | null];
  sunColor: [number | null, number | null, number | null];
  ambientColor: [number | null, number | null, number | null];
  /** The ambient's section, none where not named. */
  ambient: string | null;
  /** The lens flare's section, none where not named. */
  sun: string | null;
  /** The sun's direction as the keyframe stands it; none on an engine that stands it by its sun table. */
  sunDirection: [number | null, number | null, number | null] | null;
  /**
   * Whether OpenXRay reads the keyframe's sun from `sun_dir` rather than the two angles, which clears the
   * descriptor's `use_dynamic_sun_dir`. Only the weather editor's save reads that flag back; the mixer stands its sun by
   * `openxray.ltx`'s `dynamic_sun_dir` whatever the keyframes write.
   */
  isSunFixed: boolean;
  /** Radians, what OpenXRay turns its computed sun by. */
  sunAzimuth: number | null;
  sunShaftsIntensity: number | null;
  waterIntensity: number | null;
  treeAmplitude: number | null;
  treeSpeed: number | null;
  treeRotation: number | null;
  treeWave: [number | null, number | null, number | null];
  /** The collection's section, none where not named. */
  thunderboltCollection: string | null;
  /** Seconds, zero without a collection. */
  thunderboltPeriod: number | null;
  /** Seconds, zero without a collection. */
  thunderboltDuration: number | null;
  hemiVibrance: number | null;
  hemiContrast: number | null;
  wetSurfaceFactor: number | null;
  volumetricIntensityFactor: number | null;
  volumetricDistanceFactor: number | null;
  bloomThreshold: number | null;
  bloomExposure: number | null;
  bloomSkyIntensity: number | null;
};
