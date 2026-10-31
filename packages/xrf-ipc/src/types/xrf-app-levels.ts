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
import { LevelEnvModifier } from "#/types/xrf-level";
import { Vector3d } from "#/types/xrf-math";
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

/** Capturing a cubemap around a viewport's camera, face by face. */
export const LEVELS_JOB_CAPTURE_CUBE = "levels.capture-cube" as const;

/** Capturing a level map in a viewport, tile by tile. */
export const LEVELS_JOB_CAPTURE_MAP = "levels.capture-map" as const;

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

/** What a change event calls a level's spawn when it goes stale. */
export const LEVEL_SPAWN_PART = "spawn" as const;

/** What a change event calls a level's spawned objects' visuals when they go stale. */
export const LEVEL_SPAWN_VISUALS_PART = "spawnVisuals" as const;

/** What a change event calls where a level opens when it goes stale. */
export const LEVEL_START_PART = "start" as const;

/** What a change event calls how a level's surfaces are drawn when it goes stale. */
export const LEVEL_SURFACES_PART = "surfaces" as const;

/** What a change event calls what a level's surfaces' textures resolved to when it goes stale. */
export const LEVEL_TEXTURES_PART = "textures" as const;

/** What a change event calls what a level's weather plays with beyond its environment when it goes stale. */
export const LEVEL_WEATHER_PART = "weather" as const;

/** A burst asked of a viewport, and the name of the folder of the level viewer's bursts it is written into. */
export type BurstCaptureRequest = {
  viewport: RenderViewportId;
  burst: RenderBurst;
  name: string;
};

/**
 * A burst written: its folder, the scene's seconds each frame was drawn at in the order written, and the sizes of
 * its frames and of their marks.
 */
export type BurstOutcome = {
  folder: string;
  times: Array<number | null>;
  width: number;
  height: number;
  marksWidth: number;
  marksHeight: number;
};

/** A cubemap asked of a viewport, and the cross PNG it is written to. */
export type CubeCaptureRequest = {
  viewport: RenderViewportId;
  capture: RenderCubeCapture;
  path: string;
};

/**
 * What a cubemap is exported as: one cube DDS, six TGAs named `1` to `6` as `CDemoRecord` writes its faces, or six
 * PNGs named by face.
 */
export type CubeExportFormat =
  | { kind: "png" }
  | { kind: "tga" }
  | ({
      kind: "dds";
    } & ViewerDdsExport);

/** A cubemap's capture written again in another shape. */
export type CubeExportRequest = {
  /** The capture's cross PNG in the screenshots folder. */
  path: string;
  /** The DDS written, or the folder six faces are written into. */
  target: string;
  format: CubeExportFormat;
  /** Pixels a face takes on a side, at most the capture's. */
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
   * A static sound (`SStaticSound`): the sound, its volume and frequency, and its windows in milliseconds, each from
   * and to: the game day's hours it plays in, how long each play lasts and the pause between, none for always, its
   * whole length and looped forever.
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
   * A static sound (`SStaticSound`): the sound, its volume and frequency, and its windows in milliseconds, each from
   * and to: the game day's hours it plays in, how long each play lasts and the pause between, none for always, its
   * whole length and looped forever.
   */
  | {
      kind: "sound";
      sound: string;
      volume: number | null;
      frequency: number | null;
      active: [number, number];
      play: [number, number];
      pause: [number, number];
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
 * The annotations asked of a level map: its PNG in the screenshots folder, beside which they are written as an SVG,
 * the ground it covers, the kinds of level object drawn, and whether every object is named.
 */
export type MapAnnotationsWriteRequest = {
  path: string;
  area: LevelMapArea;
  kinds: Array<LevelObjectKind>;
  isLabelled: boolean;
};

/** A level map asked of a viewport, and the PNG it is written to. */
export type MapCaptureRequest = {
  viewport: RenderViewportId;
  capture: RenderMapCapture;
  path: string;
};

/** How a level map's annotations, the SVG beside its capture, go into an export of it. */
export enum EMapExportAnnotations {
  /** Drawn over the map. */
  OVER = "over",
  /** Alone, clear where they draw nothing. */
  ALONE = "alone",
}

/** Every `EMapExportAnnotations` as the spelling it crosses IPC as, for a value no member has narrowed. */
export type MapExportAnnotations = `${EMapExportAnnotations}`;

/**
 * What a level map is exported as: `CDemoRecord`'s own uncompressed TGA, a PNG, a DDS as the game loads a map, or, for
 * its annotations alone, their SVG.
 */
export type MapExportFormat =
  | { kind: "png" }
  | { kind: "tga" }
  | { kind: "svg" }
  | ({
      kind: "dds";
    } & ViewerDdsExport);

/** A level map's capture written again in another shape. */
export type MapExportRequest = {
  /** The capture's PNG in the screenshots folder, its coverage in alpha. */
  path: string;
  target: string;
  format: MapExportFormat;
  /** Pixels the longer side takes, at most the capture's; the shorter follows the capture's shape. */
  longSide: number;
  /** The colour the map is laid over, its alpha made opaque, or none to keep its coverage. */
  backdrop: [number, number, number] | null;
  /** Whether and how its annotations go in, or none for the map alone. */
  annotations: MapExportAnnotations | null;
};

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

/** A DDS an export writes, as the game loads a map or a cubemap: its format, how hard it is encoded, and its mipmaps. */
export type ViewerDdsExport = {
  format: TextureEncodingFormat;
  quality: TextureEncodingQuality;
  /** Whether it carries its whole mip chain, reduced with the X-Ray converter's Kaiser kernel. */
  isMipmapped: boolean;
};
