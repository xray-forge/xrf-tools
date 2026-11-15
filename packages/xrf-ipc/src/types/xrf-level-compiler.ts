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

/** What a geometry compile made, for its report. */
export type GeometryLevelCensus = {
  /** Static visuals. */
  visuals: number;
  /** Their triangles at full detail. */
  triangles: number;
  /** Static and multi-use visuals made progressive. */
  progressive: number;
  /** Multi-use placements. */
  trees: number;
  sectors: number;
};

/** What a level's compile ran (`LevelCompiler::compile`), as the front-ends report it. */
export type LevelCompileReport = {
  /** The level's directory. */
  level: string;
  /** Each step that ran, in the order it ran. */
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
  /** The game data the compilers read their libraries from (`$game_data$`): `gamemtl.xr` and `shaders_xrlc.xr`. */
  gamedata: string;
  /**
   * The textures roots (`$game_textures$`), searched in order for each file: a `.thm` for every texture `build.prj`
   * names, and the image where the texture has alpha or is lit implicitly.
   */
  textures: Array<string>;
  /** Whether a step overwrites a file it writes where the level's directory holds it, rather than refusing. */
  isForced: boolean;
  /**
   * What traces the rays: the light step's lightmap and implicit map shadow rays, the details step's shadow rays and
   * the AI step's corner rays.
   */
  backend: RayBackend;
  /** DXC's `dxcompiler.dll`, with which the GPU backend traces on D3D12; Vulkan alone without it. */
  dxc: string | null;
  /** The GPU the GPU backend traces on; the high-performance one without it. */
  gpu: RayGpu | null;
  /** The covers the AI step computes: full ones traced against `build.cform`, or none, needing no `build.cform`. */
  aiCovers: AiMapCovers;
  /**
   * How finely the light step lights the level: by `build.prj`'s own settings, or a draft that regenerates the sky and
   * sun and takes fewer samples. It also decides the sky and sun the geometry step writes to `build.lights`.
   */
  lightQuality: LightQuality;
};

/**
 * One step of a level's compile from the files `level build` writes, in the order they run: the geometry (xrLC's
 * geometry phases), its light (xrLC's light phases, baked within the geometry step's pass, since the lightmap pages
 * decide the visuals), the details' light (xrDO_Light's, by `DetailLighter`) and the AI map (xrAI's `-f`, by
 * `AiMapCompiler`).
 */
export enum ELevelCompileStep {
  GEOMETRY = "geometry",
  LIGHT = "light",
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
  /**
   * The detail lighter (`DetailLighter`), how it met the slots, and the GPU that traced its shadow rays, none where
   * the CPU traced them.
   */
  DETAIL_LIGHTER = "detailLighter",
  /** The geometry compiler (`GeometryLevel`), and what it made. */
  GEOMETRY_COMPILER = "geometryCompiler",
  /**
   * The light baker (`LightBaker`), run within the geometry compile, what it made, and the GPU that traced its
   * shadow rays, none where the CPU traced them.
   */
  LIGHT_BAKER = "lightBaker",
  /**
   * The AI map compiler (`AiMapCompiler`): the covers it computed, the GPU that traced their rays, none where
   * the CPU did or no ray was traced, where the map's identity came from, and the graph check run on the map it wrote,
   * with what the map held.
   */
  AI_MAP_COMPILER = "aiMapCompiler",
}

/** What ran a step of a level's compile, and what it says of the run. */
export type LevelCompileStepRun =
  /**
   * The detail lighter (`DetailLighter`), how it met the slots, and the GPU that traced its shadow rays, none where
   * the CPU traced them.
   */
  | { kind: "detailLighter"; slots: DetailSlotCounts; gpu: string | null }
  /** The geometry compiler (`GeometryLevel`), and what it made. */
  | { kind: "geometryCompiler"; census: GeometryLevelCensus }
  /**
   * The light baker (`LightBaker`), run within the geometry compile, what it made, and the GPU that traced its
   * shadow rays, none where the CPU traced them.
   */
  | { kind: "lightBaker"; census: LightBakeCensus; gpu: string | null }
  /**
   * The AI map compiler (`AiMapCompiler`): the covers it computed, the GPU that traced their rays, none where
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

/** What a light bake made, for its report. */
export type LightBakeCensus = {
  /** Lightmap charts. */
  charts: number;
  /** Lightmap pages, each written as `lmap#N_1` and `_2`. */
  lightmaps: number;
  /** Textures lit implicitly, such as terrain's, each written as `<texture>.dds` and `<texture>_lm.dds`. */
  implicitMaps: number;
};

/** How finely a level is lit: by `build.prj`'s own settings, or by a cheaper preset that regenerates what it changes. */
export enum ELightQuality {
  /** `build.prj`'s own: its sky and sun lights as the editor generated them, its sample counts. */
  SCENE = "scene",
  /**
   * The cheap estimate: the sky and the sun regenerated at quality 1 (26 and 9 lights), the lightmaps 3 texels a metre
   * of one sample each, each multi-use model lit from one sample.
   */
  DRAFT = "draft",
}

/** Every `ELightQuality` as the spelling it crosses IPC as, for a value no member has narrowed. */
export type LightQuality = `${ELightQuality}`;

/**
 * What traces the compile's rays through the ray model: the light baker's lightmap and implicit map shadow rays, the
 * detail lighter's shadow rays and the AI map's corner rays.
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
