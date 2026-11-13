// Auto-generated rust bindings. Do not edit it manually.

import { LevelAiGraphCensus } from "#/types/xrf-level";
import { Report } from "#/types/xrf-report";

/** Which covers the AI map compiler computes. */
export enum EAiMapCovers {
  /** None, as xrAI's `-draft` leaves them: every value 0, and no `build.cform` needed. */
  NONE = "none",
  /** Each corner node traced against `build.cform` and the other nodes mixed from them, as xrAI's full run does. */
  FULL = "full",
}

/** Every `EAiMapCovers` as the spelling it crosses IPC as, for a value no member has narrowed. */
export type AiMapCovers = `${EAiMapCovers}`;

/**
 * Where a compiled `level.ai`'s graph identity came from.
 *
 * xrAI gives every run a new random identity, which the root spawn's game graph and cross table must carry for the
 * level to load. Ours is the replaced map's when the graph a spawn indexes is unchanged, so the existing spawn still
 * loads the level; otherwise one derived from the nodes, so the same input gives the same file.
 */
export enum EAiMapGuid {
  /** The replaced map's: its nodes are as many, in the same order, at the same cells and with the same links. */
  KEPT = "kept",
  /** Hashed from the compiled nodes, there being no map to replace or its graph having changed. */
  DERIVED = "derived",
}

/** Every `EAiMapGuid` as the spelling it crosses IPC as, for a value no member has narrowed. */
export type AiMapGuid = `${EAiMapGuid}`;

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
  /** What traces the rays: the details step's shadow rays and the AI step's corner rays. */
  backend: RayBackend;
  /** DXC's `dxcompiler.dll`, with which the GPU backend traces on D3D12; Vulkan alone without it. */
  dxc: string | null;
  /** The GPU the GPU backend traces on; the high-performance one without it. */
  gpu: RayGpu | null;
  /** The covers the AI step computes: full ones traced against `build.cform`, or none, needing no `build.cform`. */
  aiCovers: AiMapCovers;
};

/**
 * One step of a level's compile from the files `level build` writes, in the order they run: the geometry (xrLC's
 * part, lightmaps included), the details' light (xrDO_Light's, done by [`crate::DetailLighter`]) and the AI map
 * (xrAI's `-f`, done by [`crate::AiMapCompiler`]). A step no compiler of ours does yet runs the SDK's own
 * ([`Self::is_sdk`]).
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
  /**
   * The AI map compiler ([`crate::AiMapCompiler`]): the covers it computed, the GPU that traced their rays, none where
   * the CPU did or no ray was traced, where the map's identity came from, and the graph check run on the map it wrote,
   * with what the map held.
   */
  AI_MAP_COMPILER = "aiMapCompiler",
}

/** What ran a step of a level's compile, and what it says of the run. */
export type LevelCompileStepRun =
  /** One of the SDK's compilers: its path, and its log where it wrote one. */
  | { kind: "program"; program: string; log: string | null }
  /**
   * The detail lighter ([`crate::DetailLighter`]), how it met the slots, and the GPU that traced its shadow rays,
   * none where the CPU traced them.
   */
  | { kind: "detailLighter"; slots: DetailSlotCounts; gpu: string | null }
  /**
   * The AI map compiler ([`crate::AiMapCompiler`]): the covers it computed, the GPU that traced their rays, none where
   * the CPU did or no ray was traced, where the map's identity came from, and the graph check run on the map it wrote,
   * with what the map held.
   */
  | {
      kind: "aiMapCompiler";
      covers: AiMapCovers;
      gpu: string | null;
      guid: AiMapGuid;
      census: LevelAiGraphCensus;
      graph: Report;
    };

/**
 * What traces the compile's rays through the ray model: the detail lighter's shadow rays and the AI map's corner rays.
 * The CPU is the reference the SDK's runs are matched against; the GPU traces with hardware ray queries, each slot and
 * each cover within one step of the CPU's, several times faster.
 */
export enum ERayBackend {
  CPU = "cpu",
  /** Refused where the build has no GPU backend or no GPU traces rays. */
  GPU = "gpu",
  /** The GPU where the build has its backend and a GPU traces rays, the CPU otherwise. */
  AUTO = "auto",
}

/** Every `ERayBackend` as the spelling it crosses IPC as, for a value no member has narrowed. */
export type RayBackend = `${ERayBackend}`;

/**
 * The GPU the GPU backend traces on, by its PCI vendor and device ids: the renderer's `RenderGpu`,
 * spelled again here so a request names one in a build without the `gpu` feature, which links no graphics code.
 */
export type RayGpu = {
  vendor: number;
  device: number;
};
