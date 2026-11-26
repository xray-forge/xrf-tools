// Auto-generated rust bindings. Do not edit it manually.

import { XrayEngineChoice } from "#/types/xrf-engine-target";
import { JobOutcome } from "#/types/xrf-job";
import { LevelCompileReport } from "#/types/xrf-level-compiler";
import { LevelMapArea, LevelObjectKind, LevelSource } from "#/types/xrf-level-source";
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
import { TextureEncodingFormat, TextureEncodingQuality } from "#/types/xrf-texture";
import { XrayRoots } from "#/types/xrf-vfs";
import { VisualBounds } from "#/types/xrf-visual";

/** Building a Level Editor scene into a level directory. */
export const LEVELS_JOB_BUILD = "levels.build" as const;

/** Capturing a cubemap around a viewport's camera, face by face. */
export const LEVELS_JOB_CAPTURE_CUBE = "levels.capture-cube" as const;

/** Capturing a level map in a viewport, tile by tile. */
export const LEVELS_JOB_CAPTURE_MAP = "levels.capture-map" as const;

/** Decompiling a compiled level into a Level Editor scene. */
export const LEVELS_JOB_DECOMPILE = "levels.decompile" as const;

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

/**
 * How a scene's build, and the compile after it, ended: the build's report once it finished, and the compile's once
 * one was asked for and finished.
 */
export type LevelBuildResult = {
  outcome: JobOutcome;
  report: LevelBuildReport | null;
  compile: LevelCompileReport | null;
};

/** How a level's decompile into a scene ended, with its report once it completed. */
export type LevelDecompileResult = {
  outcome: JobOutcome;
  report: LevelDecompileReport | null;
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
