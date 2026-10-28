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
import { RenderBurst, RenderCubeCapture, RenderMapCapture, RenderViewportId } from "#/types/xrf-renderer";
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

/** What a change event calls what a level's object motions were read from when it goes stale. */
export const LEVEL_OBJECT_MOTIONS_PART = "objectMotions" as const;

/** What a change event calls what a level's particle systems were read from when it goes stale. */
export const LEVEL_PARTICLES_PART = "particles" as const;

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

/** What a level map's annotations draw, each kind on its own switch and in its own colour. */
export enum ELevelAnnotationKind {
  /** The smart terrains' shapes, named. */
  SMART_TERRAINS = "smartTerrains",
  /** The space restrictors' shapes, those keeping the actor in apart from those keeping him out. */
  RESTRICTORS = "restrictors",
  /** The anomalous and torrid zones' shapes. */
  ANOMALIES = "anomalies",
  /** The level changers' shapes, each with an arrow toward the level it leads to and its name. */
  LEVEL_CHANGERS = "levelChangers",
  /** The game graph's vertices on the level, as dots. */
  GRAPH_POINTS = "graphPoints",
  /**
   * The patrol paths starting on the level: their links as lines, their points as dots, named at the first where
   * every object is named.
   */
  PATROLS = "patrols",
}

/** Every `ELevelAnnotationKind` as the spelling it crosses IPC as, for a value no member has narrowed. */
export type LevelAnnotationKind = `${ELevelAnnotationKind}`;

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

/** What a spawned object is, by the engine class its spawn stores: what the viewer groups and toggles it by. */
export enum ELevelSpawnCategory {
  /** Physics objects, breakables, inventory boxes, vehicles, and zones drawn as a model. */
  PROPS = "props",
  /** Every inventory item but a weapon. */
  ITEMS = "items",
  /** Firearms and grenades. */
  WEAPONS = "weapons",
  /** Hanging lamps the engine spawns on R2. */
  LAMPS = "lamps",
}

/** Every `ELevelSpawnCategory` as the spelling it crosses IPC as, for a value no member has narrowed. */
export type LevelSpawnCategory = `${ELevelSpawnCategory}`;

/** One spawned object the viewer draws: what it is, where it stands, and which visual it stands as. */
export type LevelSpawnObject = {
  /** Its place among the level's spawned objects, which names it to the backend. */
  index: number;
  name: string;
  section: string;
  clsid: ClsId;
  /** Absent for an object without one (`INVALID_STORY_ID`). */
  storyId: number | null;
  category: LevelSpawnCategory;
  /** Why a new game releases it, absent for an object it keeps; a released one is drawn only when asked. */
  release: LevelSpawnRelease | null;
  /** The visual, by its index among the description's visuals. */
  visual: number;
  /** The object's `XFORM`, in renderer space. */
  transform: VisualTransform;
};

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

/** The spawned objects the viewer draws, and the visuals they stand as, each named once. */
export type LevelSpawnObjectsDescription = {
  visuals: Array<string>;
  objects: Array<LevelSpawnObject>;
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
 * the ground it covers, what of them is drawn, and whether every volume is named.
 */
export type MapAnnotationsWriteRequest = {
  path: string;
  area: LevelMapArea;
  kinds: Array<LevelAnnotationKind>;
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
