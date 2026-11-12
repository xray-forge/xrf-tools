// Auto-generated rust bindings. Do not edit it manually.

/** What a level's compile ran ([`crate::LevelCompiler::compile`]), as the front-ends report it. */
export type LevelCompileReport = {
  level: string;
  /** The directory the compilers ran in, with their `fsgame.ltx` and logs. */
  work: string;
  steps: Array<LevelCompileStepReport>;
};

/**
 * A level's compile, as a front-end asks for it: the level's directory the build wrote into, the steps to run, and
 * what the SDK's compilers need to run them.
 */
export type LevelCompileRequest = {
  /** The level's directory, `<levels>/<level>`, holding what `level build` wrote; the compile writes beside it. */
  level: string;
  /** The steps to run, each once in [`LevelCompileStep::ALL`]'s order whatever the order given; every step when empty. */
  steps: Array<LevelCompileStep>;
  /** The SDK's `editors` directory, whose `bin` holds xrLC, xrDO_Light and xrAI. */
  sdk: string;
  /** The game data the compilers read their libraries from (`$game_data$`): `gamemtl.xr` and `shaders_xrlc.xr`. */
  gamedata: string;
  /**
   * The textures root (`$game_textures$`): a `.thm` for every texture `build.prj` names, and the image where the
   * texture has alpha or is lit implicitly.
   */
  textures: string;
  /**
   * The directory the compilers run in, holding the `fsgame.ltx` written for them and their logs; a directory of the
   * system's temporary one named after the level by default.
   */
  work: string | null;
};

/**
 * One step of a level's compile from the files `level build` writes, in the order they run: the geometry (xrLC's
 * part, lightmaps included), the details' light (xrDO_Light's) and the AI map (xrAI's `-f`). Each runs the SDK's own
 * compiler until a compiler of ours takes its place.
 */
export enum ELevelCompileStep {
  GEOMETRY = "geometry",
  DETAILS = "details",
  AI = "ai",
}

/** Every `ELevelCompileStep` as the spelling it crosses IPC as, for a value no member has narrowed. */
export type LevelCompileStep = `${ELevelCompileStep}`;

/** One step a compile ran. */
export type LevelCompileStepReport = {
  step: LevelCompileStep;
  /** The program that ran it. */
  program: string;
  /** How long it ran, in milliseconds. */
  duration: number;
  /** The compiler's log, where it wrote one. */
  log: string | null;
};
