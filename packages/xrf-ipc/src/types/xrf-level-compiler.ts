// Auto-generated rust bindings. Do not edit it manually.

/**
 * The GPU the detail lighter's GPU backend traces on, by its PCI vendor and device ids: the renderer's `RenderGpu`,
 * spelled again here so a request names one in a build without the `gpu` feature, which links no graphics code.
 */
export type DetailGpu = {
  vendor: number;
  device: number;
};

/**
 * What traces the detail lighter's shadow rays. The CPU is the reference the SDK's runs are matched against; the GPU
 * traces with hardware ray queries, each slot within one step of the CPU's light, several times faster.
 */
export enum EDetailLightBackend {
  CPU = "cpu",
  /** Refused where the build has no GPU backend or no GPU traces rays. */
  GPU = "gpu",
  /** The GPU where the build has its backend and a GPU traces rays, the CPU otherwise. */
  AUTO = "auto",
}

/** Every `EDetailLightBackend` as the spelling it crosses IPC as, for a value no member has narrowed. */
export type DetailLightBackend = `${EDetailLightBackend}`;

/** How many slots of a level's details the detail lighter met each way. */
export type DetailSlotCounts = {
  /** Planting nothing. */
  unplanted: number;
  /** Planting something, but at no density, and so cleared. */
  emptied: number;
  /** Planted, but touching no face of the ray model, and so left unlit. */
  untouched: number;
  lit: number;
};

/** What a level's compile ran ([`crate::LevelCompiler::compile`]), as the front-ends report it. */
export type LevelCompileReport = {
  level: string;
  /** The directory the SDK's compilers ran in, with their `fsgame.ltx` and logs; none where no step ran one. */
  work: string | null;
  steps: Array<LevelCompileStepReport>;
};

/**
 * A level's compile, as a front-end asks for it: the level's directory the build wrote into, the steps to run, and
 * what they read.
 */
export type LevelCompileRequest = {
  /** The level's directory, `<levels>/<level>`, holding what `level build` wrote; the compile writes beside it. */
  level: string;
  /** The steps to run, each once in [`LevelCompileStep::ALL`]'s order whatever the order given; every step when empty. */
  steps: Array<LevelCompileStep>;
  /**
   * The SDK's `editors` directory, whose `bin` holds xrLC and xrAI; needed only by the steps the SDK still runs
   * ([`LevelCompileStep::is_sdk`]).
   */
  sdk: string | null;
  /** The game data the compilers read their libraries from (`$game_data$`): `gamemtl.xr` and `shaders_xrlc.xr`. */
  gamedata: string;
  /**
   * The textures roots (`$game_textures$`), searched in order for each file: a `.thm` for every texture `build.prj`
   * names, and the image where the texture has alpha or is lit implicitly. The SDK's compilers take exactly one.
   */
  textures: Array<string>;
  /**
   * The directory the SDK's compilers run in, holding the `fsgame.ltx` written for them and their logs; a directory of
   * the system's temporary one named after the level by default.
   */
  work: string | null;
  /** Whether a step of ours overwrites a file it writes where the level's directory holds it, rather than refusing. */
  isForced: boolean;
  /** What traces the details step's shadow rays. */
  detailBackend: DetailLightBackend;
  /** DXC's `dxcompiler.dll`, with which the details step's GPU backend traces on D3D12; Vulkan alone without it. */
  dxc: string | null;
  /** The GPU the details step's GPU backend traces on; the high-performance one without it. */
  detailGpu: DetailGpu | null;
};

/**
 * One step of a level's compile from the files `level build` writes, in the order they run: the geometry (xrLC's
 * part, lightmaps included), the details' light (xrDO_Light's, done by [`crate::DetailLighter`]) and the AI map
 * (xrAI's `-f`). A step no compiler of ours does yet runs the SDK's own ([`Self::is_sdk`]).
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
  /** How long it ran, in milliseconds. */
  duration: number;
  run: LevelCompileStepRun;
};

/** Every `kind` the `LevelCompileStepRun` union is told apart by, so a switch or a comparison names one. */
export enum ELevelCompileStepRun {
  /** One of the SDK's compilers: its path, and its log where it wrote one. */
  PROGRAM = "program",
  /**
   * The detail lighter ([`crate::DetailLighter`]), how it met the slots, and the GPU that traced its shadow rays,
   * none where the CPU traced them.
   */
  DETAIL_LIGHTER = "detailLighter",
}

/** What ran a step of a level's compile, and what it says of the run. */
export type LevelCompileStepRun =
  /** One of the SDK's compilers: its path, and its log where it wrote one. */
  | { kind: "program"; program: string; log: string | null }
  /**
   * The detail lighter ([`crate::DetailLighter`]), how it met the slots, and the GPU that traced its shadow rays,
   * none where the CPU traced them.
   */
  | { kind: "detailLighter"; slots: DetailSlotCounts; gpu: string | null };
