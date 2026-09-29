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

/** Every `kind` the `EnvironmentOrigin` union is told apart by, so a switch or a comparison names one. */
export enum EEnvironmentOrigin {
  /** Written in the section itself. */
  DECLARED = "declared",
  /** Inherited from the section that writes it. */
  INHERITED = "inherited",
  /** Won a load-order contest under a patching dialect, DLTX's `mod_` files among them. */
  LOADED = "loaded",
}

/** Where a key's resolved value is written, for a writer that has to put an edit back where it came from. */
export type EnvironmentOrigin =
  /** Written in the section itself. */
  | { kind: "declared"; file: string | null }
  /** Inherited from the section that writes it. */
  | { kind: "inherited"; section: string; file: string | null }
  /** Won a load-order contest under a patching dialect, DLTX's `mod_` files among them. */
  | { kind: "loaded"; file: string; depth: number; operation: string };

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
 * One section of an environment config as authored: every key the engine reads that it writes, as the engine reads
 * it, and every other key as written.
 */
export type EnvironmentSection = {
  /** The section's name as its header writes it. */
  name: string;
  /** The config it was read from, as a logical path. */
  file: string;
  /** Every key of the table the section writes, whether or not this engine reads it. */
  values: { [key in string]: EnvironmentValue };
  /** Keys outside the table, as written, so a writer can keep them. */
  extras: { [key in string]: string };
  /** Where each written key's value came from, for a read that was asked to record it; empty otherwise. */
  origins: { [key in string]: EnvironmentOrigin };
};

/** Every `kind` the `EnvironmentValue` union is told apart by, so a switch or a comparison names one. */
export enum EEnvironmentValue {
  NUMBER = "number",
  INTEGER = "integer",
  VECTOR = "vector",
  FLAG = "flag",
  TEXT = "text",
  LIST = "list",
}

/** One key's value as the engine holds it after reading. */
export type EnvironmentValue =
  | { kind: "number"; value: number | null }
  | { kind: "integer"; value: number }
  | { kind: "vector"; value: Array<number | null> }
  | { kind: "flag"; value: boolean }
  | { kind: "text"; value: string }
  | { kind: "list"; value: Array<string> };

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

/**
 * Monolith's `environment\sun_positions.ltx`: where the sun stands at each whole hour, lerped by the minute
 * (`calculate_config_sun_dir`) in place of any keyframe's.
 */
export type SunTable = {
  /** The config, as a logical path. */
  file: string;
  /** Twenty-four hours, midnight first; an hour the config does not write is left empty, at the engine's zero. */
  hours: Array<EnvironmentSection>;
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

/** One weather cycle or effect: a config whose every section is a keyframe. */
export type WeatherCycle = {
  /** The file's name without its extension, which is what the engine and its scripts name it by. */
  name: string;
  /** The config, as a logical path. */
  file: string;
  kind: WeatherCycleKind;
  /** Sorted by time as the engine sorts them, a keyframe whose name it refuses last. */
  keyframes: Array<WeatherKeyframe>;
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
  /** Whether the keyframe fixes its sun, `sun_dir`, against OpenXRay's own computed one. */
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

/** One keyframe of a weather cycle or effect, as authored: a section named for the time of day it applies at. */
export type WeatherKeyframe = {
  /** When it applies; none for a section name the engine refuses. */
  time: WeatherTime | null;
  section: EnvironmentSection;
};

/** A time of day as a keyframe's section names it, in whole seconds since midnight: the engine's `exec_time`. */
export type WeatherTime = number;
