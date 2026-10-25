// Auto-generated rust bindings. Do not edit it manually.

/** One check a verification runs, spelled on the wire and the command line as it displays. */
export enum EGamedataVerificationType {
  /** Files one source holds but the engine cannot reach, rather than any kind of asset authored inside them. */
  COLLISIONS = "collisions",
  /** Declared sources the run could not open, rather than anything authored inside the ones it did. */
  COVERAGE = "coverage",
  ANIMATIONS = "animations",
  /** Everything under `configs\environment` the engine reads, and the textures its keyframes name. */
  ENVIRONMENT = "environment",
  LEVELS = "levels",
  LTX = "ltx",
  /** The game material library, and every name the engine, configs and spawn objects resolve against it. */
  MATERIALS = "materials",
  MESHES = "meshes",
  PARTICLES = "particles",
  PARTICLES_USAGE = "particles-usage",
  SCRIPTS = "scripts",
  SHADERS = "shaders",
  SOUNDS = "sounds",
  SPAWNS = "spawns",
  TEXTURES = "textures",
  WEAPONS = "weapons",
}

/** Every `EGamedataVerificationType` as the spelling it crosses IPC as, for a value no member has narrowed. */
export type GamedataVerificationType = `${EGamedataVerificationType}`;
