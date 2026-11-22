// Auto-generated rust bindings. Do not edit it manually.

import { XrayEngine, XrayEngineChoice, XrayEngineResolution } from "#/types/xrf-engine-target";
import {
  EnvironmentFinding,
  LevelWeather,
  SunPosition,
  ThunderboltCollection,
  WeatherCycleKind,
  WeatherDescriptor,
} from "#/types/xrf-environment";
import { JobOutcome } from "#/types/xrf-job";
import { LevelEnvModifier } from "#/types/xrf-level";
import { LevelCompileReport } from "#/types/xrf-level-compiler";
import { Vector2d, Vector3d } from "#/types/xrf-math";
import { LevelBuildReport, LevelDecompileReport } from "#/types/xrf-rawdata";
import {
  RenderBurst,
  RenderCubeCapture,
  RenderHelperIcon,
  RenderMapCapture,
  RenderScreenshot,
  RenderViewportId,
} from "#/types/xrf-renderer";
import { ClsId } from "#/types/xrf-spawn";
import { TextureEncodingFormat, TextureEncodingQuality } from "#/types/xrf-texture";
import { XrayRoots } from "#/types/xrf-vfs";
import { SectorOutline, VisualBounds, VisualTransform } from "#/types/xrf-visual";

/** Building a Level Editor scene into a level directory. */
export const LEVELS_JOB_BUILD = "levels.build" as const;

/** Capturing a cubemap around a viewport's camera, face by face. */
export const LEVELS_JOB_CAPTURE_CUBE = "levels.capture-cube" as const;

/** Capturing a level map in a viewport, tile by tile. */
export const LEVELS_JOB_CAPTURE_MAP = "levels.capture-map" as const;

/** Decompiling a compiled level into a Level Editor scene. */
export const LEVELS_JOB_DECOMPILE = "levels.decompile" as const;

/** The part name a change event uses when a level's AI map goes stale. */
export const LEVEL_AI_MAP_PART = "aiMap" as const;

/** What a change event calls a level's collision form when it goes stale. */
export const LEVEL_COLLISION_PART = "collision" as const;

/** What a change event calls a level's console defaults when they go stale. */
export const LEVEL_CONSOLE_PART = "console" as const;

/** What a change event calls what a level's grass was read from when it goes stale. */
export const LEVEL_DETAILS_PART = "details" as const;

/** What a change event calls a level's environment when it goes stale. */
export const LEVEL_ENVIRONMENT_PART = "environment" as const;

/** What a change event calls a level's render geometry when it goes stale. */
export const LEVEL_GEOMETRY_PART = "geometry" as const;

/** What a change event calls a level's graph when it goes stale. */
export const LEVEL_GRAPH_PART = "graph" as const;

/** What a change event calls a level's hemisphere lighting when it goes stale. */
export const LEVEL_HEMI_PART = "hemi" as const;

/** What a change event calls what a level's lights were read from when it goes stale. */
export const LEVEL_LIGHTS_PART = "lights" as const;

/** What a change event calls where a level's map lies when it goes stale. */
export const LEVEL_MAP_PART = "levelMap" as const;

/**
 * Every kind of level object, in the order `LevelObjectKind` declares them: smart terrains orange, restrictors blue,
 * anomalies red, level changers green, patrols yellow and graph points pale grey as the map has drawn them, the
 * spawned visuals teal, characters magenta, smart covers cyan, placements muted violet and cyan, lights warm white.
 */
export const LEVEL_OBJECT_KINDS: Array<LevelObjectKindEntry> = [
  {
    color: "#26a69a",
    drawDistance: 200.0,
    group: "spawned",
    icon: null,
    isDrawn: true,
    isHelpedByDefault: false,
    kind: "props",
    label: "Props",
  },
  {
    color: "#4db6ac",
    drawDistance: 200.0,
    group: "spawned",
    icon: null,
    isDrawn: true,
    isHelpedByDefault: false,
    kind: "items",
    label: "Items",
  },
  {
    color: "#00897b",
    drawDistance: 200.0,
    group: "spawned",
    icon: null,
    isDrawn: true,
    isHelpedByDefault: false,
    kind: "weapons",
    label: "Weapons",
  },
  {
    color: "#80cbc4",
    drawDistance: 200.0,
    group: "spawned",
    icon: null,
    isDrawn: true,
    isHelpedByDefault: false,
    kind: "lamps",
    label: "Lamps",
  },
  {
    color: "#f06292",
    drawDistance: 300.0,
    group: "characters",
    icon: "actorStart",
    isDrawn: false,
    isHelpedByDefault: false,
    kind: "actor",
    label: "Actor start",
  },
  {
    color: "#e040fb",
    drawDistance: 300.0,
    group: "characters",
    icon: "stalker",
    isDrawn: false,
    isHelpedByDefault: false,
    kind: "stalkers",
    label: "Stalkers",
  },
  {
    color: "#ab47bc",
    drawDistance: 300.0,
    group: "characters",
    icon: "creature",
    isDrawn: false,
    isHelpedByDefault: false,
    kind: "creatures",
    label: "Creatures",
  },
  {
    color: "#ce93d8",
    drawDistance: 300.0,
    group: "characters",
    icon: "trader",
    isDrawn: false,
    isHelpedByDefault: false,
    kind: "traders",
    label: "Traders",
  },
  {
    color: "#ba68c8",
    drawDistance: 300.0,
    group: "characters",
    icon: "squad",
    isDrawn: false,
    isHelpedByDefault: false,
    kind: "squads",
    label: "Squads",
  },
  {
    color: "#ff9800",
    drawDistance: 600.0,
    group: "volumes",
    icon: null,
    isDrawn: false,
    isHelpedByDefault: true,
    kind: "smartTerrains",
    label: "Smart terrains",
  },
  {
    color: "#2196f3",
    drawDistance: 250.0,
    group: "volumes",
    icon: null,
    isDrawn: false,
    isHelpedByDefault: true,
    kind: "restrictors",
    label: "Restrictors",
  },
  {
    color: "#f44336",
    drawDistance: 250.0,
    group: "volumes",
    icon: null,
    isDrawn: false,
    isHelpedByDefault: true,
    kind: "anomalies",
    label: "Anomalies",
  },
  {
    color: "#4caf50",
    drawDistance: 600.0,
    group: "volumes",
    icon: null,
    isDrawn: false,
    isHelpedByDefault: true,
    kind: "levelChangers",
    label: "Level changers",
  },
  {
    color: "#a1887f",
    drawDistance: 150.0,
    group: "volumes",
    icon: null,
    isDrawn: false,
    isHelpedByDefault: true,
    kind: "climbables",
    label: "Ladders",
  },
  {
    color: "#00bcd4",
    drawDistance: 150.0,
    group: "covers",
    icon: "smartCover",
    isDrawn: false,
    isHelpedByDefault: false,
    kind: "smartCovers",
    label: "Smart covers",
  },
  {
    color: "#ffeb3b",
    drawDistance: 250.0,
    group: "paths",
    icon: "patrolPoint",
    isDrawn: false,
    isHelpedByDefault: true,
    kind: "patrols",
    label: "Patrols",
  },
  {
    color: "#e0e0e0",
    drawDistance: 150.0,
    group: "navigation",
    icon: "graphPoint",
    isDrawn: false,
    isHelpedByDefault: false,
    kind: "graphPoints",
    label: "Graph points",
  },
  {
    color: "#b39ddb",
    drawDistance: 150.0,
    group: "placements",
    icon: "particles",
    isDrawn: false,
    isHelpedByDefault: false,
    kind: "staticParticles",
    label: "Static particles",
  },
  {
    color: "#80deea",
    drawDistance: 150.0,
    group: "placements",
    icon: "sound",
    isDrawn: false,
    isHelpedByDefault: false,
    kind: "staticSounds",
    label: "Static sounds",
  },
  {
    color: "#4dd0e1",
    drawDistance: 150.0,
    group: "placements",
    icon: null,
    isDrawn: false,
    isHelpedByDefault: false,
    kind: "soundEnvironments",
    label: "Sound environments",
  },
  {
    color: "#9575cd",
    drawDistance: 150.0,
    group: "placements",
    icon: null,
    isDrawn: false,
    isHelpedByDefault: false,
    kind: "fogVolumes",
    label: "Fog volumes",
  },
  {
    color: "#7e57c2",
    drawDistance: 150.0,
    group: "placements",
    icon: null,
    isDrawn: false,
    isHelpedByDefault: false,
    kind: "envModifiers",
    label: "Weather modifiers",
  },
  {
    color: "#9fa8da",
    drawDistance: 150.0,
    group: "placements",
    icon: "respawnPoint",
    isDrawn: false,
    isHelpedByDefault: false,
    kind: "respawnPoints",
    label: "Respawn points",
  },
  {
    color: "#fff3c4",
    drawDistance: 200.0,
    group: "lights",
    icon: "light",
    isDrawn: false,
    isHelpedByDefault: false,
    kind: "lights",
    label: "Lights",
  },
];

/** What a change event calls what a level's object motions were read from when it goes stale. */
export const LEVEL_OBJECT_MOTIONS_PART = "objectMotions" as const;

/** What a change event calls what a level's particle systems were read from when it goes stale. */
export const LEVEL_PARTICLES_PART = "particles" as const;

/** What a change event calls a level's placements when they go stale. */
export const LEVEL_PLACEMENTS_PART = "placements" as const;

/** What a change event calls a level's sections when they go stale. */
export const LEVEL_SECTIONS_PART = "sections" as const;

/**
 * The helper kind bit, after the kind table's, that switches the sound emitter helpers: each static and zone sound's
 * spheres at its minimum and maximum distance.
 */
export const LEVEL_SOUND_EMITTERS_HELPER_KIND: number = 24;

/** The part name a change event uses when the sound files a level reads go stale. */
export const LEVEL_SOUND_FILES_PART = "soundFiles" as const;

/** What a change event calls a level's spawn when it goes stale. */
export const LEVEL_SPAWN_PART = "spawn" as const;

/** The part name a change event uses when a level's spawned visuals go stale. */
export const LEVEL_SPAWN_VISUALS_PART = "spawnVisuals" as const;

/** What a change event calls where a level opens when it goes stale. */
export const LEVEL_START_PART = "start" as const;

/** What a change event calls how a level's surfaces are drawn when it goes stale. */
export const LEVEL_SURFACES_PART = "surfaces" as const;

/** What a change event calls what a level's surfaces' textures resolved to when it goes stale. */
export const LEVEL_TEXTURES_PART = "textures" as const;

/** What a change event calls what a level's weather plays with beyond its environment when it goes stale. */
export const LEVEL_WEATHER_PART = "weather" as const;

/** The change event name for stale binary tool counts. */
export const SCENE_BINARY_TOOLS_PART = "sceneBinaryTools" as const;

/** The change event name for stale drawn scene objects. */
export const SCENE_DRAWN_OBJECTS_PART = "sceneDrawnObjects" as const;

/** The change event name for a stale scene draw description. */
export const SCENE_DRAW_PART = "sceneDraw" as const;

/** The change event name for stale scene helpers. */
export const SCENE_HELPERS_PART = "sceneHelpers" as const;

/**
 * Every tool whose objects the editor draws a helper for, in the editor's tool order. Each with a counterpart in the
 * level viewer's kind table looks as it does there: spawn points as its characters, ways as its patrols, shapes as its
 * restrictors, lights, sounds, particles and fog volumes as their own; glows are amber and portals pale green, which
 * nothing else is.
 */
export const SCENE_HELPER_TOOLS: Array<SceneHelperToolEntry> = [
  { color: "#ffb74d", drawDistance: 150.0, icon: null, isHelpedByDefault: true, kind: 1, tool: "glow" },
  { color: "#fff3c4", drawDistance: 200.0, icon: "light", isHelpedByDefault: true, kind: 3, tool: "light" },
  { color: "#2196f3", drawDistance: 250.0, icon: null, isHelpedByDefault: true, kind: 4, tool: "shape" },
  { color: "#80deea", drawDistance: 150.0, icon: "sound", isHelpedByDefault: true, kind: 5, tool: "sound_src" },
  { color: "#e040fb", drawDistance: 300.0, icon: "spawnPoint", isHelpedByDefault: true, kind: 6, tool: "spawn" },
  { color: "#ffeb3b", drawDistance: 250.0, icon: "patrolPoint", isHelpedByDefault: true, kind: 7, tool: "way" },
  { color: "#aed581", drawDistance: 250.0, icon: null, isHelpedByDefault: true, kind: 9, tool: "portal" },
  { color: "#4dd0e1", drawDistance: 150.0, icon: null, isHelpedByDefault: true, kind: 10, tool: "sound_env" },
  { color: "#b39ddb", drawDistance: 150.0, icon: "particles", isHelpedByDefault: true, kind: 11, tool: "ps" },
  { color: "#9575cd", drawDistance: 150.0, icon: null, isHelpedByDefault: true, kind: 15, tool: "fog_volume" },
];

/** The change event name for stale scene objects. */
export const SCENE_OBJECTS_PART = "sceneObjects" as const;

/**
 * The scene render id bit, above the tool bits, naming the model of an object's collision surfaces apart from its
 * own: the renderer holds one model per spawn object, so each model needs an id.
 */
export const SCENE_RENDER_COLLISION_BIT: number = 2147483648;

/**
 * The scene render id bit, above the tool bits, naming the model of an object's editor-only surfaces apart from its
 * own.
 */
export const SCENE_RENDER_EDITOR_BIT: number = 1073741824;

/**
 * The low bits of a scene render id, holding its object's index among its tool's objects; the bits above hold the
 * tool's place in the editor's tool order.
 */
export const SCENE_RENDER_INDEX_BITS: number = 24;

/**
 * Every tool in the editor's order, which a scene render id's tool bits index. Only the page reads it; the backend
 * takes a tool's place from its variant.
 */
export const SCENE_RENDER_TOOLS: Array<SceneToolKind> = [
  "group",
  "glow",
  "scene_object",
  "light",
  "shape",
  "sound_src",
  "spawn",
  "way",
  "sector",
  "portal",
  "sound_env",
  "ps",
  "detail_object",
  "ai_map",
  "wallmark",
  "fog_volume",
];

/** The change event name for stale spawn models. */
export const SCENE_SPAWN_MODELS_PART = "sceneSpawnModels" as const;

/** The change event name for stale spawn visuals. */
export const SCENE_SPAWN_VISUALS_PART = "sceneSpawnVisuals" as const;

/**
 * A burst asked of a viewport: `name` is its folder among the level viewer's bursts, `has_sound` records what the
 * camera hears as `sound.wav`, and `has_video` writes the frames with that sound as `burst.mp4`.
 */
export type BurstCaptureRequest = {
  viewport: RenderViewportId;
  burst: RenderBurst;
  name: string;
  hasSound: boolean;
  hasVideo: boolean;
};

/**
 * A written burst: its folder, each frame's scene time in seconds in write order, the frame and marks sizes, and its
 * sound and video when asked for.
 */
export type BurstOutcome = {
  folder: string;
  times: Array<number | null>;
  width: number;
  height: number;
  marksWidth: number;
  marksHeight: number;
  sound: BurstSoundOutcome | null;
  video: BurstVideoOutcome | null;
};

/**
 * A written burst sound: its file, sample rate and channels, and the sound frame each burst frame starts at, in write
 * order.
 */
export type BurstSoundOutcome = {
  file: string;
  rate: number;
  channels: number;
  offsets: Array<number>;
};

/** A written burst video: its file, frame count, duration in seconds, size, and the H.264 encoder's name. */
export type BurstVideoOutcome = {
  file: string;
  frames: number;
  duration: number | null;
  width: number;
  height: number;
  encoder: string;
};

/** A cubemap capture asked of a viewport, written as a cross PNG to `path`. */
export type CubeCaptureRequest = {
  viewport: RenderViewportId;
  capture: RenderCubeCapture;
  path: string;
};

/** Every `kind` the `CubeExportFormat` union is told apart by, so a switch or a comparison names one. */
export enum ECubeExportFormat {
  PNG = "png",
  TGA = "tga",
  DDS = "dds",
}

/**
 * A cubemap export format: one cube DDS, six TGAs named `1` to `6` as `CDemoRecord` writes them, or six PNGs named by
 * face.
 */
export type CubeExportFormat =
  | { kind: "png" }
  | { kind: "tga" }
  | ({
      kind: "dds";
    } & ViewerDdsExport);

/** A request to re-export a cubemap capture in another format or size. */
export type CubeExportRequest = {
  /** The capture's cross PNG in the screenshots folder. */
  path: string;
  /** The DDS file, or the folder the six faces go into. */
  target: string;
  format: CubeExportFormat;
  /** Face side in pixels, at most the capture's. */
  size: number;
};

/** One cycle or effect of a catalog, as a list shows it. */
export type EnvironmentCycleEntry = {
  name: string;
  file: string;
  kind: WeatherCycleKind;
  keyframes: number;
  /** Findings in its config. */
  findings: number;
};

/**
 * How a scene's build, and the compile after it, ended: the build's report once it finished, and the compile's once
 * one was asked for and finished.
 */
export type LevelBuildResult = {
  outcome: JobOutcome;
  report: LevelBuildReport | null;
  compile: LevelCompileReport | null;
};

/**
 * What a game's console sets of how its levels are lit, exposed and corrected: its installation's `user.ltx` over its
 * shipped defaults (Anomaly's `default_controls.ltx`); each `None` it leaves to the engine.
 */
export type LevelConsoleDefaults = {
  /** Whether the game ships console defaults at all. */
  isShipped: boolean;
  /** `r2_tonemap`. */
  isTonemapped: boolean | null;
  /** `r2_tonemap_amount`, `r2_tonemap_middlegray`, `r2_tonemap_lowlum` and `r2_tonemap_adaptation`. */
  tonemapAmount: number | null;
  tonemapMiddleGray: number | null;
  tonemapLowLuminance: number | null;
  tonemapAdaptation: number | null;
  /** `r2_sun_lumscale`, `r2_sun_lumscale_hemi` and `r2_sun_lumscale_amb`. */
  sunScale: number | null;
  hemiScale: number | null;
  ambientScale: number | null;
  /** `r__exposure`, `r__gamma`, `r__saturation` and `r__color_grading`. */
  imageExposure: number | null;
  imageGamma: number | null;
  imageSaturation: number | null;
  colorGrading: [number | null, number | null, number | null] | null;
  /** `r2_ls_bloom_threshold`, `r2_ls_bloom_kernel_g` and `r2_ls_bloom_kernel_scale`. */
  bloomThreshold: number | null;
  bloomRadius: number | null;
  bloomStrength: number | null;
};

/** How a level's decompile into a scene ended, with its report once it completed. */
export type LevelDecompileResult = {
  outcome: JobOutcome;
  report: LevelDecompileReport | null;
};

/** One compiled level the roots hold, as a picker lists it. */
export type LevelEntry = {
  /** The name the installation knows the level by, which is its directory under `levels`. */
  name: string;
  /** Its engine identity, which is what opening it takes. */
  logicalPath: string;
  /** Whether `level.geom` sits beside the bundle; a level without it draws nothing. */
  hasGeometry: boolean;
};

/** The ground a level map covers, engine space: its least and greatest `x` and `z`, as `bound_rect` places it. */
export type LevelMapArea = {
  /** The first corner, the least `x` and `z`. */
  minX: number | null;
  minZ: number | null;
  /** The second corner, the greatest `x` and `z`. */
  maxX: number | null;
  maxZ: number | null;
};

/** Where the PDA lays the level's map over it, as the level's `level.ltx` places it in `[level_map]`. */
export type LevelMapDescription = {
  /** The ground the map covers, as `bound_rect` places it. */
  area: LevelMapArea;
  /** The map's texture, as `texture` names it, where it names one. */
  texture: string | null;
};

/**
 * One discrete thing placed in a level: what it is, where it stands, what it belongs to, and what its source records
 * of it.
 */
export type LevelObject = {
  id: LevelObjectId;
  kind: LevelObjectKind;
  name: string;
  /** Where it stands, its `XFORM` in renderer space; its shapes stand in the space it gives. */
  transform: VisualTransform;
  /** The object it is a part of, as a patrol point is of its path, listed before it. */
  parent: LevelObjectId | null;
  /** What the spawn records of it, for an object the spawn places. */
  spawn: LevelObjectSpawn | null;
  /** What it covers, for a logic volume. */
  volume: LevelObjectVolume | null;
  /** Its points and their links, for a patrol path. */
  patrol: LevelObjectPatrol | null;
  /** What the spawn records of it, for a patrol point. */
  patrolPoint: LevelObjectPatrolPoint | null;
  /** What its companion file records of it, for a placement. */
  placement: LevelObjectPlacement | null;
  /** What the bundle records of it, for a static light. */
  light: LevelObjectLight | null;
};

/** The family a level object's kind belongs to, which the Outliner and the Show popover nest kinds under. */
export enum ELevelObjectGroup {
  /** The spawn's objects drawn as a model. */
  SPAWNED = "spawned",
  /** Who the spawn places, drawn as nothing: the actor's start, stalkers, creatures, traders and squads. */
  CHARACTERS = "characters",
  /** The logic volumes: smart terrains, restrictors, anomalies, level changers and ladders. */
  VOLUMES = "volumes",
  /** The smart covers. */
  COVERS = "covers",
  /** The patrol paths and their points. */
  PATHS = "paths",
  /** The game graph's points. */
  NAVIGATION = "navigation",
  /**
   * What the level's companion files place: planted particles, static sounds, sound environments, fog volumes,
   * environment modifiers and respawn points.
   */
  PLACEMENTS = "placements",
  /** The level's static lights. */
  LIGHTS = "lights",
}

/** Every `ELevelObjectGroup` as the spelling it crosses IPC as, for a value no member has narrowed. */
export type LevelObjectGroup = `${ELevelObjectGroup}`;

/** Every `source` the `LevelObjectId` union is told apart by, so a switch or a comparison names one. */
export enum ELevelObjectId {
  /** An object of the level's spawn, by its index among them. */
  SPAWN = "spawn",
  /** A patrol path starting on the level, by its index among them in the spawn's order. */
  PATROL = "patrol",
  /** One point of a patrol path, by its index along the path. */
  PATROL_POINT = "patrolPoint",
  /** A game graph vertex on the level, by its id. */
  GRAPH_POINT = "graphPoint",
  /** A record of one of the level's companion files, by its index among the file's records. */
  PLACEMENT = "placement",
  /** One of the level's static lights, by its index among the bundle's. */
  LIGHT = "light",
}

/**
 * What names one level object for as long as its level is open, by where it is read from: the selection, a pick, the
 * Outliner's rows and the renderer's helpers all address objects by it.
 */
export type LevelObjectId =
  /** An object of the level's spawn, by its index among them. */
  | { source: "spawn"; index: number }
  /** A patrol path starting on the level, by its index among them in the spawn's order. */
  | { source: "patrol"; path: number }
  /** One point of a patrol path, by its index along the path. */
  | { source: "patrolPoint"; path: number; point: number }
  /** A game graph vertex on the level, by its id. */
  | { source: "graphPoint"; vertex: number }
  /** A record of one of the level's companion files, by its index among the file's records. */
  | { source: "placement"; file: LevelPlacementFile; index: number }
  /** One of the level's static lights, by its index among the bundle's. */
  | { source: "light"; index: number };

/** What a level object is, which the viewer lists, shows and draws helpers by. */
export enum ELevelObjectKind {
  PROPS = "props",
  ITEMS = "items",
  WEAPONS = "weapons",
  LAMPS = "lamps",
  /** Where the actor starts. */
  ACTOR = "actor",
  STALKERS = "stalkers",
  /** Monsters, crows and phantoms. */
  CREATURES = "creatures",
  TRADERS = "traders",
  SQUADS = "squads",
  SMART_TERRAINS = "smartTerrains",
  RESTRICTORS = "restrictors",
  /** Anomalous and torrid zones. */
  ANOMALIES = "anomalies",
  LEVEL_CHANGERS = "levelChangers",
  /** Ladders. */
  CLIMBABLES = "climbables",
  SMART_COVERS = "smartCovers",
  /** Patrol paths, and the points along them. */
  PATROLS = "patrols",
  /** The game graph's vertices on the level. */
  GRAPH_POINTS = "graphPoints",
  /** Particle systems `level.ps_static` plants for a single player game. */
  STATIC_PARTICLES = "staticParticles",
  /** Sounds `level.snd_static` plays where they stand. */
  STATIC_SOUNDS = "staticSounds",
  /** The reverb environments' boxes of `level.snd_env`. */
  SOUND_ENVIRONMENTS = "soundEnvironments",
  /** The volumetric fog's boxes of `level.fog_vol`. */
  FOG_VOLUMES = "fogVolumes",
  /** The spheres of `level.env_mod` that move the weather toward their own. */
  ENV_MODIFIERS = "envModifiers",
  /** Where a multiplayer game respawns actors, artefacts and items (`level.game`); a single player game reads none. */
  RESPAWN_POINTS = "respawnPoints",
  /** The level's static lights, the sun aside. */
  LIGHTS = "lights",
}

/** Every `ELevelObjectKind` as the spelling it crosses IPC as, for a value no member has narrowed. */
export type LevelObjectKind = `${ELevelObjectKind}`;

/**
 * One kind's row of the kind table: what the Outliner, the Show popover, the helpers and the map all name, colour,
 * picture and switch it by.
 */
export type LevelObjectKindEntry = {
  kind: LevelObjectKind;
  group: LevelObjectGroup;
  label: string;
  /** Its helpers' colour, `#rrggbb` in sRGB. */
  color: string;
  /** The icon its objects' helpers draw where they stand, for a kind whose objects have no shape of their own. */
  icon: RenderHelperIcon | null;
  /** Metres from the camera past which its helpers fade out. */
  drawDistance: number | null;
  /** Whether its objects have a picture of their own, which the Visible switch shows or hides. */
  isDrawn: boolean;
  /** Whether its helpers are drawn until the viewer is told otherwise. */
  isHelpedByDefault: boolean;
};

/**
 * What the bundle records of one static light (`CLight_DB::Load`): whether it was authored as a spot, its colour and
 * its range in metres. Every engine lights a spot as a point of that range and colour, its cone unread.
 */
export type LevelObjectLight = {
  isSpot: boolean;
  /** The diffuse colour, linear, as authored. */
  color: [number | null, number | null, number | null];
  range: number | null;
};

/** What a patrol path is drawn and described by: its points where they stand and the links between them. */
export type LevelObjectPatrol = {
  /** Every point where it stands, renderer space, by index along the path. */
  points: Array<Vector3d>;
  /** The links between them, those reaching past its points left out. */
  links: Array<LevelPatrolLink>;
};

/**
 * What the spawn records of one patrol point beyond its name and place: its flags, which a logic scheme reads, and
 * the level and game graph vertices it stands at.
 */
export type LevelObjectPatrolPoint = {
  flags: number;
  levelVertex: number;
  gameVertex: number;
};

/**
 * What one of the level's companion files records of a placement: the shapes it covers in its object's own space,
 * renderer space, none for one placed at a point, and what its file records of it.
 */
export type LevelObjectPlacement = {
  shapes: Array<LevelObjectShape>;
  details: LevelPlacementDetails;
};

/** Every `kind` the `LevelObjectShape` union is told apart by, so a switch or a comparison names one. */
export enum ELevelObjectShape {
  SPHERE = "sphere",
  /** The unit cube about the origin taken through `axes` and moved to `center`. */
  BOX = "box",
}

/** One shape of a logic volume in its object's own space, renderer space: what `CCF_Shape` tests against. */
export type LevelObjectShape =
  | { kind: "sphere"; center: Vector3d; radius: number | null }
  /** The unit cube about the origin taken through `axes` and moved to `center`. */
  | { kind: "box"; axes: [Vector3d, Vector3d, Vector3d]; center: Vector3d };

/**
 * What the spawn records of a level object it places: its section, class and story id, why a new game releases it,
 * and the visual it is drawn as.
 */
export type LevelObjectSpawn = {
  section: string;
  clsid: ClsId;
  /** Absent for an object without one (`INVALID_STORY_ID`). */
  storyId: number | null;
  /** Why a new game releases it, absent for an object it keeps; a released one is drawn only when asked. */
  release: LevelSpawnRelease | null;
  /** The visual it is drawn as, by its index among the description's visuals; none for one drawn as nothing. */
  visual: number | null;
};

/** What a logic volume covers and does: its shapes, whom it restricts, and the level a changer leads to. */
export type LevelObjectVolume = {
  /** In the object's own space, renderer space; none for a volume the spawn gives no shape. */
  shapes: Array<LevelObjectShape>;
  restriction: LevelRestriction;
  /** The level a level changer leads to, by name. */
  destination: string | null;
};

/**
 * Every level object of a level, in the order its sources keep them, and the visuals they are drawn as, each named
 * once.
 */
export type LevelObjectsDescription = {
  visuals: Array<string>;
  objects: Array<LevelObject>;
};

/** What opening a level was asked to read, and how the game's configs beside it are read. */
export type LevelOpenRequest = {
  source: LevelSource;
  /** Trees the level and its textures are searched in. */
  roots: XrayRoots;
  /** Whether the game's configs resolve with the Monolith/Anomaly DLTX patch dialect. */
  isDltx: boolean;
  /** Which engine the game's configs, its weather among them, are read as; detected on `auto`. */
  engine: XrayEngineChoice;
};

/**
 * One way a patrol goes on from a point: the point it leads to, by index along the path, and how likely it is taken
 * among the point's others.
 */
export type LevelPatrolLink = {
  from: number;
  to: number;
  weight: number | null;
};

/** Every `kind` the `LevelPlacementDetails` union is told apart by, so a switch or a comparison names one. */
export enum ELevelPlacementDetails {
  /** A planted particle system, by the effect or group it plays. */
  PARTICLES = "particles",
  /**
   * A static sound (`SStaticSound`): the sound, its volume and frequency, and its windows: the game day's hours it
   * plays in, how long each play lasts and the pause between.
   */
  SOUND = "sound",
  /** A reverb environment's box: the environment outside it and inside it, by name. */
  SOUND_ENVIRONMENT = "soundEnvironment",
  /** A volumetric fog body: the profile under `$game_config$` it is drawn by, and how many obstacles cut it. */
  FOG_VOLUME = "fogVolume",
  /**
   * An environment modifier (`CEnvModifier`): how strongly it pulls the weather at its centre, fading to none at its
   * radius, toward its far plane, fog colour and density, ambient, sky and hemisphere colours, and which of those it
   * moves, by name.
   */
  ENV_MODIFIER = "envModifier",
  /**
   * A multiplayer respawn point: what it respawns, its team, the game types it serves as a bit mask, and the item
   * preset it spawns, empty for any other.
   */
  RESPAWN_POINT = "respawnPoint",
}

/** What a placement's file records of it beyond where it stands and the shapes it covers. */
export type LevelPlacementDetails =
  /** A planted particle system, by the effect or group it plays. */
  | { kind: "particles"; effect: string }
  /**
   * A static sound (`SStaticSound`): the sound, its volume and frequency, and its windows: the game day's hours it
   * plays in, how long each play lasts and the pause between.
   */
  | {
      kind: "sound";
      sound: string;
      volume: number | null;
      frequency: number | null;
      active: LevelSoundWindow;
      play: LevelSoundWindow;
      pause: LevelSoundWindow;
    }
  /** A reverb environment's box: the environment outside it and inside it, by name. */
  | { kind: "soundEnvironment"; outer: string; inner: string }
  /** A volumetric fog body: the profile under `$game_config$` it is drawn by, and how many obstacles cut it. */
  | { kind: "fogVolume"; profile: string; obstacles: number }
  /**
   * An environment modifier (`CEnvModifier`): how strongly it pulls the weather at its centre, fading to none at its
   * radius, toward its far plane, fog colour and density, ambient, sky and hemisphere colours, and which of those it
   * moves, by name.
   */
  | {
      kind: "envModifier";
      power: number | null;
      farPlane: number | null;
      fogColor: [number | null, number | null, number | null];
      fogDensity: number | null;
      ambient: [number | null, number | null, number | null];
      skyColor: [number | null, number | null, number | null];
      hemiColor: [number | null, number | null, number | null];
      affects: Array<string>;
    }
  /**
   * A multiplayer respawn point: what it respawns, its team, the game types it serves as a bit mask, and the item
   * preset it spawns, empty for any other.
   */
  | { kind: "respawnPoint"; role: string; team: number; gameTypes: number; profile: string };

/** Which of the level's companion files a placement is read from. */
export enum ELevelPlacementFile {
  /** `level.ps_static`. */
  PARTICLES = "particles",
  /** `level.snd_static`. */
  SOUNDS = "sounds",
  /** `level.snd_env`. */
  SOUND_ENVIRONMENTS = "soundEnvironments",
  /** `level.fog_vol`. */
  FOG_VOLUMES = "fogVolumes",
  /** `level.env_mod`. */
  ENV_MODIFIERS = "envModifiers",
  /** `level.game`'s respawn points. */
  RESPAWN_POINTS = "respawnPoints",
}

/** Every `ELevelPlacementFile` as the spelling it crosses IPC as, for a value no member has narrowed. */
export type LevelPlacementFile = `${ELevelPlacementFile}`;

/**
 * Whom a logic volume keeps where, `RestrictionSpace::ERestrictorTypes` with the default and the scripted kinds
 * folded together.
 */
export enum ELevelRestriction {
  /** Restricts no one by itself. */
  NONE = "none",
  /** Keeps those it restricts outside it. */
  OUT = "out",
  /** Keeps those it restricts inside it. */
  IN = "in",
}

/** Every `ELevelRestriction` as the spelling it crosses IPC as, for a value no member has narrowed. */
export type LevelRestriction = `${ELevelRestriction}`;

/**
 * One of a static sound's windows, in milliseconds, as `SStaticSound` compares a clock against it: from and to, both
 * nought for the sound's own default (all day, its whole length, looped without a pause).
 */
export type LevelSoundWindow = {
  from: number;
  to: number;
};

/** Every `kind` the `LevelSource` union is told apart by, so a switch or a comparison names one. */
export enum ELevelSource {
  /** A compiled level directory on disk, named by its filesystem path. */
  DIRECTORY = "directory",
  /** A level of the mounted roots, named by its engine identity. */
  ASSET = "asset",
}

/** Where a compiled level is read from. */
export type LevelSource =
  /** A compiled level directory on disk, named by its filesystem path. */
  | { kind: "directory"; path: string }
  /** A level of the mounted roots, named by its engine identity. */
  | { kind: "asset"; logicalPath: string };

/** What the viewer shows of one spawned object beyond where it stands: what the game knows it by and what it carries. */
export type LevelSpawnObjectDetails = {
  /** Its ALife id in the spawn. */
  id: number;
  /**
   * The game graph vertex it stands at, which places it among the levels; none for an object without the abstract
   * part that carries it.
   */
  gameVertexId: number | null;
  /** The level graph vertex it stands at, which places it on the level's AI map; none where it has none. */
  levelVertexId: number | null;
  /** Its `custom_data`, the logic and settings a script reads, empty where it has none. */
  customData: string;
};

/**
 * Why a new game releases a spawned object before the actor first stands on its level, as Anomaly's
 * `game_setup.script` does: the config and section naming it.
 */
export enum ELevelSpawnRelease {
  /** `configs/plugins/new_game_setup.ltx`, `[remove_objects]`: removed outright. */
  REMOVE_OBJECTS = "removeObjects",
  /**
   * `configs/items/settings/dynamic_item_spawn.ltx`, `[replace_items]`: replaced by an item the script places, which
   * the viewer does not place.
   */
  REPLACE_ITEMS = "replaceItems",
}

/** Every `ELevelSpawnRelease` as the spelling it crosses IPC as, for a value no member has narrowed. */
export type LevelSpawnRelease = `${ELevelSpawnRelease}`;

/** Where a level opens, in renderer space: an actor's eye where the game puts one on the level. */
export type LevelStart = {
  /** Where the camera stands. */
  position: Vector3d;
  /** Which way it looks, level with the horizon. */
  direction: Vector3d;
};

/** The sun xrLC compiled the level against, as the light chunk records it. */
export type LevelSunDescription = {
  /** Where the light travels, in the level's own axes. */
  direction: Vector3d;
  /** Its diffuse colour, as the compiler stored it. */
  color: [number | null, number | null, number | null];
};

/** One texture a level's shader table names, and what it came to. */
export type LevelTextureReference = {
  /** The reference as the shader table spells it, which is what a surface names. */
  reference: string;
  /** What it resolved to, or `None` for a reference the roots hold nothing for. */
  logicalPath: string | null;
};

/** One cycle or effect as the engine loads it, which a viewer mixes, and what is wrong in its config. */
export type LevelWeatherCycle = {
  name: string;
  file: string;
  kind: WeatherCycleKind;
  /** Sorted by time, a keyframe whose name the engine refuses left out. */
  keyframes: Array<WeatherDescriptor>;
  findings: Array<EnvironmentFinding>;
};

/** Everything a viewer plays the open level's weather from, as its engine loads it. */
export type LevelWeatherDescription = {
  engine: XrayEngine;
  /** What the level's `weathers` resolves to. */
  weather: LevelWeather;
  /** The cycles it resolves to that the game has, in the order it offers them. */
  offered: Array<LevelWeatherCycle>;
  /** Every cycle of the game, which a viewer offers after the level's own. */
  cycles: Array<EnvironmentCycleEntry>;
  /** Every weather effect, which the game plays over a cycle. */
  effects: Array<LevelWeatherCycle>;
  /** Every thunderbolt collection of the game, which a keyframe set by hand may strike with as well as the level's own. */
  thunderboltCollections: Array<ThunderboltCollection>;
  /** Monolith's table of where the sun stands, midnight first; none on OpenXRay. */
  sunTable: Array<SunPosition> | null;
  /** The level's local overrides, `level.env_mod`; none where it has none. */
  modifiers: Array<LevelEnvModifier>;
  /** Every sky the cycles and effects name, by reference, for a hand-set keyframe to be given. */
  skies: Array<LevelWeatherTexture>;
  /** Every clouds texture they name. */
  clouds: Array<LevelWeatherTexture>;
  /**
   * Every lens flare of the game a keyframe can name by `sun`: `suns.ltx`'s sections, and on OpenXRay any `system.ltx`
   * one a keyframe names in their place.
   */
  suns: Array<string>;
};

/** A texture the game's weather names, which a hand-set keyframe may be given. */
export type LevelWeatherTexture = {
  /** The reference, and what it resolves to beside the level. */
  texture: LevelTextureReference;
  /** How many keyframes of the cycles and effects name it. */
  uses: number;
};

/**
 * Annotations asked of a level map: `path` is its PNG in the screenshots folder, the SVG is written beside it, `area`
 * is the ground it covers, `kinds` the object kinds drawn, and `is_labelled` names every object.
 */
export type MapAnnotationsWriteRequest = {
  path: string;
  area: LevelMapArea;
  kinds: Array<LevelObjectKind>;
  isLabelled: boolean;
};

/** A level map capture asked of a viewport, written as a PNG to `path`. */
export type MapCaptureRequest = {
  viewport: RenderViewportId;
  capture: RenderMapCapture;
  path: string;
};

/** How a map export includes the annotations SVG written beside its capture. */
export enum EMapExportAnnotations {
  /** Drawn over the map. */
  OVER = "over",
  /** Alone, transparent where they draw nothing. */
  ALONE = "alone",
}

/** Every `EMapExportAnnotations` as the spelling it crosses IPC as, for a value no member has narrowed. */
export type MapExportAnnotations = `${EMapExportAnnotations}`;

/** Every `kind` the `MapExportFormat` union is told apart by, so a switch or a comparison names one. */
export enum EMapExportFormat {
  PNG = "png",
  TGA = "tga",
  SVG = "svg",
  DDS = "dds",
}

/**
 * A level map export format: a PNG, an uncompressed TGA as `CDemoRecord` writes, a DDS as the game loads maps, or SVG
 * for the annotations alone.
 */
export type MapExportFormat =
  | { kind: "png" }
  | { kind: "tga" }
  | { kind: "svg" }
  | ({
      kind: "dds";
    } & ViewerDdsExport);

/** An exported level map's size in pixels; the capture's size for an annotations-only SVG. */
export type MapExportOutcome = {
  width: number;
  height: number;
};

/** A request to re-export a level map capture in another format or size. */
export type MapExportRequest = {
  /** The capture's PNG in the screenshots folder, with coverage in alpha. */
  path: string;
  target: string;
  format: MapExportFormat;
  /** The longer side in pixels, at most the capture's; the shorter keeps the capture's aspect. */
  longSide: number;
  /** A colour to flatten the map onto, making it opaque; none keeps the coverage alpha. */
  backdrop: [number, number, number] | null;
  /** How the annotations are included; none for the map alone. */
  annotations: MapExportAnnotations | null;
};

/** Whether this machine can record a viewport, and the reason when it cannot (no Media Foundation, or no audio). */
export type RecordingAvailability = {
  isAvailable: boolean;
  reason: string | null;
};

/**
 * A written recording: its file, duration in seconds, frame count, dropped frames and sound blocks, size, and the
 * H.264 encoder's name.
 */
export type RecordingOutcome = {
  file: string;
  duration: number | null;
  frames: number;
  droppedFrames: number;
  droppedSound: number;
  width: number;
  height: number;
  encoder: string;
};

/**
 * A running recording's progress, sent a few times a second: seconds written since its first frame, frames, and the
 * frames and sound blocks dropped when the recorder fell behind. `ended` gives the reason when it ends by itself.
 */
export type RecordingProgress = {
  elapsed: number | null;
  frames: number;
  droppedFrames: number;
  droppedSound: number;
  ended: string | null;
};

/** Every `tool` the `SceneBinaryToolDescription` union is told apart by, so a switch or a comparison names one. */
export enum ESceneBinaryToolDescription {
  /** `ai_map.part`: the nodes the AI walks. */
  AI_MAP = "ai_map",
  /** `detail_object.part`: the detail objects it plants, and the grid slots planting them. */
  DETAIL_OBJECT = "detail_object",
  /** `wallmark.part`: the decals, in slots of one shader and texture each. */
  WALLMARK = "wallmark",
}

/**
 * Counts of what one binary tool of a scene holds; the payload stays in the backend. Tagged `tool` with the tool's
 * part file name.
 */
export type SceneBinaryToolDescription =
  /** `ai_map.part`: the nodes the AI walks. */
  | { tool: "ai_map"; nodes: number }
  /** `detail_object.part`: the detail objects it plants, and the grid slots planting them. */
  | { tool: "detail_object"; objects: number; slots: number }
  /** `wallmark.part`: the decals, in slots of one shader and texture each. */
  | { tool: "wallmark"; slots: number; marks: number };

/** What a scene's binary tools hold, counted, and why any of them did not read. */
export type SceneBinaryToolsDescription = {
  /** The binary tools the scene holds a part of that read. */
  tools: Array<SceneBinaryToolDescription>;
  /** The binary tools whose parts did not read, each left out of [`Self::tools`]. */
  problems: Array<SceneProblem>;
};

/** What the scene's `[build_params]` set the compilers to weld, smooth and bake with; the reserved fields left out. */
export type SceneBuildParamsDescription = {
  /** Degrees between faces' normals that still smooth into one. */
  smoothAngle: number | null;
  /** Metres within which the compiler welds vertices. */
  weldDistance: number | null;
  lightPixelPerMeter: number | null;
  lightJitterSamples: number;
  lightQuality: number;
};

/** Where the Level Editor's camera stood when the scene was saved, in the scene's own space. */
export type SceneCameraDescription = {
  position: Vector3d;
  /** Heading, pitch and bank, in radians. */
  rotation: Vector3d;
};

/**
 * What the page needs of a drawn scene beside its picture: the extent to frame a camera on, and each scene object it
 * does not draw.
 */
export type SceneDrawDescription = {
  /** The extent of the drawn objects' placed mesh boxes, in renderer space; none where the scene draws no object. */
  bounds: VisualBounds | null;
  /** Each scene object not drawn, in scene object order. */
  problems: Array<SceneDrawProblem>;
};

/** One scene object the scene does not draw, and why. */
export type SceneDrawProblem = {
  /** The scene object, as the scene's objects name it. */
  object: SceneObjectId;
  /** The library object it places, a path under the objects roots without its extension. */
  reference: string;
  kind: SceneDrawProblemKind;
  message: string;
};

/** Why a scene object is not drawn. */
export enum ESceneDrawProblemKind {
  /** Its reference names no `.object` in the scene's objects roots. */
  MISSING = "missing",
  /** Its `.object` was found and does not read. */
  UNREADABLE = "unreadable",
}

/** Every `ESceneDrawProblemKind` as the spelling it crosses IPC as, for a value no member has narrowed. */
export type SceneDrawProblemKind = `${ESceneDrawProblemKind}`;

/**
 * The game data a scene's textures and shaders are found in, given as a compiled level's is. A scene with no game data
 * still opens and draws, its textures missing.
 */
export type SceneGameData = {
  /** Trees the scene's textures and shaders are searched in, in search order; none for a scene with no game data. */
  roots: XrayRoots;
  /** Whether the game's configs resolve with the Monolith/Anomaly DLTX patch dialect. */
  isDltx: boolean;
  /** Which engine the game's configs are read as; detected on `auto`. */
  engine: XrayEngineChoice;
};

/** How many of one helped tool's objects draw a helper in a scene. */
export type SceneHelperToolCount = {
  tool: SceneToolKind;
  count: number;
};

/**
 * One helped tool's row of the scene's helper table: how the Show action names, colours, pictures and switches its
 * helpers.
 */
export type SceneHelperToolEntry = {
  tool: SceneToolKind;
  /**
   * The bit of `RenderHelperOptions::kinds` switching its helpers: the tool's place in the editor's tool order, which
   * also leads each of its helpers' ids.
   */
  kind: number;
  /** Its helpers' colour, `#rrggbb` in sRGB. */
  color: string;
  /** The icon its objects without a shape draw; none for a tool whose every object has a shape. */
  icon: RenderHelperIcon | null;
  /** Metres from the camera past which its helpers fade out. */
  drawDistance: number | null;
  /** Whether its helpers draw until switched off. */
  isHelpedByDefault: boolean;
};

/**
 * What the page needs of a scene's helpers beside their picture: their extent, to frame a scene that draws only
 * helpers, each helped tool's count, and the spawn points drawn as models in place of an icon.
 */
export type SceneHelpersDescription = {
  /**
   * The extent of every helper's shapes and icons and of each spawn model's position, in renderer space; none where
   * the scene draws no helper.
   */
  bounds: VisualBounds | null;
  /** Each helped tool helping at least one object, in the editor's tool order; spawn points drawn as models count. */
  tools: Array<SceneHelperToolCount>;
  /** Each spawn point drawn as its entity's model, in spawn point order. */
  spawnModels: Array<SceneSpawnModelDescription>;
};

/** Every `kind` the `SceneLightType` union is told apart by, so a switch or a comparison names one. */
export enum ESceneLightType {
  /** `ltPoint`, the only kind the build exports. */
  POINT = "point",
  /** `ltSpot`. */
  SPOT = "spot",
  /** `ltDirect`. */
  DIRECTIONAL = "directional",
  /** A type the editor gives no name, kept as the number the part saves. */
  UNKNOWN = "unknown",
}

/** What kind of light a light is, `ELight::EType`. */
export type SceneLightType =
  /** `ltPoint`, the only kind the build exports. */
  | { kind: "point" }
  /** `ltSpot`. */
  | { kind: "spot" }
  /** `ltDirect`. */
  | { kind: "directional" }
  /** A type the editor gives no name, kept as the number the part saves. */
  | { kind: "unknown"; value: number };

/** One object of a scene's object tool, as the outliner lists it and an inspector shows it. */
export type SceneObjectDescription = {
  id: SceneObjectId;
  /** Unique within the scene. */
  name: string;
  /** The name of the group it belongs to; none for an object of its tool's own part. */
  group: string | null;
  /** `co_flags`: motion, grouping and the editor's other states. */
  flags: number;
  transform: SceneObjectTransform;
  properties: SceneObjectProperties;
};

/**
 * Names one scene object until its scene is read again: its tool, and its index in the tool's objects in editor load
 * order, group members first.
 */
export type SceneObjectId = {
  tool: SceneToolKind;
  index: number;
};

/** Every `kind` the `SceneObjectProperties` union is told apart by, so a switch or a comparison names one. */
export enum ESceneObjectProperties {
  /** A library object placed, `scene_object.part`. */
  SCENE_OBJECT = "sceneObject",
  /** The meshes the renderer culls together, `sector.part`. */
  SECTOR = "sector",
  /** A polygon one sector is seen through from another, `portal.part`. */
  PORTAL = "portal",
  /** A light, `light.part`. */
  LIGHT = "light",
  /** A light flare, `glow.part`. */
  GLOW = "glow",
  /** A spawn point, `spawn.part`, with the shape attached where its entity needs a volume. */
  SPAWN = "spawn",
  /** A shape the shape tool places, `shape.part`. */
  SHAPE = "shape",
  /** A patrol path, `way.part`. */
  WAY = "way",
  /** A particle effect planted, `ps.part`. */
  PARTICLE = "particle",
  /** A sound planted, `sound_src.part`. */
  SOUND_SOURCE = "soundSource",
  /** A volume of reverb, `sound_env.part`: its first box splits the inner environment from the outer. */
  SOUND_ENVIRONMENT = "soundEnvironment",
  /** A body of volumetric fog, `fog_volume.part`. */
  FOG_VOLUME = "fogVolume",
  /** A group, `group.part`, whose members are described under their own tools. */
  GROUP = "group",
}

/**
 * What one scene object's kind keeps beyond its name and transform, as an inspector shows it; one variant per object
 * tool.
 */
export type SceneObjectProperties =
  /** A library object placed, `scene_object.part`. */
  | {
      kind: "sceneObject";
      /** The `.object` it places, a path below the objects roots without its extension. */
      reference: string;
      flags: number;
      /** Whether its `.object` was found in the objects roots the scene was opened with. */
      isFound: boolean;
    }
  /** The meshes the renderer culls together, `sector.part`. */
  | {
      kind: "sector";
      /** The colour the editor draws it in: red, green, blue and alpha bytes. */
      color: [number, number, number, number];
      /** Whether it is the sector everything no other sector holds falls into. */
      isDefault: boolean;
      /** The level map index it switches to; none for one switching to none. */
      mapIndex: number | null;
      items: Array<SceneSectorItem>;
    }
  /** A polygon one sector is seen through from another, `portal.part`. */
  | {
      kind: "portal";
      sectorFront: string;
      sectorBack: string;
      /** Its corners in world space, in order. */
      vertices: Array<Vector3d>;
    }
  /** A light, `light.part`. */
  | {
      kind: "light";
      lightType: SceneLightType;
      /** Red, green, blue and alpha, which light at `brightness` times. */
      color: [number | null, number | null, number | null, number | null];
      brightness: number | null;
      range: number | null;
      attenuation: [number | null, number | null, number | null];
      /** A spot's cone angle, in radians. */
      cone: number | null;
      virtualSize: number | null;
      isUsedInD3d: boolean;
      /** Whether it bakes, lights at run time, and the editor's other bits, the fuzzy one aside. */
      flags: number;
      /** The light control it answers to, by its index in the light tool's controls. */
      control: number;
      /** The light animation it plays, empty for none. */
      animation: string;
      falloffTexture: string;
      /** The points a fuzzy point light spreads over, in its own space; empty for any other light. */
      fuzzyPoints: Array<Vector3d>;
    }
  /** A light flare, `glow.part`. */
  | { kind: "glow"; radius: number | null; shader: string; texture: string; flags: number }
  /** A spawn point, `spawn.part`, with the shape attached where its entity needs a volume. */
  | { kind: "spawn"; spawn: SceneSpawnKind; attached: SceneShape | null }
  /** A shape the shape tool places, `shape.part`. */
  | { kind: "shape"; shape: SceneShape }
  /** A patrol path, `way.part`. */
  | { kind: "way"; wayType: SceneWayType; points: Array<SceneWayPoint> }
  /** A particle effect planted, `ps.part`. */
  | {
      kind: "particle";
      /** The effect it plays, a definition of `particles.xr`. */
      reference: string;
      /** The game modes it plays in, one bit each. */
      gameTypes: number;
    }
  /** A sound planted, `sound_src.part`. */
  | {
      kind: "soundSource";
      /** The sound it plays, a path below the game's sounds without its extension. */
      sound: string;
      flags: number;
      /** Where the sound stands, which its transform does not move. */
      position: Vector3d;
      volume: number | null;
      frequency: number | null;
      minDistance: number | null;
      maxDistance: number | null;
      maxAiDistance: number | null;
      /** The pause between playings, in seconds. */
      randomPause: Vector2d;
      /** The window of the game's day it plays in, in seconds from midnight. */
      activeTime: Vector2d;
      /** How long one playing lasts, in seconds. */
      playTime: Vector2d;
    }
  /** A volume of reverb, `sound_env.part`: its first box splits the inner environment from the outer. */
  | { kind: "soundEnvironment"; shape: SceneShape; inner: string; outer: string }
  /** A body of volumetric fog, `fog_volume.part`. */
  | {
      kind: "fogVolume";
      shape: SceneShape;
      /** The emitter's profile; none for an occlusion volume. */
      profile: string | null;
      /** Which volumes export together; none for one never grouped. */
      group: number | null;
    }
  /** A group, `group.part`, whose members are described under their own tools. */
  | {
      kind: "group";
      /** How many objects of the described tools name it as their group. */
      members: number;
    };

/** One object tool of a scene and its objects in editor load order: group members of its class, then its own part's. */
export type SceneObjectToolDescription = {
  tool: SceneToolKind;
  objects: Array<SceneObjectDescription>;
};

/** Where a scene object stands, as the editor keeps it, in the scene's own space. */
export type SceneObjectTransform = {
  position: Vector3d;
  /** Angles about `x`, `y` and `z`, in radians. */
  rotation: Vector3d;
  scale: Vector3d;
};

/**
 * A scene's object tools' typed objects: every tool with a part or a group member, and why any tool's objects did
 * not read.
 */
export type SceneObjectsDescription = {
  /** The tools whose objects read, in the editor's tool order. */
  tools: Array<SceneObjectToolDescription>;
  /** The tools whose objects did not read, each left out of [`Self::tools`]. */
  problems: Array<SceneProblem>;
};

/**
 * A request to open a Level Editor scene: its `.level`, where its library objects are looked for, and the game data it
 * is drawn with.
 */
export type SceneOpenRequest = {
  source: SceneSource;
  /**
   * Directories the scene objects' `.object` files are looked for in, in search order; `objects` beside the directory
   * holding the `.level` where none are given.
   */
  objectsRoots: Array<string> | null;
  /**
   * Trees the scene's textures and shaders are searched in; where none are given, `gamedata` beside the `rawdata`
   * holding the scene's `levels` where it exists, else no game data.
   */
  gameRoots: XrayRoots | null;
  /** Whether the game's configs resolve with the Monolith/Anomaly DLTX patch dialect. */
  isDltx: boolean;
  /** Which engine the game's configs are read as; detected on `auto`. */
  engine: XrayEngineChoice;
};

/** One part file a tool has beside its scene, whether the editor loads it or skips it. */
export type ScenePartFileDescription = {
  /** `<tool>.part`, or a numbered one after it. */
  name: string;
  /** Its size in bytes. */
  size: number;
  /** The GUID of the scene it was saved for, `g0:g1`; none for a file that does not read as a part. */
  guid: string | null;
};

/**
 * Something of a scene the Level Editor would skip or refuse, found by the open or a part's read and kept on its tool;
 * never a refused open.
 */
export type SceneProblem = {
  kind: SceneProblemKind;
  /** The tool it belongs to, by the name its part files carry: `scene_object`, `spawn`, `ai_map` and the rest. */
  tool: string;
  /**
   * The part file it was found in, `<tool>.part` or a numbered one; none where the tool's objects do not read, its
   * group members or its own part as a whole.
   */
  file: string | null;
  message: string;
};

/**
 * Why the Level Editor would skip or refuse one of a scene's parts while still opening the scene. A part carrying
 * another scene's GUID is none of these: it refuses the scene.
 */
export enum ESceneProblemKind {
  /** The file does not read as a part, or its content does not read as its tool's. */
  UNREADABLE = "unreadable",
  /** The part's `[main]` holds a tool version other than the one its tool reads. */
  WRONG_VERSION = "wrongVersion",
  /** The part is saved as another tool's class, or holds an object of another class than its tool's. */
  FOREIGN_CLASS = "foreignClass",
}

/** Every `ESceneProblemKind` as the spelling it crosses IPC as, for a value no member has narrowed. */
export type SceneProblemKind = `${ESceneProblemKind}`;

/** One mesh a sector holds: a scene object by name, and one of its object's meshes by name. */
export type SceneSectorItem = {
  object: string;
  mesh: string;
};

/** The volume a shape encloses: its spheres and boxes, and what the shape is used for. */
export type SceneShape = {
  usage: SceneShapeUsage;
  elements: Array<SceneShapeElement>;
};

/** Every `kind` the `SceneShapeElement` union is told apart by, so a switch or a comparison names one. */
export enum ESceneShapeElement {
  SPHERE = "sphere",
  /** The unit box, `-0.5` to `0.5` on each axis, placed by its axes and translation. */
  BOX = "box",
}

/** One element of a shape, in its object's own space. */
export type SceneShapeElement =
  | { kind: "sphere"; center: Vector3d; radius: number | null }
  /** The unit box, `-0.5` to `0.5` on each axis, placed by its axes and translation. */
  | { kind: "box"; xAxis: Vector3d; yAxis: Vector3d; zAxis: Vector3d; translation: Vector3d };

/** Every `kind` the `SceneShapeUsage` union is told apart by, so a switch or a comparison names one. */
export enum ESceneShapeUsage {
  /** `eShapeCommon`. */
  COMMON = "common",
  /** `eShapeLevelBound`: the box the build takes for the level's map. */
  LEVEL_BOUND = "levelBound",
  /** A usage the editor gives no name, kept as the number the part saves. */
  UNKNOWN = "unknown",
}

/** What a shape is used for, `eShapeUsage`. */
export type SceneShapeUsage =
  /** `eShapeCommon`. */
  | { kind: "common" }
  /** `eShapeLevelBound`: the box the build takes for the level's map. */
  | { kind: "levelBound" }
  /** A usage the editor gives no name, kept as the number the part saves. */
  | { kind: "unknown"; value: number };

/**
 * A Level Editor scene's `.level` on disk. The scene is searched from the directory holding it, its parts in the
 * directory beside it named after it, so a scene opens wherever it lies.
 */
export type SceneSource = {
  /** The `.level` file's filesystem path. */
  path: string;
};

/**
 * One field of a spawned entity's data as the scene saves it in `[object_N_spawndata]`: the packet's values numbered
 * in write order, custom data among them.
 */
export type SceneSpawnField = {
  key: string;
  /** The value as the scene stores it. */
  value: string;
};

/** Every `kind` the `SceneSpawnKind` union is told apart by, so a switch or a comparison names one. */
export enum ESceneSpawnKind {
  /** Where a multiplayer team, artefact or item respawns. */
  RESPAWN_POINT = "respawnPoint",
  /** A local override of the weather. */
  ENVIRONMENT_MODIFIER = "environmentModifier",
  /** A server entity. */
  ENTITY = "entity",
}

/** What a spawn point places, with what its kind keeps. */
export type SceneSpawnKind =
  /** Where a multiplayer team, artefact or item respawns. */
  | {
      kind: "respawnPoint";
      team: number;
      /** An actor, artefact or item spawn. */
      pointType: number;
      /** The game modes it answers to, one bit each. */
      gameTypes: number;
      /** The item preset an item spawn names. */
      profile: string;
    }
  /** A local override of the weather. */
  | {
      kind: "environmentModifier";
      radius: number | null;
      power: number | null;
      viewDistance: number | null;
      /** Colours as packed `0xAARRGGBB` words. */
      fogColor: number;
      fogDensity: number | null;
      ambientColor: number;
      skyColor: number;
      hemiColor: number;
      /** Which values the engine mixes in, one bit each. */
      flags: number;
    }
  /** A server entity. */
  | {
      kind: "entity";
      /** The section the entity is created from. */
      section: string;
      /** Whether the entity respawns in multiplayer, `fl`. */
      flags: number;
      /** Its packet's fields in the order the scene saves them. */
      data: Array<SceneSpawnField>;
    };

/**
 * A spawn point drawn as its entity's model: the object, and the renderer's name for the model, under which a model
 * the renderer could not read is reported.
 */
export type SceneSpawnModelDescription = {
  object: SceneObjectId;
  model: string;
};

/** One tool a scene holds part files of, as its open listed them. */
export type SceneToolDescription = {
  /** The tool's name, which names its files: `scene_object`, `spawn`, `ai_map` and the rest. */
  name: string;
  /** The editor's tool of that name; none for a tool the editor does not save. */
  kind: SceneToolKind | null;
  /** How the tool saves its part; none for a tool the editor does not know whose files do not read. */
  format: SceneToolFormat | null;
  /** Its files' sizes summed, in bytes. */
  size: number;
  /** Its files in the editor's read order, including those it skips. */
  files: Array<ScenePartFileDescription>;
};

/** How a tool saves its part: as text holding one section per object, or as chunks holding the tool's own stream. */
export enum ESceneToolFormat {
  /** An object tool's text part: scene objects, sectors, spawn points, groups and the rest. */
  OBJECT = "object",
  /** A binary part: the AI map, the detail objects and the wall marks. */
  BINARY = "binary",
}

/** Every `ESceneToolFormat` as the spelling it crosses IPC as, for a value no member has narrowed. */
export type SceneToolFormat = `${ESceneToolFormat}`;

/**
 * A Level Editor tool, by the name its part files carry, and what the editor requires of its part before loading it.
 *
 * Declared in the editor's class id order (`ObjClassID`), the order it keeps and lists its tools in. Each crosses IPC
 * as its part file name, written per variant because no case convention spells `ps` or `sound_src`.
 */
export enum ESceneToolKind {
  /** Groups of other tools' objects, `OBJCLASS_GROUP`. */
  GROUP = "group",
  /** Light flares, `OBJCLASS_GLOW`. */
  GLOW = "glow",
  /** Library objects placed, `OBJCLASS_SCENEOBJECT`. */
  SCENE_OBJECT = "scene_object",
  /** Lights, `OBJCLASS_LIGHT`. */
  LIGHT = "light",
  /** Shapes, `OBJCLASS_SHAPE`. */
  SHAPE = "shape",
  /** Sounds planted, `OBJCLASS_SOUND_SRC`. */
  SOUND_SOURCE = "sound_src",
  /** Spawn points, `OBJCLASS_SPAWNPOINT`. */
  SPAWN = "spawn",
  /** Patrol paths, `OBJCLASS_WAY`. */
  WAY = "way",
  /** Sectors, `OBJCLASS_SECTOR`. */
  SECTOR = "sector",
  /** Portals, `OBJCLASS_PORTAL`. */
  PORTAL = "portal",
  /** Reverb volumes, `OBJCLASS_SOUND_ENV`. */
  SOUND_ENVIRONMENT = "sound_env",
  /** Particle effects planted, `OBJCLASS_PS`. */
  PARTICLE = "ps",
  /** Detail objects, `OBJCLASS_DO`. */
  DETAIL_OBJECT = "detail_object",
  /** The AI map, `OBJCLASS_AIMAP`. */
  AI_MAP = "ai_map",
  /** Wall marks, `OBJCLASS_WM`. */
  WALLMARK = "wallmark",
  /** Fog volumes, `OBJCLASS_FOG_VOL`. */
  FOG_VOLUME = "fog_volume",
}

/** Every `ESceneToolKind` as the spelling it crosses IPC as, for a value no member has narrowed. */
export type SceneToolKind = `${ESceneToolKind}`;

/** A link from one way point to another of its way. */
export type SceneWayLink = {
  /** The point it leads to, by its place in the way's points. */
  to: number;
  probability: number | null;
};

/** One point of a way, with the links leading from it. */
export type SceneWayPoint = {
  name: string;
  position: Vector3d;
  /** Thirty-two flags the scripts give meaning to. */
  flags: number;
  /** Its links in the order the editor keeps them. */
  links: Array<SceneWayLink>;
};

/** Every `kind` the `SceneWayType` union is told apart by, so a switch or a comparison names one. */
export enum ESceneWayType {
  /** `wtPatrolPath`, every way the editor makes. */
  PATROL_PATH = "patrolPath",
  /** A type the editor gives no name, kept as the number the part saves. */
  UNKNOWN = "unknown",
}

/** What a way is, `EWayType`. */
export type SceneWayType =
  /** `wtPatrolPath`, every way the editor makes. */
  | { kind: "patrolPath" }
  /** A type the editor gives no name, kept as the number the part saves. */
  | { kind: "unknown"; value: number };

/**
 * A screenshot asked of a viewport, drawn as `screenshot` asks, and the PNG of the level viewer's screenshots folder
 * it is written to.
 */
export type ScreenshotSaveRequest = {
  viewport: RenderViewportId;
  path: string;
  screenshot: RenderScreenshot;
};

/**
 * What the viewer is showing, paired with where it came from: the level itself, its root. What is read of it from
 * other files - where it starts, its map, its surfaces and their textures - each has a query of its own
 * (`describe_start`, `describe_level_map`, `describe_surfaces`, `describe_textures`), read again apart from it.
 */
export type SelectedLevelDescription = {
  source: LevelSource;
  /** The roots the level and its textures were resolved in, centred on the level itself. */
  roots: XrayRoots;
  /** The engine everything engine-dependent about the level follows, and what decided it. */
  engine: XrayEngineResolution;
  /** The compiler version the header records, which decides how the level's chunks are laid out. */
  xrlcVersion: number;
  /** The quality the compiler built the level at, as its header records it. */
  xrlcQuality: number;
  /** Visuals the level declares, the hierarchy's inner nodes with them. */
  visuals: number;
  /** Visuals that draw something themselves, the hierarchy's leaves. */
  drawables: number;
  /** Entries of the shader table, each a shader and the textures it binds. */
  shaderEntries: number;
  /** Portals joining the level's sectors. */
  portals: number;
  /** Lights the level places, its sun among them. */
  lights: number;
  /**
   * The directional light the level names its sun, which is what its baked occlusion was computed for; none for a
   * level naming no sun.
   */
  sun: LevelSunDescription | null;
  /** What each sector is and where, as the visuals declare them. */
  sectors: Array<SectorOutline>;
  /** Extent every sector together covers, which is where a camera is framed from. */
  bounds: VisualBounds | null;
};

/**
 * What the page shows of an open scene itself: its names, its `.level` settings, its tools' part files and their
 * problems. The tools' objects are parts with queries of their own.
 */
export type SelectedSceneDescription = {
  source: SceneSource;
  /** The `.level`'s name, which names its part directory. */
  name: string;
  /** The scene's GUID, `g0:g1`, which each of its parts carries. */
  guid: string;
  /** Where its scene objects' library objects are looked for. */
  objectsRoots: XrayRoots;
  /** The game data roots its textures and shaders are searched in, if it has any. */
  game: SceneGameData;
  /** What its `[build_params]` set, none where they do not read. */
  buildParams: SceneBuildParamsDescription | null;
  /** Where the editor's camera stood, none where `[camera]` does not read. */
  camera: SceneCameraDescription | null;
  /** Every tool it holds part files of, in the editor's order, then the tools the editor does not save, by name. */
  tools: Array<SceneToolDescription>;
  /** What the open found the editor would skip or refuse of its part files. */
  problems: Array<SceneProblem>;
};

/** A DDS an export writes, as the game loads a map or a cubemap: its format, how hard it is encoded, and its mipmaps. */
export type ViewerDdsExport = {
  format: TextureEncodingFormat;
  quality: TextureEncodingQuality;
  /** Whether it carries its whole mip chain, reduced with the X-Ray converter's Kaiser kernel. */
  isMipmapped: boolean;
};
