// Auto-generated rust bindings. Do not edit it manually.

import { ArchiveProject, ArchiveReadPolicy } from "@/core/ipc/types/xrf-archive";
import { SpawnHeaderChunk } from "@/core/ipc/types/xrf-db";
import { DialogProjectMode } from "@/core/ipc/types/xrf-dialog";
import { JobOutcome, JobProgress } from "@/core/ipc/types/xrf-job";
import { LtxAnchoredFinding, LtxFileStructure, LtxFileText, LtxInventory } from "@/core/ipc/types/xrf-ltx-inspect";
import { XrayMaterialDescriptor, XraySurfaceDescriptor } from "@/core/ipc/types/xrf-material";
import { ArchivePackConfig, ArchivePatchConfig } from "@/core/ipc/types/xrf-pack";
import { EquipmentSlotOccupant } from "@/core/ipc/types/xrf-texture";
import {
  TranslationBuildLanguageSummary,
  TranslationParseCensus,
  TranslationProjectDescriptor,
  TranslationProjectMode,
  TranslationVerifyLanguageSummary,
} from "@/core/ipc/types/xrf-translation";
import { XrayAsset, XrayAssetContainer, XrayPathCollision, XrayRoots, XraySourceKind } from "@/core/ipc/types/xrf-vfs";
import { VisualDependencies, VisualDescription } from "@/core/ipc/types/xrf-visual";

/** Every `kind` the `ArchiveAnimationBehavior` union is told apart by, so a switch or a comparison names one. */
export enum EArchiveAnimationBehavior {
  /** `BEH_RESET`: falls back to nothing. */
  RESET = "reset",
  /** `BEH_CONSTANT`: holds the value of the nearest key, which is what everything shipped declares. */
  CONSTANT = "constant",
  /** `BEH_REPEAT`: plays the keyed range again from its start. */
  REPEAT = "repeat",
  /** `BEH_OSCILLATE`: plays the keyed range again backwards. */
  OSCILLATE = "oscillate",
  /** `BEH_OFFSET`: repeats, each pass starting where the last ended. */
  OFFSET = "offset",
  /** `BEH_LINEAR`: carries on along the slope the end key leaves. */
  LINEAR = "linear",
  /** A value the engine gives no name, kept as the number the file stores. */
  UNNAMED = "unnamed",
}

/** What a channel does outside its keys, `BEH_*` (`xrCore/Animation/Envelope.hpp`). */
export type ArchiveAnimationBehavior =
  /** `BEH_RESET`: falls back to nothing. */
  | { kind: "reset" }
  /** `BEH_CONSTANT`: holds the value of the nearest key, which is what everything shipped declares. */
  | { kind: "constant" }
  /** `BEH_REPEAT`: plays the keyed range again from its start. */
  | { kind: "repeat" }
  /** `BEH_OSCILLATE`: plays the keyed range again backwards. */
  | { kind: "oscillate" }
  /** `BEH_OFFSET`: repeats, each pass starting where the last ended. */
  | { kind: "offset" }
  /** `BEH_LINEAR`: carries on along the slope the end key leaves. */
  | { kind: "linear" }
  /** A value the engine gives no name, kept as the number the file stores. */
  | { kind: "unnamed"; value: number };

/** One animated channel: what it drives, and the keys that drive it. */
export type ArchiveAnimationChannel = {
  /** What the channel animates, which is its position in the file rather than anything the file names. */
  name: string;
  keys: number;
  /** Seconds its first key sits at, absent for a channel carrying none. */
  firstSeconds: number | null;
  /** Seconds its last key sits at, absent for a channel carrying none. */
  lastSeconds: number | null;
  /** The smallest value any of its keys holds, absent for a channel carrying none. */
  minimum: number | null;
  /** The largest value any of its keys holds, absent for a channel carrying none. */
  maximum: number | null;
  /** What it does before its first key. */
  behaviorBefore: ArchiveAnimationBehavior;
  /** What it does after its last key. */
  behaviorAfter: ArchiveAnimationBehavior;
  /** Curve shapes its keys use, named, each once. */
  shapes: Array<string>;
};

/** Everything the viewer says about one object motion. */
export type ArchiveAnmDescription = {
  /** The name the editor saved it under, absent for the 1,051 of 1,221 shipped animations carrying none. */
  name: string | null;
  version: number;
  frameStart: number;
  frameEnd: number;
  /** Frames the range spans, counting both ends, which is the engine's `Length`. */
  frames: number;
  fps: number | null;
  /** Seconds the engine plays it for, which is what a camera effect's lifetime is taken from. */
  durationSeconds: number | null;
  /** Keys across every channel. */
  keys: number;
  /** Seconds the last key of any channel sits at, absent when nothing is keyed. */
  keyedSeconds: number | null;
  /** One entry per channel, in the order the format stores them. */
  channels: Array<ArchiveAnimationChannel>;
};

/** How much space something covers, as the extent of the box it declares rather than where that box sits. */
export type ArchiveBounds = {
  width: number | null;
  height: number | null;
  depth: number | null;
};

/** One chunk of a container, and whatever its payload turned out to hold. */
export type ArchiveChunkNode = {
  /** Chunk id with the compression flag masked off, which is what a format's own constants compare against. */
  id: number;
  /** Bytes of payload, which the children below account for in full when there are any. */
  size: number;
  /** Whether the id carried `CFS_CompressMark`, in which case the payload stands for data it is not and is not walked. */
  isCompressed: boolean;
  /** Chunks the payload is made of, empty for one holding data rather than a container. */
  children: Array<ArchiveChunkNode>;
};

/** The container a file is, for a file nothing here reads. */
export type ArchiveChunksDescription = {
  /** Chunks in the order the file frames them. */
  chunks: Array<ArchiveChunkNode>;
  /** Every chunk of the tree, the nested ones included. */
  nodes: number;
  /** How far the deepest branch descends. */
  depth: number;
  /** Bytes the file occupies unpacked, which the chunks account for in full. */
  size: number;
};

/** Every `kind` the `ArchiveDescribeRefusal` union is told apart by, so a switch or a comparison names one. */
export enum EArchiveDescribeRefusal {
  /**
   * Nothing describes this format yet, and the file is not a container whose shape could be shown instead. The
   * extension is the authored spelling, empty for a name without one.
   */
  NO_DESCRIBER = "noDescriber",
  /** The entry is larger than the policy admits for a read that holds the payload. */
  TOO_LARGE = "tooLarge",
}

/** Why an entry was not described. */
export type ArchiveDescribeRefusal =
  /**
   * Nothing describes this format yet, and the file is not a container whose shape could be shown instead. The
   * extension is the authored spelling, empty for a name without one.
   */
  | { kind: "noDescriber"; extension: string }
  /** The entry is larger than the policy admits for a read that holds the payload. */
  | { kind: "tooLarge"; size: number; maximum: number };

/** Every `kind` the `ArchiveDescribeScope` union is told apart by, so a switch or a comparison names one. */
export enum EArchiveDescribeScope {
  /** The merged name table of the volumes the explorer has open. */
  VOLUMES = "volumes",
  /** Every source a mounted world searches, which is what the engine would search. */
  WORLD = "world",
}

/** What a description's reference lookups searched. */
export type ArchiveDescribeScope =
  /** The merged name table of the volumes the explorer has open. */
  | { kind: "volumes"; volumes: number }
  /** Every source a mounted world searches, which is what the engine would search. */
  | { kind: "world" };

/** One object of a level's detail library, and how much of the level it is planted across. */
export type ArchiveDetailEntry = {
  /** Position in the library, which is what a slot's six bits address. */
  index: number;
  /** Slot corners planting this object across the whole grid. */
  plantedCorners: number;
  model: ArchiveDetailModel;
};

/** Everything the viewer says about a level's detail layer. */
export type ArchiveDetailLibraryDescription = {
  version: number;
  sizeX: number;
  sizeZ: number;
  /** Cells the grid holds, which is `size_x * size_z`. */
  slots: number;
  /** Cells planting at least one object, which is what says how much of the level is actually dressed. */
  plantedSlots: number;
  /** Ground the grid covers along each axis, in engine units, which are metres. */
  coversX: number | null;
  coversZ: number | null;
  /** The library, in the order the file numbers it. */
  entries: Array<ArchiveDetailEntry>;
};

/** Everything the viewer says about one detail object. */
export type ArchiveDetailModel = {
  /**
   * The blender it draws through, which names a definition inside `shaders.xr` rather than a file, so it crosses as
   * text and not as a reference.
   */
  shader: string;
  /** The texture it draws with, absent when the object names none. */
  texture: ArchiveReference | null;
  minScale: number | null;
  maxScale: number | null;
  /** Whether the renderer sways it in the wind, which is `DO_NO_WAVING` being clear rather than set. */
  isWaving: boolean;
  /** Bits of the flag word no name here claims. */
  unnamedFlags: number;
  vertices: number;
  triangles: number;
  /** The box the mesh occupies as authored, before a slot's own scale applies; absent for a model carrying no mesh. */
  bounds: ArchiveBounds | null;
};

/** Everything the viewer says about one trained evaluation function. */
export type ArchiveEfdDescription = {
  builderVersion: number;
  dataFormat: number;
  /** Which base function this one registers itself as. */
  functionType: number;
  minimumResult: number | null;
  maximumResult: number | null;
  /** Discrete buckets each input is divided into, in the order the function declares them. */
  variableRanges: Array<number>;
  /** The base function each input reads its value from. */
  variableKinds: Array<number>;
  patterns: Array<ArchiveEfdPattern>;
  /** Weights the table holds, which the file never stores and the patterns alone decide. */
  weights: number;
};

/** One term of an evaluation function, as the viewer reads it. */
export type ArchiveEfdPattern = {
  /** Inputs the term reads, by their positions in the function's own variable list. */
  variables: Array<number>;
  /** Weights the term claims, which is the product of its inputs' ranges and a number the file never stores. */
  weights: number | null;
};

/** One described entry, and what the lookups behind it searched. */
export type ArchiveFileDescription = {
  scope: ArchiveDescribeScope;
  format: ArchiveFormatDescription;
};

/** Every `kind` the `ArchiveFormatDescription` union is told apart by, so a switch or a comparison names one. */
export enum EArchiveFormatDescription {
  CHUNKS = "chunks",
  SPAWN = "spawn",
  LEVEL_SPAWN = "levelSpawn",
  ANM = "anm",
  DETAIL = "detail",
  DETAIL_LIBRARY = "detailLibrary",
  LEVEL_ENV_MOD = "levelEnvMod",
  LEVEL_FOG_VOL = "levelFogVol",
  LEVEL_GAME = "levelGame",
  GAME_MTL = "gameMtl",
  LEVEL_HOM = "levelHom",
  LEVEL_GEOM = "levelGeom",
  LIGHT_ANIM = "lightAnim",
  SHADER_COMPILER = "shaderCompiler",
  SOUND_ENVIRONMENT = "soundEnvironment",
  LEVEL_LIGHTS = "levelLights",
  LEVEL_PS_STATIC = "levelPsStatic",
  LEVEL_SND_STATIC = "levelSndStatic",
  LEVEL_SOM = "levelSom",
  LEVEL_WALLMARKS = "levelWallmarks",
  EFD = "efd",
  LEVEL = "level",
  LEVEL_AI = "levelAi",
  LEVEL_COLLISION = "levelCollision",
  OMF = "omf",
  PARTICLES = "particles",
  PPE = "ppe",
  SHADERS = "shaders",
  THM = "thm",
  UNSUPPORTED = "unsupported",
}

/** What the explorer can say about one entry it cannot draw. */
export type ArchiveFormatDescription =
  | { kind: "chunks"; description: ArchiveChunksDescription }
  | { kind: "spawn"; description: ArchiveSpawnDescription }
  | { kind: "levelSpawn"; description: ArchiveLevelSpawnDescription }
  | { kind: "anm"; description: ArchiveAnmDescription }
  | { kind: "detail"; description: ArchiveDetailModel }
  | { kind: "detailLibrary"; description: ArchiveDetailLibraryDescription }
  | { kind: "levelEnvMod"; description: ArchiveLevelEnvModDescription }
  | { kind: "levelFogVol"; description: ArchiveLevelFogVolDescription }
  | { kind: "levelGame"; description: ArchiveLevelGameDescription }
  | { kind: "gameMtl"; description: ArchiveGameMtlDescription }
  | { kind: "levelHom"; description: ArchiveLevelHomDescription }
  | { kind: "levelGeom"; description: ArchiveLevelGeomDescription }
  | { kind: "lightAnim"; description: ArchiveLightAnimDescription }
  | { kind: "shaderCompiler"; description: ArchiveShaderCompilerDescription }
  | { kind: "soundEnvironment"; description: ArchiveSoundEnvironmentDescription }
  | { kind: "levelLights"; description: ArchiveLevelLightsDescription }
  | { kind: "levelPsStatic"; description: ArchiveLevelPsStaticDescription }
  | { kind: "levelSndStatic"; description: ArchiveLevelSndStaticDescription }
  | { kind: "levelSom"; description: ArchiveLevelSomDescription }
  | { kind: "levelWallmarks"; description: ArchiveLevelWallmarksDescription }
  | { kind: "efd"; description: ArchiveEfdDescription }
  | { kind: "level"; description: ArchiveLevelDescription }
  | { kind: "levelAi"; description: ArchiveLevelAiDescription }
  | { kind: "levelCollision"; description: ArchiveLevelCollisionDescription }
  | { kind: "omf"; description: ArchiveOmfDescription }
  | { kind: "particles"; description: ArchiveParticlesDescription }
  | { kind: "ppe"; description: ArchivePpeDescription }
  | { kind: "shaders"; description: ArchiveShadersDescription }
  | { kind: "thm"; description: ArchiveThmDescription }
  | { kind: "unsupported"; reason: ArchiveDescribeRefusal };

/** Everything the viewer says about the game material library. */
export type ArchiveGameMtlDescription = {
  version: number;
  materials: Array<ArchiveGameMtlMaterial>;
  /** Pairings of two materials, which decide what is heard and seen where they meet. */
  pairs: number;
  /** Pairings that declare nothing of their own and take everything from the pairing they name as parent. */
  inheritingPairs: number;
  /** What the pairings declare, counted per property rather than listed. */
  properties: Array<ArchiveGameMtlProperty>;
};

/** One game material, as the viewer reads it. */
export type ArchiveGameMtlMaterial = {
  /** The library's own number, which is what a collision face stores rather than the name. */
  id: number;
  name: string;
  /** Absent where the material declares no description chunk at all. */
  description: string | null;
  /** The flags it sets, named. */
  flags: Array<string>;
  friction: number | null;
  bouncing: number | null;
  /** How freely a bullet passes, where 1 is straight through. */
  shootFactor: number | null;
  /** How freely it is walked through, where below 1 the engine slows the walker down. */
  flotationFactor: number | null;
  /** How fast standing in it costs health, which is what makes a material injurious. */
  injuriousSpeed: number | null;
  /** How much sound it stops, where 1 lets everything through. */
  soundOcclusionFactor: number | null;
};

/** One thing a material pairing can declare, and how many pairings declare it. */
export type ArchiveGameMtlProperty = {
  name: string;
  pairs: number;
};

/** What a level's navigation grid covers, from the 56 bytes that say so. */
export type ArchiveLevelAiDescription = {
  version: number;
  nodes: number;
  /** Spacing between nodes on the ground plane, in engine units. */
  nodeSize: number | null;
  /** Height one node spans, which is what decides whether a step is walkable. */
  nodeHeight: number | null;
  bounds: ArchiveBounds;
  /** Identity the spawn set built against this grid carries as its graph guid. */
  guid: string;
  size: number;
};

/** What the bundle holds, taken over the whole of it. */
export type ArchiveLevelBundle = {
  /** Compiler version the bundle was built by, `hdrLEVEL::XRLC_version`. */
  xrlcVersion: number;
  /** Compiler quality the build ran at, `hdrLEVEL::XRLC_quality`. */
  xrlcQuality: number;
  /** Rows of the shader table, which is what a face addresses by position. */
  surfaces: number;
  /** Distinct shader names the surfaces draw with. */
  shaders: number;
  /** Distinct shader names no open library defines. Zero while there is no library to ask. */
  undefinedShaders: number;
  /** Distinct textures the surfaces bind. */
  textures: number;
  /** Distinct textures the subject being browsed does not hold. */
  absentTextures: number;
  /** The blender library the shader names were asked of, absent when the subject holds none. */
  library: ArchiveReference | null;
  /** Whether the file declares a shader table at all. */
  hasShaderTable: boolean;
};

/**
 * What a level's collision mesh weighs, from the 36 bytes that say so.
 *
 * `level.cform` is not chunked: `CDB` casts the file's leading bytes straight onto a header and streams the mesh
 * behind it. So everything here is the first 36 bytes of a file that reaches 191 MB in Anomaly, and the mesh itself
 * is never touched - which is the only reason describing one is affordable at all.
 */
export type ArchiveLevelCollisionDescription = {
  version: number;
  vertices: number;
  faces: number;
  bounds: ArchiveBounds;
  size: number;
};

/** Everything the viewer says about a compiled level bundle. */
export type ArchiveLevelDescription = {
  bundle: ArchiveLevelBundle;
  surfaces: Array<ArchiveLevelSurface>;
};

/** Every `kind` the `ArchiveLevelEntry` union is told apart by, so a switch or a comparison names one. */
export enum EArchiveLevelEntry {
  SKIPPED = "skipped",
  UNUSABLE = "unusable",
  DRAWN = "drawn",
}

/** What one row of the table holds, in the three shapes the renderer distinguishes. */
export type ArchiveLevelEntry =
  | { kind: "skipped" }
  | { kind: "unusable"; raw: string }
  | { kind: "drawn"; shader: ArchiveLevelShader; textures: Array<ArchiveReference> };

/** Everything the viewer says about a level's local weather overrides. */
export type ArchiveLevelEnvModDescription = {
  version: number;
  modifiers: Array<ArchiveLevelEnvModifier>;
};

/** One local weather override of a level, as the viewer reads it. */
export type ArchiveLevelEnvModifier = {
  /** How far the override reaches, falling off linearly to that edge. */
  radius: number | null;
  /** How much of itself it mixes in at the centre. */
  power: number | null;
  farPlane: number | null;
  fogDensity: number | null;
  /**
   * Which of its values the engine mixes in, named. A file below version `0x0016` carries no flag word and the
   * engine mixes in all of them, which is what `use_flags.one()` does before the read.
   */
  usedParameters: Array<string>;
  /** Whether the file said which values to use, rather than the reader assuming all of them. */
  declaresParameters: boolean;
};

/** Everything the viewer says about a level's volumetric fog. */
export type ArchiveLevelFogVolDescription = {
  version: number;
  volumes: Array<ArchiveLevelFogVolume>;
  /** Obstacles across every body. */
  obstacles: number;
};

/** One volumetric fog body of a level, as the viewer reads it. */
export type ArchiveLevelFogVolume = {
  /**
   * The config its simulation settings come from, which is an LTX under `$game_config$` rather than anything the
   * file itself holds. Absent where the body names none.
   */
  profile: ArchiveReference | null;
  /** Bodies the simulation flows around. */
  obstacles: number;
};

/** Everything the viewer says about a level's game data. */
export type ArchiveLevelGameDescription = {
  /** Respawn points, grouped by what they spawn. */
  spawns: Array<ArchiveLevelGameSpawn>;
  rpoints: number;
  ways: number;
  /** Nodes across every patrol path. */
  wayPoints: number;
  /** Paths carrying no nodes at all, which are a path in name only. */
  emptyWays: number;
};

/** One kind of respawn point a level declares, and how many of them there are. */
export type ArchiveLevelGameSpawn = {
  /** What the points spawn, named where the engine names the kind. */
  label: string | null;
  /** The stored kind, kept because a file may carry one the engine gives no name. */
  kind: number;
  points: number;
  /** Points of this kind naming a spawn preset, which only item points do. */
  profiled: number;
};

/**
 * Everything the viewer says about a level's render geometry.
 *
 * Read down to its shape alone: the file reaches 143 MB and the vertices themselves answer nothing a description
 * asks, so each buffer's payload is stepped over rather than held.
 */
export type ArchiveLevelGeomDescription = {
  /** Whether this is the detail twin, `level.geomX`, which the renderer draws distant geometry from. */
  isDetail: boolean;
  vertexBuffers: number;
  indexBuffers: number;
  vertices: number;
  indices: number;
  /** Triangles the indices draw, taking them as triangle lists. */
  triangles: number;
  /** Meshes that drop detail with distance, which `level.geomX` carries none of. */
  progressiveMeshes: number;
  /** Detail levels across every progressive mesh. */
  detailLevels: number;
  /** The vertex layouts the buffers are built from, grouped. */
  layouts: Array<ArchiveLevelGeomLayout>;
  size: number;
};

/** One vertex layout a level's geometry is built from, and how much is built with it. */
export type ArchiveLevelGeomLayout = {
  /** Bytes one vertex of this layout occupies. */
  stride: number;
  /** Elements the declaration names, which is what the stride is made of. */
  elements: number;
  buffers: number;
  vertices: number;
};

/** Everything the viewer says about a level's occlusion mesh. */
export type ArchiveLevelHomDescription = {
  version: number;
  triangles: number;
  /** How much world the occluders span, absent for a mesh carrying none. */
  bounds: ArchiveBounds | null;
};

/** Everything the viewer says about a level's compiled lights. */
export type ArchiveLevelLightsDescription = {
  lights: number;
  /** Lights the runtime turns into light sources, which are the point ones of the chunk it opens. */
  used: number;
  /** Every chunk, including any the compiler wrote that is not a run of lights. */
  groups: Array<ArchiveLevelLightsGroup>;
  /** How much world the lights stand in, absent for a file carrying none. */
  bounds: ArchiveBounds | null;
};

/** One chunk of a level's compiled light list, as the viewer reads it. */
export type ArchiveLevelLightsGroup = {
  /** The chunk id the compiler wrote the run under. */
  id: number;
  lights: number;
  /** Lights the runtime would make a light source of, which are the point ones. */
  point: number;
  /** Whether the engine opens this chunk at all: `CLight_DB::LoadHemi` takes `fsL_HEADER` and ignores the rest. */
  isReadByEngine: boolean;
  /** Whether the payload was a run of lights at all, rather than something kept verbatim. */
  isLights: boolean;
};

/** Everything the viewer says about the particle effects a level plants. */
export type ArchiveLevelPsStaticDescription = {
  version: number;
  placements: number;
  /** Placements only some multiplayer modes load, which a single-player session never plays. */
  restricted: number;
  /** The effects planted, grouped by name. */
  effects: Array<ArchiveLevelPsStaticEffect>;
};

/** One particle effect a level plants, and how widely. */
export type ArchiveLevelPsStaticEffect = {
  /** The effect played, which names a definition inside `particles.xr` rather than a file, so it crosses as text. */
  name: string;
  placements: number;
  /** Placements of this effect that only some multiplayer modes load, which a single-player session never plays. */
  restricted: number;
};

/** The blender a surface draws with, and whether the subject being browsed defines it. */
export type ArchiveLevelShader = {
  name: string;
  status: ArchiveReferenceStatus;
};

/** Everything the viewer says about the sounds a level plants. */
export type ArchiveLevelSndStaticDescription = {
  /** Sounds that only play inside a window of the day. */
  scheduled: number;
  sounds: Array<ArchiveLevelSndStaticSound>;
};

/** One sound a level plants, as the viewer reads it. */
export type ArchiveLevelSndStaticSound = {
  /** The sound played, which does name a file. Absent where the record names none. */
  sound: ArchiveReference | null;
  volume: number | null;
  frequency: number | null;
  /**
   * Whether the sound only plays inside a window of the day, which four of the 670 shipped ones do. A pair of
   * zeroes is no window at all rather than a window of no length.
   */
  isScheduled: boolean;
  activeFrom: number;
  activeTo: number;
};

/** Everything the viewer says about a level's sound occlusion mesh. */
export type ArchiveLevelSomDescription = {
  version: number;
  triangles: number;
  /** Triangles that occlude from both sides, which the loader turns into a second, reversed face each. */
  twoSided: number;
  /** Faces the sound renderer ends up with, which is more than the triangle count wherever one is two-sided. */
  faces: number;
  /** How much sound the quietest and loudest faces let through, absent for a mesh carrying none. */
  minimumOcclusion: number | null;
  maximumOcclusion: number | null;
  /** How much world the occluders span, absent for a mesh carrying none. */
  bounds: ArchiveBounds | null;
};

/** Everything the viewer says about the objects a level spawns. */
export type ArchiveLevelSpawnDescription = {
  objects: number;
  /** What is spawned, grouped by the section each object is built from. */
  sections: Array<ArchiveLevelSpawnSection>;
  /** How much of the level the objects stand in, absent for a list holding none. */
  bounds: ArchiveBounds | null;
  size: number;
};

/** One config section a level spawns objects from, and how many of them. */
export type ArchiveLevelSpawnSection = {
  /** The config section the objects are built from, which names an LTX section rather than a file. */
  name: string;
  objects: number;
};

/** One row of the level's shader table. */
export type ArchiveLevelSurface = {
  index: number;
  entry: ArchiveLevelEntry;
};

/** One material of a level's baked decals, as the viewer reads it. */
export type ArchiveLevelWallmarkSlot = {
  /** The blender the decals draw through, which names a definition inside `shaders.xr` rather than a file. */
  shader: string;
  /**
   * The texture they draw with, which does name a file. Absent for a slot holding nothing, because the exporter
   * writes no names for one.
   */
  texture: ArchiveReference | null;
  marks: number;
  /** Vertices across the slot's decals, which is what it costs to draw. */
  vertices: number;
};

/**
 * Everything the viewer says about a level's baked decals.
 *
 * Authored by the level editor and read by nothing in the runtime, which places its own wallmarks at play time.
 */
export type ArchiveLevelWallmarksDescription = {
  slots: Array<ArchiveLevelWallmarkSlot>;
  marks: number;
  /** Vertices across every decal, which is what the layer costs to draw. */
  vertices: number;
};

/** Everything the viewer says about the colour animation library. */
export type ArchiveLightAnimDescription = {
  version: number;
  /** Whether the colours are stored channel-swapped, which the engine corrects as it loads them. */
  isBgr: boolean;
  items: Array<ArchiveLightAnimItem>;
  /** Keys across every animation. */
  keys: number;
};

/** One colour animation, as the viewer reads it. */
export type ArchiveLightAnimItem = {
  /** The name a light, a glow or a particle effect reaches this animation by. */
  name: string;
  fps: number | null;
  frames: number;
  /** How long it runs, absent for an animation whose rate never advances it. */
  durationSeconds: number | null;
  keys: number;
};

/** What a bank holds, taken over the whole of it. */
export type ArchiveOmfBank = {
  version: number;
  /** Whether the version carries motion marks at all, which version 3 does not. */
  carriesMarks: boolean;
  motions: number;
  /** Motions flagged `esmFX`, which play on a bone rather than on a part of the partition. */
  effects: number;
  bones: number;
  frames: number;
  /** Seconds every motion together spans at the format's fixed sample rate, before playback speed applies. */
  durationSeconds: number | null;
  /** Motions whose payload still carries a name that is not theirs. */
  divergingLabels: number;
  /** Motions carrying at least one mark. */
  markedMotions: number;
  /** Motions whose declared falloff the engine replaces on load. */
  replacedFalloffs: number;
};

/** How a motion blends in and out, in the values the engine blends with. */
export type ArchiveOmfBlend = {
  /** What the engine blends in over, `1.5 × Dequantize(accrue)`. */
  accrue: number | null;
  /** What the engine blends out over, after the rewrite. */
  falloff: number | null;
  declaredAccrue: number | null;
  declaredFalloff: number | null;
  /** Whether the engine replaced the declared falloff rather than reading it. */
  isFalloffReplaced: boolean;
};

/** Everything the viewer says about one motion bank. */
export type ArchiveOmfDescription = {
  bank: ArchiveOmfBank;
  parts: Array<ArchiveOmfPart>;
  /** Every motion, in the order the bank declares them, which is also the order the engine pairs them by. */
  motions: Array<ArchiveOmfMotion>;
};

/** One named set of moments within a motion, `motion_marks`. */
export type ArchiveOmfMark = {
  name: string;
  /** The moments the mark covers, in the order the file lists them. */
  intervals: Array<ArchiveOmfMarkInterval>;
};

/** One stretch of a motion a mark covers, in seconds from its start. */
export type ArchiveOmfMarkInterval = {
  from: number | null;
  to: number | null;
};

/** One motion of a bank: what it is called, how long it is, and how the engine plays it. */
export type ArchiveOmfMotion = {
  /** The name the engine resolves the motion by, which is the definition's and never the payload's label. */
  name: string;
  frames: number;
  /** Seconds the frames span at the format's fixed 30 fps, before playback speed applies. */
  durationSeconds: number | null;
  /** Seconds playing it actually takes, or `None` when the engine reads a speed of zero and the division has no answer. */
  playbackSeconds: number | null;
  speed: ArchiveOmfQuantized;
  power: ArchiveOmfQuantized;
  blend: ArchiveOmfBlend;
  target: ArchiveOmfTarget;
  /** The named bits the definition's word carries, in bit order; only the set ones. */
  flags: Array<string>;
  /** Bits of the word no name here claims. */
  unnamedFlags: number;
  marks: Array<ArchiveOmfMark>;
  /** Whether the payload still carries the name of the motion it holds. */
  hasDivergingLabel: boolean;
};

/** One part of a bank's partition, and what plays on it. */
export type ArchiveOmfPart = {
  name: string;
  /** Bones the part drives, in the order it lists them. */
  bones: Array<string>;
  /** Cycles this bank routes to the part, which is what makes it more than a name. */
  cycles: number;
};

/** A playback value as the engine reads it, beside the one the file stores. */
export type ArchiveOmfQuantized = {
  /** What the engine reads, after the quantizer. */
  value: number | null;
  /** The float the file stores. */
  declared: number | null;
  /** Whether the quantizer's range refused the declared value. */
  isClamped: boolean;
};

/** Every `kind` the `ArchiveOmfTarget` union is told apart by, so a switch or a comparison names one. */
export enum EArchiveOmfTarget {
  PART = "part",
  BONE = "bone",
  UNNAMED = "unnamed",
}

/** What a motion plays on. */
export type ArchiveOmfTarget =
  | { kind: "part"; index: number; name: string | null }
  | { kind: "bone"; index: number; name: string | null }
  | { kind: "unnamed" };

/** What one fold of a subject onto engine identities found: the copies a patch buried, and the copies nothing reaches. */
export type ArchiveOverrideReport = {
  /** Engine paths this subject answers with more than one copy, winner first, ordered by path. */
  overridden: Array<ArchiveWorldEntry>;
  /** Copies no lookup reaches, because another entry of the same source already claims their engine path. */
  unreachable: Array<XrayPathCollision>;
};

/** Everything the viewer says about the particle library. */
export type ArchiveParticlesDescription = {
  library: ArchiveParticlesLibrary;
  effects: Array<ArchiveParticlesEffect>;
  groups: Array<ArchiveParticlesGroup>;
};

/** One emitter of the library: what it draws with, how much of it, and what moves it. */
export type ArchiveParticlesEffect = {
  name: string;
  /** Particles the emitter may hold at once, `max_particles`. */
  maxParticles: number;
  /** Seconds the emitter runs for, absent where it declares no limit and runs until something stops it. */
  timeLimit: number | null;
  /** The blender the sprite is drawn with, which names a definition of `shaders.xr` rather than a file. */
  shader: string;
  /** The texture the sprite is drawn from, resolved against the subject being browsed. */
  texture: ArchiveReference;
  /** Every action kind the effect is built from, in the order it first uses each. */
  actions: Array<string>;
  /** Actions in total, which is more than the kinds above when one kind is used twice. */
  actionsCount: number;
  flags: number;
};

/** An effect a group names, and whether this library is where it is defined. */
export type ArchiveParticlesEffectName = {
  name: string;
  isDefined: boolean;
};

/** One sequence of the library: the effects it plays and what each one starts alongside itself. */
export type ArchiveParticlesGroup = {
  name: string;
  /** Seconds the group runs for; zero where it declares no limit. */
  timeLimit: number | null;
  effects: Array<ArchiveParticlesGroupEffect>;
};

/** One slot of a group: the effect it plays, and the effects that effect starts with it. */
export type ArchiveParticlesGroupEffect = {
  effect: ArchiveParticlesEffectName;
  onBirth: ArchiveParticlesEffectName | null;
  onPlay: ArchiveParticlesEffectName | null;
  onDead: ArchiveParticlesEffectName | null;
  /** Seconds into the group the slot starts and stops, `time_0` and `time_1`. */
  from: number | null;
  to: number | null;
  flags: number;
};

/** What the library holds, taken over the whole of it. */
export type ArchiveParticlesLibrary = {
  version: number;
  effects: number;
  groups: number;
  /** Actions across every effect. */
  actions: number;
  /**
   * Distinct textures the effects draw from, which is far fewer than the effects naming them - 125 against 921 in
   * vanilla.
   */
  textures: number;
  /** Distinct textures the subject being browsed does not hold. */
  absentTextures: number;
  /** Effect names the groups use that this library does not define. */
  undefinedEffects: number;
};

/** One colour parameter of an effect, and the three channels it is assembled from. */
export type ArchivePpeColor = {
  name: string;
  /**
   * `m_fBase`, stored ahead of the envelopes and never read at runtime - `update` assembles the colour from the
   * three channels alone.
   */
  base: number | null;
  /** Keys across all three channels. */
  keys: number;
  /** Seconds the longest of the three spans, which is what this parameter contributes to the effect's own length. */
  lengthSeconds: number | null;
  channels: Array<ArchiveAnimationChannel>;
};

/** The colour grading an effect applies, which only version 2 carries. */
export type ArchivePpeColorMap = {
  /** The gradient texture the grading samples, absent when the effect names none. */
  texture: ArchiveReference | null;
  /** How much of the graded colour is mixed in over time. */
  influence: ArchiveAnimationChannel;
  /** Whether the effect grades at all, which is a name being present rather than an influence being non-zero. */
  isUsed: boolean;
};

/** Everything the viewer says about one post-process effect. */
export type ArchivePpeDescription = {
  version: number;
  /** Seconds the effect runs for, which is its longest parameter and not where its last key sits. */
  lengthSeconds: number | null;
  /** Keys across every parameter. */
  keys: number;
  /** The three colour parameters, in the order the file stores them. */
  colors: Array<ArchivePpeColor>;
  /** The seven scalar parameters, in the order the file stores them. */
  values: Array<ArchiveAnimationChannel>;
  /** The colour grading version 2 appends, absent below it. */
  colorMap: ArchivePpeColorMap | null;
};

/** One file a description names, and what became of it. */
export type ArchiveReference = {
  /** The name as the file authored it: engine style, and without the extension the loader implies. */
  name: string;
  /** The engine path the name was looked up as, or `None` for a name no logical path can be made of. */
  path: string | null;
  /** The name the open subject lists the file under, which is what a surface selects in the tree. */
  entry: string | null;
  status: ArchiveReferenceStatus;
};

/** Whether the subject being browsed holds what a description named. */
export enum EArchiveReferenceStatus {
  PRESENT = "present",
  ABSENT = "absent",
  UNKNOWN = "unknown",
}

/** Every `EArchiveReferenceStatus` as the spelling it crosses IPC as, for a value no member has narrowed. */
export type ArchiveReferenceStatus = `${EArchiveReferenceStatus}`;

/** How the open subject answers an engine path: every source it searches, in the order it searches them. */
export type ArchiveResolution = {
  /** Sources searched, highest priority first: the first one holding an engine path is the one that answers for it. */
  sources: Array<ArchiveResolutionSource>;
  /** Sources named but never opened, which are absent from the search rather than empty within it. */
  unread: Array<ArchiveUnreadSource>;
};

/** One source the open subject searches, and everything that says why it is searched where it is. */
export type ArchiveResolutionSource = {
  /** Where the source lives: a loose tree's root, or the directory holding a volume set. */
  path: string;
  /** Short name for the source — the directory or volume-set name. */
  label: string;
  /** Whether the source is a loose tree or a set of volumes. */
  kind: XraySourceKind;
  /**
   * How the mount plan described it: an `fsgame.ltx` alias such as `$game_data$`, or `root` or `volumes` for a path
   * named directly. Absent for a source no plan named.
   */
  origin: string | null;
  /** The root whose declaration reached this source, as the open named it. */
  step: string;
  /** Logical base the source mounts at; empty for a complete root. */
  base: string;
  /** Engine identities the source answers for, before anything searched ahead of it shadows one. */
  entries: number;
  /** Volumes merged behind this source, in the order a lookup reaches them. Empty for a loose tree. */
  volumes: Array<ArchiveResolutionVolume>;
};

/** One volume of a set, in the order a lookup reaches it. */
export type ArchiveResolutionVolume = {
  /** The volume file itself. */
  path: string;
  /** Entries its name table holds, before the merge shadows one of them. */
  entries: number;
  /** Bytes its entries occupy as stored. */
  sizeCompressed: number;
  /** Bytes its entries occupy once unpacked. */
  sizeReal: number;
  /** Root the volume unpacks under, from `[header] entry_point` with its alias stripped. */
  outputRootPath: string;
};

/** Everything the viewer says about the compiler shader library. */
export type ArchiveShaderCompilerDescription = {
  shaders: Array<ArchiveShaderCompilerShader>;
};

/** One compiler shader, as the viewer reads it. */
export type ArchiveShaderCompilerShader = {
  /** The name a surface declares, which the renderer's own blender library answers under too. */
  name: string;
  /** What the compiler is told to do with the surface, named. */
  flags: Array<string>;
  vertexTranslucency: number | null;
  vertexAmbient: number | null;
  /** Lightmap texels per unit, which is what a surface costs to bake. */
  lightmapDensity: number | null;
};

/** One definition of the library: what a shader name resolves to. */
export type ArchiveShadersBlender = {
  /** The shader name a mesh, a level surface or a config declares to reach this definition. */
  name: string;
  /** The class tag, which decides which passes are compiled and how its properties are read. */
  class: string;
  /** The class's own format version, which decides the order its `Load` reads properties in. */
  version: number;
  /** Machine the SDK last saved it on, the file's only provenance; empty where it carries none. */
  computer: string;
  /** Save time as the SDK stored it, which is a `u32` of its own and not an instant this can date. */
  time: number;
  properties: Array<ArchiveShadersProperty>;
};

/** Everything the viewer says about the blender library. */
export type ArchiveShadersDescription = {
  library: ArchiveShadersLibrary;
  blenders: Array<ArchiveShadersBlender>;
};

/** What the library holds, taken over the whole of it. */
export type ArchiveShadersLibrary = {
  blenders: number;
  /** Distinct class tags, of which the workspace libraries all use fifteen. */
  classes: number;
  /** Distinct textures the blenders bind by name, slots the renderer fills excluded. */
  textures: number;
  /** Distinct named textures the subject being browsed does not hold. */
  absentTextures: number;
};

/** One value an author left in a blender's property grid. */
export type ArchiveShadersProperty = {
  name: string;
  /** The type the file tags the payload with, as `xrEngine/Properties.h` names it. */
  kind: string;
  /** The value as a reader would compare it, empty for a marker, which carries none. */
  value: string;
  /** The texture a texture property names, when it names a file rather than a slot the renderer binds. */
  texture: ArchiveReference | null;
};

/** One copy of an engine path no lookup reaches, as the explorer lists it. */
export type ArchiveShadowedCopy = {
  /** Where this copy physically sits — a file on disk, or an entry of a volume. */
  container: XrayAssetContainer;
  /** Payload bytes once unpacked, as the mount holding this copy records or measures them. */
  sizeReal: number;
};

/** One reverb preset, as the viewer reads it. */
export type ArchiveSoundEnvironment = {
  /** The name a level's sound environments reach this preset by. */
  name: string;
  version: number;
  /** How long the reverb takes to fall away, in seconds. */
  decayTime: number | null;
  /** How much the room adds at low and at high frequencies, in hundredths of a decibel. */
  room: number | null;
  roomHf: number | null;
  /** How big the space sounds, in metres. */
  environmentSize: number | null;
  /** The EAX preset it stands for, which only version 4 and above declares. */
  environment: number | null;
};

/** Everything the viewer says about the sound environment library. */
export type ArchiveSoundEnvironmentDescription = {
  environments: Array<ArchiveSoundEnvironment>;
};

/** What a spawn set holds, taken from its header and the weight of its sections. */
export type ArchiveSpawnDescription = {
  version: number;
  /** Identity of this build of the set, which a save game is pinned to. */
  guid: string;
  /** Identity of the game graph the set was built against. */
  graphGuid: string;
  objects: number;
  levels: number;
  /** Top-level sections in the order the file frames them, with what each weighs. */
  sections: Array<ArchiveSpawnSection>;
  size: number;
};

/** One top-level section of a spawn set, by id and weight. */
export type ArchiveSpawnSection = {
  id: number;
  /** What the section holds, where the format names it. */
  label: string | null;
  size: number;
};

/** Every `kind` the `ArchiveSubject` union is told apart by, so a switch or a comparison names one. */
export enum EArchiveSubject {
  /** The volumes at one path, merged into a single name table. */
  VOLUMES = "volumes",
  /** A game folder read as the engine mounts it, archives and loose tree together. */
  WORLD = "world",
}

/**
 * What the explorer has open: a set of `.db` volumes, or a whole mounted world.
 *
 * The two are not folded into one shape. A volume set can say which volume an entry sits in, at what offset, with
 * which recorded CRC; a world can say which copy of an engine path wins and what that decision hides. A single
 * descriptor covering both would have a loose file claiming a volume position, which is the fiction this split
 * exists to avoid.
 *
 * Every difference between them is answered here, so a command stays an adapter and a reader asking how browsing an
 * installation differs from browsing a volume set opens one file.
 */
export type ArchiveSubject =
  /** The volumes at one path, merged into a single name table. */
  | { kind: "volumes"; project: ArchiveProject }
  /** A game folder read as the engine mounts it, archives and loose tree together. */
  | { kind: "world"; world: ArchiveWorld };

/** The bump declaration of a descriptor, `THM_CHUNK_BUMP`. */
export type ArchiveThmBump = {
  modeLabel: string;
  mode: number;
  /** Height the generator builds the pair against, read at generation time and never at runtime. */
  virtualHeight: number | null;
  /** The bump texture named, absent when the chunk names none. */
  texture: ArchiveReference | null;
  /**
   * Whether the engine would try to resolve the name: a mode that uses one, and a name to use.
   *
   * A name that resolves to nothing does not turn bump mapping off. `bump_exist` tests only that the name is
   * non-empty, so the renderer still takes the `_bump` variant and the loader substitutes `ed\ed_dummy_bump`.
   */
  isUsed: boolean;
};

/**
 * A declared size the texture beside the descriptor does not match.
 *
 * Reported as two facts rather than as a fault: the descriptor's width and height are authoring data and the file is
 * the authority, so a disagreement is worth seeing and is not by itself wrong.
 */
export type ArchiveThmDeclaredSize = {
  width: number;
  height: number;
  /** Whether the declaration is a cube map's source strip: six faces wide, one face tall. */
  isCubeStrip: boolean;
};

/**
 * Everything the viewer says about one texture descriptor.
 *
 * Field order is reading order, and reading order is the engine's rather than the file's: what the descriptor
 * describes, whether the engine reads it at all, what it names, how it shades, and only then the build recipe the
 * converter already consumed. The file's own chunk order puts the recipe third, which is the order to write it back
 * in and not the order to read it in.
 *
 * Every chunk stays optional, because an absent chunk and a chunk holding a default are different files: a descriptor
 * with no bump chunk is not one declaring `bump mode: None`.
 */
export type ArchiveThmDescription = {
  texture: ArchiveThmTexture;
  textureType: ArchiveThmTextureType;
  bump: ArchiveThmBump | null;
  detail: ArchiveThmDetail | null;
  /** The normal map replacing the one the generator would derive, when the file names one. */
  externalNormalMap: ArchiveReference | null;
  material: ArchiveThmMaterial | null;
  parameters: ArchiveThmParameters | null;
  /** Mip level the converter starts fading from, `fade_delay` (`ETextureParams.h:88`). */
  fadeDelay: number | null;
  file: ArchiveThmFile;
};

/** The detail association of a descriptor, `THM_CHUNK_DETAIL_EXT`. */
export type ArchiveThmDetail = {
  scale: number | null;
  /** The detail texture named, absent when the chunk names none. */
  texture: ArchiveReference | null;
  /**
   * The flags that switch this association on, by the SDK's own spelling.
   *
   * Empty when the engine reads past the association. Repeated here as well as in the flag word because the
   * association means nothing without them.
   */
  enabledBy: Array<string>;
};

/** What the file is, apart from what it declares. */
export type ArchiveThmFile = {
  version: number | null;
  /** Whether the version is the one `ETextureThumbnail::Load` accepts. */
  isSupportedVersion: boolean;
  /** The kind of asset the thumbnail describes; `1` is a texture and the only one these tools read. */
  thumbnailType: number | null;
  thumbnail: ArchiveThmThumbnail | null;
  /** Chunk ids the reader could not fold into a field, in the order it read them. */
  extraChunks: Array<number>;
};

/** One bit of the texture param flag word. */
export type ArchiveThmFlag = {
  /** SDK identifier, which is the name an author of a `.thm` would recognise. */
  label: string;
  isSet: boolean;
};

/**
 * The shading declaration of a descriptor, `THM_CHUNK_MATERIAL`.
 *
 * The one piece of authoring data that reaches the renderer through the descriptor rather than through the DDS.
 */
export type ArchiveThmMaterial = {
  /** The two lighting models the surface sits between. */
  label: string;
  value: number;
  /** Where between them it sits. */
  weight: number | null;
};

/**
 * The conversion parameters a descriptor carries, `THM_CHUNK_TEXTUREPARAM`.
 *
 * Authoring data the converter consumed and the runtime does not read, with two exceptions that live in
 * the detail section, which reports them where they take effect.
 */
export type ArchiveThmParameters = {
  formatLabel: string;
  format: number;
  mipFilterLabel: string;
  mipFilter: number;
  borderColor: number;
  fadeColor: number;
  fadeAmount: number;
  width: number;
  height: number;
  /**
   * Every bit the SDK names, in bit order, set or not.
   *
   * All twelve rather than the set ones: a recipe is read to see what the converter was told to do, and "dither is
   * off" answers that as well as "dither is on".
   */
  flags: Array<ArchiveThmFlag>;
  /**
   * Bits the word carries that the SDK has no name for.
   *
   * Two vanilla descriptors carry one, so dropping the residue would silently lose what somebody's tool set.
   */
  unnamedFlags: number;
};

/** The texture a descriptor sits beside, and what that file actually is. */
export type ArchiveThmTexture = {
  /** The `.dds` this descriptor describes, which is the file beside it rather than a name it carries. */
  reference: ArchiveReference;
  /** What that file's header declares, when it was found and its header parsed. */
  shape: AssetTextureShape | null;
  /** The size the descriptor claims, carried only when the file beside it measures something else. */
  declared: ArchiveThmDeclaredSize | null;
};

/** The kind of texture a descriptor describes, `STextureParams::ETType`. */
export type ArchiveThmTextureType = {
  /** Engine token for the type, or the raw number for one the SDK does not name. */
  label: string;
  value: number;
  /**
   * Whether `CTextureDescrMngr::LoadTHM` reads the bump, detail and material of a descriptor of this type at all.
   *
   * False for 743 of vanilla's 2,736 descriptors - every cube map and every bump map - whose declarations the engine
   * never looks at however complete they are.
   */
  isReadByEngine: boolean;
  /** Whether the file declares a type, or the engine's zeroed default is what applies. */
  isDeclared: boolean;
};

/**
 * The preview picture a descriptor carries, `THM_CHUNK_DATA`.
 *
 * Reported by size and never decoded. The trunk SDK stopped writing the chunk - its `w_chunk` call is commented out
 * in `ETextureThumbnail::Save` - and 11 of the 19,849 descriptors across the workspace trees still carry one.
 */
export type ArchiveThmThumbnail = {
  /** Whether the payload is the engine's own compressed stream, which is the only form seen in the wild. */
  isCompressed: boolean;
  size: number;
};

/** A source the plan named that could not be opened, so the search reaches nothing it holds. */
export type ArchiveUnreadSource = {
  /** Where the source that failed to open lives. */
  path: string;
  /** How the plan described it, such as an `fsgame.ltx` alias. */
  origin: string;
  /** Why it could not be opened. */
  reason: string;
};

/**
 * One mounted world the explorer browses: an installation, or any tree read as the engine would read it.
 *
 * The other subject of the same explorer answers for one volume set and nothing else. Pointing that at a game folder
 * lists the archives and silently omits the loose `gamedata` tree in front of them, so it shows the archived payload
 * for files the engine would serve from disk. This one answers the other question: which copy actually wins, and what
 * that decision hides.
 *
 * A listing rather than a VFS. The mounts live in the application's one [`crate::core::assets::AssetMountState`],
 * where every other surface's reads already go and where mounting one installation twice costs one index; what the
 * session owns is the answer it published.
 */
export type ArchiveWorld = {
  /** How the world was opened, so every later read of it addresses exactly these mounts. */
  roots: XrayRoots;
  /** Sources searched, highest priority first, as each one names itself. */
  mounts: Array<string>;
  /** Winning entries, one per engine path, ordered by that path. */
  files: Array<ArchiveWorldEntry>;
  readPolicy: ArchiveReadPolicy;
  /** Unpacked bytes of the winning entries. What is shadowed is not counted; it is not what the engine would load. */
  sizeReal: number;
  /** Engine paths this world answers with more than one copy for. */
  shadowedCount: number;
  /** Unpacked bytes held by the copies no lookup reaches. */
  shadowedSizeReal: number;
};

/**
 * One file of a mounted world, as the explorer lists it.
 *
 * Shaped like [`xrf_archive::ArchiveFileDescriptor`] where the two can agree — a `name` and a `size_real` — because
 * the tree, the filter and the preview gate above them need nothing else, and giving each subject its own spelling of
 * those two would fork every one of those surfaces.
 *
 * Where they cannot agree, this says less rather than inventing something. A loose file has no volume position, no
 * stored size and no recorded CRC, so nothing here claims one; what it has instead is the copies it stands in front
 * of, which a volume set has no way to express.
 */
export type ArchiveWorldEntry = {
  /** Engine identity: lower-case and backslash separated, which is what every read of this world is addressed by. */
  name: string;
  /** Where the winning copy physically sits — a file on disk, or an entry of a volume. */
  container: XrayAssetContainer;
  /** Payload bytes once unpacked. */
  sizeReal: number;
  /** Copies of this engine path no lookup reaches, in mount priority order behind the winner. */
  shadowed: Array<ArchiveShadowedCopy>;
};

/**
 * Directory extraction request for whichever subject the explorer has open.
 *
 * One shape for both: the two differ in where the bytes come from, never in what a person asked for.
 */
export type ArchivesExtractRequest = {
  sessionId: SessionId;
  /** Directory inside the opened tree to extract. An empty prefix means everything. */
  prefix: string;
  /** Directory to write the contents into. */
  destination: string;
};

/** Archive packing request. */
export type ArchivesPackRequest = {
  /** What to pack and how. */
  config: ArchivePackConfig;
  /** Whether an existing output may be overwritten. */
  isForced: boolean;
};

/** Shared request for archive comparison and patch publication. */
export type ArchivesPatchRequest = {
  /** What to compare and where to publish the difference. */
  config: ArchivePatchConfig;
  /** Whether an existing output may be overwritten. Ignored by a comparison, which writes nothing. */
  isForced: boolean;
  /** Whether a checksum match should be proven by comparing the payloads themselves. */
  isVerifyingPayload: boolean;
};

/** Archive unpacking request. */
export type ArchivesUnpackRequest = {
  /** Archive or directory of archives to read. */
  from: string;
  /** Directory to write the contents into. */
  destination: string;
};

/** What a texture file is, once it has been located. */
export type AssetTextureDescriptor = {
  /** Bytes the file occupies, which is also what a renderer uploads for a block-compressed texture. */
  size: number;
  /** Header facts, absent when the bytes are not a readable DDS. */
  shape: AssetTextureShape | null;
};

/** Pixel layout a DDS header declares. */
export type AssetTextureShape = {
  width: number;
  height: number;
  /** Levels the file carries, one meaning no mip chain at all. */
  mipmapLevels: number;
  /** Format name from [`DdsMetadata::get_format_label`], so the viewer and the sweep agree on what a file is. */
  format: string;
};

/**
 * What a sound is, once it has been located.
 *
 * Every field is optional because there are two independent ways to know less than everything: bytes that are not a
 * readable ogg at all, and a perfectly good ogg carrying no X-Ray comment. Reporting a zero for either would make an
 * unreadable file indistinguishable from a silent one.
 */
export type AudioDescriptor = {
  /** Absent when the bytes carry no readable stream header. */
  channels: number | null;
  /** Absent when the bytes carry no readable stream header. */
  sampleRate: number | null;
  /** Absent for a sound carrying no recognized X-Ray comment, where the engine uses its own defaults. */
  parameters: AudioSourceParameters | null;
};

/** The X-Ray source parameters carried in a sound's first vorbis comment. */
export type AudioSourceParameters = {
  minDistance: number | null;
  maxDistance: number | null;
  baseVolume: number | null;
  gameType: number;
  maxAiDistance: number | null;
};

/**
 * One config as the authored view renders it.
 *
 * Text and structure travel together but stay separate records: the text is what a person edits and the structure is
 * what only the parser knows, and a future edit replaces one without invalidating the shape of the other.
 */
export type ConfigsDocument = {
  text: LtxFileText;
  structure: LtxFileStructure;
  /** What is wrong with this file itself: it will not parse, or an `#include` reached nothing. */
  findings: Array<LtxAnchoredFinding>;
};

/** What a config formatting run, or a check of one, was asked to do. */
export type ConfigsFormatRequest = {
  /** Trees to search, and how each is read. */
  roots: XrayRoots;
  /** Scope inside those trees, or nothing for all of them. */
  prefix: string | null;
};

/** What opening a configs project for browsing was asked to do. */
export type ConfigsOpenRequest = {
  sessionId: SessionId;
  /** Trees to search, and how each is read. */
  roots: XrayRoots;
  /** Scope inside those trees, or nothing for all of them. */
  prefix: string | null;
  /** Whether to resolve with the Monolith/Anomaly DLTX patch dialect. */
  isDltx: boolean;
};

/** What one open of the configs explorer answers with. */
export type ConfigsProjectDescriptor = {
  /** Identity every later read is addressed by. */
  sessionId: SessionId;
  /** The trees this project searched, as the backend resolved them, so a reload restores the same open. */
  roots: XrayRoots;
  /** Scope inside those trees, or nothing for all of them. */
  prefix: string | null;
  /** Whether configs resolve under the Monolith/Anomaly patch dialect. */
  isDltx: boolean;
  /** Host path the project reports itself at, for a crumb that names something a person recognises. */
  root: string;
  /** Every config the project holds, and what each one is to it. */
  inventory: LtxInventory;
  /**
   * Section schemes the project's `*.scheme.ltx` files declare, by name.
   *
   * Sent once with the open rather than per document: a tree declares tens of them and every structure read joins
   * against the same set.
   */
  declaredSchemes: Array<string>;
};

/** Which config of which open a reader wants. */
export type ConfigsReadDocumentRequest = {
  /** The open this read is addressed to; a read naming a replaced one is refused rather than answered. */
  sessionId: SessionId;
  /** Engine identity of the config to read. */
  path: string;
};

/** Which sections of a resolved root a page wants. */
export type ConfigsReadSectionsRequest = {
  /** The open this read is addressed to; a read naming a replaced one is refused rather than answered. */
  sessionId: SessionId;
  /** Engine identity of the entry point the sections belong to. */
  entry: string;
  /** Sections to read, as the index named them. */
  names: Array<string>;
};

/** Which resolved root a reader wants, of which open. */
export type ConfigsResolvedRequest = {
  /** The open this read is addressed to; a read naming a replaced one is refused rather than answered. */
  sessionId: SessionId;
  /** Engine identity of the entry point to resolve. */
  entry: string;
};

/** Which section of a resolved root a reader wants explained. */
export type ConfigsSectionRequest = {
  /** The open this read is addressed to; a read naming a replaced one is refused rather than answered. */
  sessionId: SessionId;
  /** Engine identity of the entry point the section belongs to. */
  entry: string;
  /** The section to explain, as the index named it. */
  section: string;
};

/** What a config verification was asked to do. */
export type ConfigsVerifyRequest = {
  /** Trees to search, and how each is read. */
  roots: XrayRoots;
  /** Scope inside those trees, or nothing for all of them. */
  prefix: string | null;
  /** Whether to resolve with the Monolith/Anomaly DLTX patch dialect. */
  isDltx: boolean;
};

/**
 * What everything below one process costs, folded in a single walk of the table.
 *
 * A pair rather than one figure, because the total is unreadable without it: a webview runs a browser process, a GPU
 * process and a renderer per frame tree, and "580 MB" means something different across two of those than across
 * eight.
 */
export type DescendantUsage = {
  /** Resident set of every process descended from the root, at any depth. */
  residentMemory: number;
  /** How many such processes there are. Zero on a platform that runs the webview in the host process. */
  processes: number;
};

/** What opening a dialogs project was asked to do. */
export type DialogsOpenRequest = {
  sessionId: SessionId;
  /** Trees to search, and how each is read. */
  roots: XrayRoots;
  /** How much of the project to read. */
  mode: DialogProjectMode;
  /** Scope holding the dialog files, or nothing for all of them. */
  dialogsPrefix: string | null;
  /** Scope holding the string tables, or nothing for all of them. */
  translationsPrefix: string | null;
};

/** Identifies a dialog within one committed project. */
export type DialogsReadRequest = {
  sessionId: SessionId;
  logicalPath: string;
  id: string;
  language: string | null;
};

/** Every `kind` the `EquipmentConfigSource` union is told apart by, so a switch or a comparison names one. */
export enum EEquipmentConfigSource {
  /** A `system.ltx` on disk, named by its filesystem path. */
  FILE = "file",
  /** An entry point of the roots, named by its logical path such as `configs\system.ltx`. */
  ASSET = "asset",
}

/**
 * Where the configuration that annotates a sheet is read from.
 *
 * Parallel to [`EquipmentSheetSource`] and for the same reason, with one difference that matters: reading a
 * configuration out of the roots resolves a whole include tree through the VFS, which is what reaches an
 * installation's `db\configs` volumes. A file named directly is resolved from its own directory, which is what
 * carries `mod_*.ltx` attachments sitting beside it.
 */
export type EquipmentConfigSource =
  /** A `system.ltx` on disk, named by its filesystem path. */
  | { kind: "file"; path: string }
  /** An entry point of the roots, named by its logical path such as `configs\system.ltx`. */
  | { kind: "asset"; logicalPath: string };

/** Where an opened sheet turned out to be, and whether anything can write there. */
export type EquipmentSheetLocation = {
  /** The sheet as the roots located it, or nothing for a file opened by path outside any tree. */
  asset: XrayAsset | null;
  /** Absolute path of the sheet, or nothing when it lives inside an archive volume. */
  path: string | null;
  /** Where a write to the sheet would land. */
  writeTarget: string | null;
};

/** Every `kind` the `EquipmentSheetSource` union is told apart by, so a switch or a comparison names one. */
export enum EEquipmentSheetSource {
  /** A loose `.dds` on disk, named by its filesystem path. */
  FILE = "file",
  /** A sheet of the roots, named by its engine reference such as `ui\ui_icon_equipment`. */
  ASSET = "asset",
}

/** Where an equipment sheet is read from. */
export type EquipmentSheetSource =
  /** A loose `.dds` on disk, named by its filesystem path. */
  | { kind: "file"; path: string }
  /** A sheet of the roots, named by its engine reference such as `ui\ui_icon_equipment`. */
  | { kind: "asset"; reference: string };

/** Everything the editor is told about one opened sheet. */
export type EquipmentSpriteMetadata = {
  /** Name the sheet is streamed to the webview under. */
  name: string;
  /** What this open was asked to read, so a reload repeats the request rather than a reconstruction of it. */
  open: EquipmentSpriteOpen;
  /** Which copy of the sheet this is, and whether anything can write to it. */
  location: EquipmentSheetLocation;
  /** Why the configuration was not read, when it was asked for and could not be. */
  configError: string | null;
  /** Every section occupying a slot on the sheet, in the order the configuration declares them. */
  occupants: Array<EquipmentSlotOccupant>;
};

/** Everything one opening of a sheet was asked to read. */
export type EquipmentSpriteOpen = {
  /** Trees to search, and how each is read. */
  roots: XrayRoots;
  sheet: EquipmentSheetSource;
  /** The configuration naming what sits on the sheet, or nothing to open it unannotated. */
  config: EquipmentConfigSource | null;
  /** Whether to resolve that configuration with the Monolith/Anomaly DLTX patch dialect. */
  isDltx: boolean;
};

/** One check's verdict, as the desktop surface shows it. */
export type GamedataCheckSummary = {
  /** The check that ran, spelled as the command line spells it. */
  check: string;
  /** `passed`, `failed`, `incomplete`, or `skipped`. */
  status: string;
  /** The check's own one-line verdict. */
  summary: string;
  findings: number;
  /** How long this check took, where it measured itself. */
  duration: number | null;
};

/** What a verification was asked to do. */
export type GamedataVerifyRequest = {
  /** Gamedata root to verify. */
  root: string;
  /** Checks to run, or nothing for every one this build knows. */
  checks: Array<string> | null;
  /** Whether a check that would warn should fail instead. */
  isStrict: boolean;
};

/** What a whole verification reports back to the desktop surface. */
export type GamedataVerifySummary = {
  /**
   * Whether every selected check finished, or the run stopped before or inside a check.
   *
   * A stopped run's checks are real verdicts; its silence about the rest is not one.
   */
  outcome: JobOutcome;
  /** The aggregate verdict over the checks that ran. */
  status: string;
  checks: Array<GamedataCheckSummary>;
  duration: number;
};

/**
 * What the application is running on and with, none of which changes while it runs.
 *
 * Split from [`RuntimeSnapshot`](super::RuntimeSnapshot) because that one is polled: re-reading the operating
 * system's name every second to show the same string is work nobody asked for, and mixing a constant into a reading
 * invites a surface to refresh the wrong half.
 *
 * Every field an operating system may decline to report is `Option`, the way `BuildInfo` treats what a build could
 * not record - naming the absence beats substituting a plausible default.
 */
export type HostInfo = {
  /** Tauri the application was linked against. */
  tauriVersion: string;
  /**
   * Webview actually serving the window, which is the runtime installed on the machine rather than a compiled-in
   * version. Absent where the platform cannot be asked, and on Windows where no WebView2 runtime answered.
   */
  webviewVersion: string | null;
  /** Operating system's short name, such as `Windows` or `Ubuntu`. */
  osName: string | null;
  /** Operating system's own version, as it numbers itself. */
  osVersion: string | null;
  /** Kernel behind it, which on Windows is the build number a compatibility report is quoted by. */
  kernelVersion: string | null;
  /** Architecture the binary is executing on, as opposed to the target triple it was built for. */
  arch: string;
  /** Logical processors, which is what the execution pool's width is drawn from. */
  cpuCount: number;
  /** Physical cores, absent where the platform does not distinguish them. */
  physicalCoreCount: number | null;
  /** Total physical memory of the machine, the figure every usage reading is read against. */
  totalMemory: number;
  /** This process's own identifier, for pairing what is shown here with a task manager. */
  pid: number;
};

/** How a job that is no longer running ended. */
export enum EJobConclusion {
  COMPLETED = "completed",
  CANCELLED = "cancelled",
  FAILED = "failed",
}

/** Every `EJobConclusion` as the spelling it crosses IPC as, for a value no member has narrowed. */
export type JobConclusion = `${EJobConclusion}`;

/** One job as the listing describes it, running or recently finished. */
export type JobDescription = {
  id: string;
  /** What kind of work this is, as the command that started it named itself. */
  kind: JobKind;
  /** What this job holds exclusively, so a refused start can be explained by pointing at the job that refused it. */
  leaseKeys: Array<string>;
  /**
   * What the job was asked to do, as the command that started it described itself.
   * Absent for a job whose command described nothing.
   */
  request: unknown | null;
  /**
   * Whether stopping has been asked for. A job can carry this and still be running: cancellation lands at a boundary
   * the operation chooses, and the gap between asking and stopping is exactly what a reader wants to see.
   */
  isCancelRequested: boolean;
  /**
   * The job's own progress: live for a running job, as last seen for a finished one.
   * Absent for a job registered but not yet reporting — a run holding a lease while it validates its inputs, say.
   */
  progress: JobProgress | null;
  /** Absent while the job is running. */
  conclusion: JobConclusion | null;
  /** Why it failed, where it did. */
  error: string | null;
  /**
   * What the run answered, for a job that completed.
   * Absent while the job runs, and for a job that failed or was cancelled before it had an answer.
   */
  result: unknown | null;
  /** How long the job ran, measured by the registry rather than by the operation. */
  duration: number;
  /** When the registry took it on, as milliseconds since the epoch. */
  startedAt: number;
};

/** The application operations that can be registered, cancelled and rediscovered. */
export enum EJobKind {
  ARCHIVES_EXTRACT = "archives.extract",
  ARCHIVES_COMPARE = "archives.compare",
  ARCHIVES_PACK = "archives.pack",
  ARCHIVES_PATCH = "archives.patch",
  ARCHIVES_UNPACK = "archives.unpack",
  CONFIGS_CHECK_FORMAT = "configs.check-format",
  CONFIGS_FORMAT = "configs.format",
  CONFIGS_VERIFY = "configs.verify",
  SPAWN_PACK = "spawn.pack",
  SPAWN_UNPACK = "spawn.unpack",
  SPRITE_EQUIPMENT_PACK = "sprite-equipment.pack",
  GAMEDATA_VERIFY = "gamedata.verify",
  TEXTURES_BUILD = "textures.build",
  TEXTURES_COMPARE_ENCODINGS = "textures.compare-encodings",
  TEXTURES_MAKE_BUMP = "textures.make-bump",
  TEXTURES_SAVE = "textures.save",
  TRANSLATIONS_BUILD = "translations.build",
  TRANSLATIONS_CHECK_FORMAT = "translations.check-format",
  TRANSLATIONS_FORMAT = "translations.format",
  TRANSLATIONS_PARSE = "translations.parse",
  TRANSLATIONS_VERIFY = "translations.verify",
}

/** Every `EJobKind` as the spelling it crosses IPC as, for a value no member has narrowed. */
export type JobKind = `${EJobKind}`;

/** What the machine as a whole is using. */
export type MachineUsage = {
  /** Physical memory in use across every process. */
  usedMemory: number;
  /**
   * Physical memory the machine reports as free for a new allocation, which is not `total - used`: the difference is
   * cache the operating system would hand back under pressure.
   */
  availableMemory: number;
};

/** What an equipment sprite pack was asked to do. */
export type PackSpriteRequest = {
  /** Directory of loose icons to draw from. */
  sourcePath: string;
  /** Sheet to write. */
  outputPath: string;
  /** `system.ltx` declaring which icons exist and where they sit. */
  systemLtxPath: string;
  /** Whether to resolve that config with the Monolith/Anomaly DLTX patch dialect. */
  isDltx: boolean;
};

/** What a path is right now, for a form field that has to say what it points at before a command runs. */
export type PathDescription = {
  kind: PathKind;
  /** Entries directly inside a directory. */
  entryCount: number | null;
};

/**
 * What a path turned out to be.
 *
 * Carried instead of a pair of booleans so the states cannot disagree. Failing to look is not a variant here: it
 * reaches the caller as an error, which is what lets a refused check read as unknown rather than as absent.
 */
export enum EPathKind {
  MISSING = "missing",
  FILE = "file",
  DIRECTORY = "directory",
}

/** Every `EPathKind` as the spelling it crosses IPC as, for a value no member has narrowed. */
export type PathKind = `${EPathKind}`;

/** What one process holds. */
export type ProcessUsage = {
  /** Physical memory the process actually occupies. */
  residentMemory: number;
  /** Address space it has reserved, which is routinely several times the resident set and is not what it costs. */
  virtualMemory: number;
};

/**
 * One reading of what the application costs, and of how long it has been running.
 *
 * Every figure is a reading rather than a total: nothing here accumulates, so a caller polling this sees the current
 * state and never a history it did not ask to keep.
 *
 * Grouped by subject rather than flattened, because the same word means three different things depending on whose
 * memory is being reported, and a prefix on each field is a worse way of saying so than a name around each group.
 */
export type RuntimeSnapshot = {
  /** Milliseconds since the epoch at which the process began, stamped in `main`. */
  startedAt: number;
  /** Milliseconds it has been running, measured monotonically rather than by subtracting two wall-clock readings. */
  uptime: number;
  /** What the backend process itself holds. */
  process: ProcessUsage;
  /** What the processes below it hold, which on this stack is the webview. */
  descendants: DescendantUsage;
  /** What the whole machine is using, for reading the two above against. */
  machine: MachineUsage;
};

/**
 * What the viewer is showing, paired with where it came from.
 *
 * The enclosing snapshot supplies the geometry identity; source and roots describe its inputs and texture lookups.
 */
export type SelectedVisualDescription = {
  source: VisualSource;
  /** The roots used to resolve this selection's texture files. */
  roots: XrayRoots;
  description: VisualDescription;
  dependencies: VisualDependencies;
  /** What each located texture file is, keyed by the logical path that located it. */
  textures: { [key in string]: AssetTextureDescriptor };
  /** What the renderer builds for each declared texture, keyed by the reference as the mesh declares it. */
  materials: { [key in string]: XrayMaterialDescriptor };
  /** How the renderer draws each declared shader, keyed by the shader name as the mesh declares it. */
  surfaces: { [key in string]: XraySurfaceDescriptor };
  /** A `textures.ltx` the searched roots hold, or `None`. */
  texturesLtx: XrayAsset | null;
};

/** A single opening, allocated by its caller before dispatch so it can also be closed while pending. */
export type SessionId = string;

/** The optional committed snapshot returned during restoration. */
export type SessionRestore<T> = SessionSnapshot<T> | null;

/** An immutable value addressed by the opening that produced it; a held snapshot survives close. */
export type SessionSnapshot<T> = {
  sessionId: SessionId;
  value: T;
};

/** The conversion performed, retained with the result after a window reload. */
export enum ESpawnConversion {
  PACK = "pack",
  UNPACK = "unpack",
}

/** Every `ESpawnConversion` as the spelling it crosses IPC as, for a value no member has narrowed. */
export type SpawnConversion = `${ESpawnConversion}`;

/** Source and output paths of a standalone spawn conversion. */
export type SpawnConversionRequest = {
  source: string;
  destination: string;
};

/** What reached disk; cancellation is accepted only before writing begins. */
export type SpawnConversionResult = {
  operation: SpawnConversion;
  destination: string;
  outcome: JobOutcome;
};

/** One coherent opening, restored without reading the large chunks. */
export type SpawnSessionDescriptor = {
  sessionId: SessionId;
  path: string;
  header: SpawnHeaderChunk;
};

/** The complete identity and inputs of one sprite opening. */
export type SpriteEquipmentOpenRequest = {
  sessionId: SessionId;
} & EquipmentSpriteOpen;

/** What a tree shows on a texture before anyone opens it, read from its descriptor alone. */
export type TextureBadges = {
  /** Both halves of the declared pair resolved to the files the declaration names. */
  isBumped: boolean;
  /** The bump shader path is taken but a half is a dummy or absent, so the surface is not what was authored. */
  isDegraded: boolean;
  /** The descriptor's texture type makes `LoadTHM` skip it whole, bump declaration included. */
  isEngineSkipped: boolean;
  /** A detail texture is named and one of the two flags that switch it on is set. */
  isDetailAssociated: boolean;
  /** A `.thm` sits there and does not parse as one. */
  isUnreadable: boolean;
};

/** The roots and listing mode restored after a frontend reload. */
export type TextureBrowseSession = {
  roots: XrayRoots;
  mode: TextureCatalogMode;
};

/** One recipe field the descriptor asks for that this build does not carry out. */
export type TextureBuildOmissionReport = {
  /** The descriptor field this sits beside, under the name the SDK gives it. */
  field: string;
  /** Why the build does not do it, in words a person deciding whether to rebuild can act on. */
  reason: string;
};

/** What a rebuilt texture came to. */
export type TextureBuildOutcome = {
  /**
   * Whether the texture was written or the run stopped before it started.
   *
   * A cancelled build wrote nothing. There is one boundary and it is before the work: the encode writes the file
   * itself and is a single call with no seam inside it. That costs nothing worth having, because the encode is tens
   * of milliseconds - a descriptor decides the format here and `ETFormat` has no name for BC7, so the one candidate
   * that takes seconds cannot arise.
   */
  outcome: JobOutcome;
  destination: string;
  /** Size of the source, which the descriptor's own width and height are refreshed from. */
  width: number;
  height: number;
  /** Levels written, counting the base. */
  mipmapLevels: number;
  /** Recipe fields the descriptor asked for and the build did not carry out. */
  omissions: Array<TextureBuildOmissionReport>;
};

/** The two references a declaration binds, which is what folds a pair under the texture that declares it. */
export type TextureBumpPair = {
  bump: string;
  companion: string;
};

/** Every texture the roots hold, once per reference, in reference order. */
export type TextureCatalog = {
  /** How this listing was made, which decides what its rows are addressed by and whether a sweep can badge them. */
  mode: TextureCatalogMode;
  /** The roots the catalog was listed from, so a later read searches what the listing searched. */
  roots: XrayRoots;
  /** A `textures.ltx` the roots hold, or `None`. Its declarations are not read, so a surface says where it matters. */
  texturesLtx: XrayAsset | null;
  entries: Array<TextureEntry>;
  /**
   * `.dds` files the roots hold outside `textures\`, which no engine reference names and this catalog leaves out.
   *
   * Counted rather than dropped silently: a level's lightmaps are the usual case, and a person wondering where a file
   * went deserves the number.
   */
  outsideTexturesCount: number;
};

/**
 * How a listing addressed what it found.
 *
 * Two shapes rather than one because the two cases want opposite defaults. In a game tree the files that yield no
 * engine reference are a level's lightmaps, and burying two thousand named textures in them is the bug; in a folder
 * somebody is authoring in, those files are the entire point and there are no references to be had at all.
 */
export enum ETextureCatalogMode {
  /** The game tree: listed by engine reference, archives included, files outside `textures\` counted not listed. */
  ROOTS = "roots",
  /** A plain directory: every `.dds` under it, addressed by its own path. */
  LOOSE_DIRECTORY = "looseDirectory",
}

/** Every `ETextureCatalogMode` as the spelling it crosses IPC as, for a value no member has narrowed. */
export type TextureCatalogMode = `${ETextureCatalogMode}`;

/** Everything the inspection panels say about one texture, resolved in one call. */
export type TextureDescription = {
  source: TextureSource;
  /** What to call this texture on screen: its engine reference, or a standalone file's own stem. */
  reference: string;
  /** The roots the description was resolved in, so a later read searches what this searched. */
  roots: XrayRoots;
  /** The `.dds` the reference resolves to, or `None` for a descriptor with no texture. */
  texture: XrayAsset | null;
  /** What the base texture file is, when it is located and its bytes can be reached. */
  base: AssetTextureDescriptor | null;
  /**
   * What the renderer would build for this texture, or `None` for a file outside every tree.
   *
   * Absent rather than empty for a standalone file. There is no tree to resolve a bump pair or a detail against, so
   * answering "declares nothing" would be a claim this description is in no position to make - the descriptor beside
   * the file may well declare a pair, and what the engine would do with it depends on a game tree nobody has named.
   */
  material: XrayMaterialDescriptor | null;
  /** What the bound bump file is, when the material binds one and its bytes can be reached. */
  bump: AssetTextureDescriptor | null;
  /** What the bound bump companion file is, on the same terms. */
  companion: AssetTextureDescriptor | null;
  /** The descriptor's editable fields, when a `.thm` was located and parsed. */
  form: TextureDescriptorForm | null;
  /** Where an edit of this texture would write, absent for a texture served out of an archive. */
  targets: TextureEditTargets | null;
};

/** The descriptor fields the editor owns, read off a `.thm` and written back onto one. */
export type TextureDescriptorForm = {
  /** The thumbnail's texture type, which is the gate `LoadTHM` reads before anything else. */
  textureType: number;
  bumpMode: number;
  /** Bump texture path without extension, engine-style with backslashes. Empty when unused. */
  bumpName: string;
  /** Detail texture path on the same terms. */
  detailName: string;
  detailScale: number | null;
  /**
   * The whole `STextureParams` flag word, named bits and unnamed alike.
   *
   * One word rather than a boolean per bit: the twelve the SDK names are the ones a surface offers, and the rest are
   * bits somebody's tool set that this editor has no business dropping.
   */
  flags: number;
  format: number;
  mipFilter: number;
  borderColor: number;
  fadeColor: number;
  fadeAmount: number;
  /** Mip level the converter starts fading from. */
  fadeDelay: number;
  material: number;
  materialWeight: number | null;
  extNormalMapName: string;
  /** Read by the bump generator and by nothing at runtime. */
  virtualHeight: number | null;
  /** What the descriptor claims its texture measures, which the DDS beside it is the authority on. */
  width: number;
  height: number;
};

/** The descriptor half of a save. */
export type TextureDescriptorSave = {
  target: TextureSaveTarget;
  form: TextureDescriptorForm;
};

/**
 * Where an edit of one texture would write, and what was there when the editor read it.
 *
 * Resolved by the command that located the files rather than derived by the frontend, for the same reason the
 * descriptor is: a path assembled in TypeScript out of a reference and a separator is a guess about where the VFS
 * found something, and the two disagree the moment a root is nested or a name is cased differently.
 *
 * Absent for a texture served out of an archive, which has no file to replace at all.
 */
export type TextureEditTargets = {
  /**
   * The `.thm` to write, whether or not one is there yet.
   *
   * Always present, because the editor can author a descriptor for a texture that has none: its
   * [`TextureSaveTarget::expected`] is what says which of the two cases this is.
   */
  descriptor: TextureSaveTarget;
  /** The `.dds` to replace when a re-encode is saved. */
  texture: TextureSaveTarget;
};

/** Every candidate weighed against one texture, with the texture itself for a baseline. */
export type TextureEncodingComparison = {
  sessionId: SessionId;
  source: TextureSource;
  roots: XrayRoots;
  /**
   * Whether every candidate was weighed or the run stopped because it was asked to.
   *
   * A cancelled comparison still reports what it managed, and the session still holds those encodes: a candidate it
   * reached is a real measurement and a real set of bytes, whatever happened after it.
   */
  outcome: JobOutcome;
  /** The texture the encodes were made from, which a save has to name to claim them. */
  reference: string;
  current: TextureEncodingCurrent;
  /** The candidates weighed, in the order [`DdsEncodeCandidate::ALL`] lists them, cheapest first. */
  candidates: Array<TextureEncodingReport>;
};

/** The texture as it stands, for the row a comparison is read against. */
export type TextureEncodingCurrent = {
  /** Format name from the file's own header, which is not always one of the five candidates. */
  label: string;
  fileBytes: number;
  gpuBytes: number;
  width: number;
  height: number;
  mipmapLevels: number;
};

/**
 * A format the base texture can be written in, of the five worth offering.
 *
 * A plugin-side mirror of [`DdsEncodeCandidate`] rather than the crate's own enum, for the reason every wire type
 * here is one: `xrf-dds` is a pure image crate and carries no bindings feature, and a surface naming a format wants
 * a name that cannot change under it.
 */
export enum ETextureEncodingFormat {
  BC1 = "bc1",
  BC2 = "bc2",
  BC3 = "bc3",
  RGBA8 = "rgba8",
  BC7 = "bc7",
}

/** Every `ETextureEncodingFormat` as the spelling it crosses IPC as, for a value no member has narrowed. */
export type TextureEncodingFormat = `${ETextureEncodingFormat}`;

/** How hard the encoder works, which trades seconds for fidelity. */
export enum ETextureEncodingQuality {
  FAST = "fast",
  NORMAL = "normal",
  /** The default, because everything but BC7 costs pennies at it. */
  SLOW = "slow",
}

/** Every `ETextureEncodingQuality` as the spelling it crosses IPC as, for a value no member has narrowed. */
export type TextureEncodingQuality = `${ETextureEncodingQuality}`;

/** What one candidate cost and what it lost, weighed against the texture as it is now. */
export type TextureEncodingReport = {
  format: TextureEncodingFormat;
  /** The format's name with the DXT name the descriptor and the SDK use for it. */
  label: string;
  /** Bytes on disk, header included. */
  fileBytes: number;
  /** Bytes once uploaded, which is the whole mip chain without the header. */
  gpuBytes: number;
  encodeDuration: number;
  /** Peak signal-to-noise ratio, absent where nothing was lost. */
  psnr: number | null;
  /** Root mean square error per channel, in the eight-bit units the pixels are stored in. */
  channelRmse: [number | null, number | null, number | null, number | null];
  /** What the renderers together make of the format, as a sentence a badge can show. */
  supportSummary: string;
  compatibility: Array<TextureRendererSupport>;
};

/** The texture half of a save, which is one of the candidates a comparison already encoded. */
export type TextureEncodingSave = {
  sessionId: SessionId;
  target: TextureSaveTarget;
  format: TextureEncodingFormat;
};

/** One texture name and the files the roots hold for it. */
export type TextureEntry = {
  /**
   * What to call this row: an engine reference such as `ston\ston_beton05`, or a loose file's path below its root.
   *
   * Unique within one listing either way, so a tree can key on it. It is a label rather than an address; what to
   * open is `source`, because a loose file has no reference to be resolved back into.
   */
  reference: string;
  /** How to open this row, which is the address the describe and every write take. */
  source: TextureSource;
  role: TextureRole;
  /** The `.dds` the roots answer for this reference, winner first. */
  texture: XrayAsset | null;
  /** The `.thm` the roots answer for this reference, winner first. */
  descriptor: XrayAsset | null;
};

/**
 * What the editor read at a path, so a later write cannot overwrite a change it never saw.
 *
 * Size and modification time rather than a hash of the bytes: a texture is megabytes, the editor holds one node at a
 * time for minutes rather than days, and the case worth catching is an SDK or a converter having rewritten the file in
 * the meantime - which moves both.
 */
export type TextureFileStamp = {
  size: number;
  /** Milliseconds since the Unix epoch, as the platform reports the file's modification time. */
  modifiedMs: number;
};

/** One bit of the `STextureParams` flag word. */
export type TextureFlagEntry = {
  /** The bit, as a mask rather than an index, because the SDK's own bits are not contiguous. */
  bit: number;
  label: string;
};

/** What a generated pair came to. */
export type TextureMakeBumpOutcome = {
  /**
   * Whether the pair was written or the run stopped because it was asked to.
   *
   * A cancelled run wrote neither half: both are encoded before either is written, so there is no point at which
   * stopping could leave one half of a pair on disk with the other missing.
   */
  outcome: JobOutcome;
  /** The normals and gloss, written as `<name>_bump.dds`. */
  bump: string;
  /** The compression error and the height, written as `<name>_bump#.dds`. */
  companion: string;
  /** Mean gloss over the whole surface, in `0..=1`. */
  glossPower: number | null;
  /**
   * Whether the gloss is too dark for the surface to show a specular response worth having.
   *
   * A verdict rather than a failure, exactly as in the SDK: the pair is written either way, because a modder who
   * meant to author a matte surface is not making a mistake and one who did not wants to be told.
   */
  isGlossTooDark: boolean;
};

/** One descriptor's contribution to the tree: its badges, and the pair it names so both halves fold under it. */
export type TextureMaterialSummary = {
  /** The reference of the texture the descriptor describes. */
  reference: string;
  /** The pair the engine will try to bind, when the declaration is one it reads. */
  bump: TextureBumpPair | null;
  badges: TextureBadges;
};

/** One renderer's answer about a format. */
export type TextureRendererSupport = {
  renderer: string;
  /** `supported`, `unsupported`, or `unverified` - the third being a path nobody has read, not a refusal. */
  support: string;
};

/**
 * What a texture name is by convention, before any descriptor has been read.
 *
 * Read off the name so a tree can fold a pair under its texture the moment the listing arrives; which pairs are
 * declared, and by whom, is what the sweep then says. The convention itself is `xrf-material`'s, shared with the
 * renderer's fallback rule and the companion derivation.
 */
export enum ETextureRole {
  /** A texture a mesh or a level binds by name. */
  TEXTURE = "texture",
  /** The first half of a bump pair: packed normal and gloss. */
  BUMP = "bump",
  /** The second half of a bump pair: packed error and height. */
  BUMP_COMPANION = "bumpCompanion",
}

/** Every `ETextureRole` as the spelling it crosses IPC as, for a value no member has narrowed. */
export type TextureRole = `${ETextureRole}`;

/** What a save left on disk. */
export type TextureSaveOutcome = {
  /**
   * Whether the save wrote what it was asked to or stopped because it was asked to.
   *
   * A cancelled save wrote nothing. Cancellation is read once, after both files have been prepared and before either
   * is written, because there is no useful boundary inside two staged writes: stopping between them would leave a
   * texture whose descriptor still describes the old one, which is the state a save exists to avoid.
   */
  outcome: JobOutcome;
  /** The files written, in the order they were written. */
  written: Array<string>;
  /**
   * The format the descriptor ended up naming, when writing a texture changed it.
   *
   * Reported rather than left to the caller to infer, because the rule is the SDK's: `tfDXT1` and `tfADXT1` are one
   * encoder distinguished by the alpha flag, so only the descriptor's own flags can say which of them a BC1 texture
   * is. Absent when nothing synced - no texture was written, or its format has no name in `ETFormat`.
   */
  descriptorFormat: number | null;
};

/** One file a write addresses, and what was there when the editor read it. */
export type TextureSaveTarget = {
  /** Absolute path of the file to replace or create. */
  path: string;
  /** The stamp the editor read there, or `None` for a file it is creating. */
  expected: TextureFileStamp | null;
};

/** Every `kind` the `TextureSource` union is told apart by, so a switch or a comparison names one. */
export enum ETextureSource {
  /** A loose `.dds` or `.thm` on disk, named by its filesystem path. */
  FILE = "file",
  /** A texture of the roots, loose or archived, named by its engine reference such as `ston\ston_beton05`. */
  ASSET = "asset",
}

/** Where a texture is named from. */
export type TextureSource =
  /** A loose `.dds` or `.thm` on disk, named by its filesystem path. */
  | { kind: "file"; path: string }
  /** A texture of the roots, loose or archived, named by its engine reference such as `ston\ston_beton05`. */
  | { kind: "asset"; reference: string };

/**
 * Every named value the descriptor form's numeric fields can take.
 *
 * Answered once when the editor opens rather than carried on every description: it is the same table for every
 * texture in every root, and a description that repeated it would spend it thousands of times over a sweep.
 */
export type TextureVocabulary = {
  /** The gate `LoadTHM` reads before anything else. */
  textureTypes: Array<TextureVocabularyEntry>;
  formats: Array<TextureVocabularyEntry>;
  /** Fifteen values, of which fourteen are kernels and `Advanced` is the SDK's own chain. */
  mipFilters: Array<TextureVocabularyEntry>;
  materials: Array<TextureVocabularyEntry>;
  bumpModes: Array<TextureVocabularyEntry>;
  /** The twelve bits the SDK names, in bit order. A word may carry others, and those have no name to show. */
  flags: Array<TextureFlagEntry>;
  /**
   * The bump mode that makes the engine bind a pair.
   *
   * Named rather than left to a surface to recognise, because a tool that has just written a pair has to point the
   * descriptor at it and there is exactly one value that does. Matching on the display label would work until
   * somebody rewords it; matching on the number would work until it is spelled differently in two places.
   */
  bumpModeUse: number;
};

/** One value a descriptor field can take, under the name the SDK gives it. */
export type TextureVocabularyEntry = {
  /** The number stored in the file. */
  value: number;
  /** The SDK's own identifier for it, which is the name an author of a `.thm` would recognise. */
  label: string;
};

/** What a texture rebuild was asked to do. */
export type TexturesBuildRequest = {
  /** Path of the `.dds` to write, which is the file beside the descriptor. */
  destination: string;
  /** Path of the image to encode, of whatever kind `image` decodes. */
  source: string;
  /**
   * The descriptor to read as a recipe, as the editor currently has it rather than as it is on disk.
   *
   * The form rather than the file, because a build should produce what the editor's own format and flags describe. A
   * person who has changed the format and not saved yet wants to see that format built.
   */
  descriptor: TextureDescriptorForm;
  quality: TextureEncodingQuality;
};

/** What a format comparison was asked to weigh. */
export type TexturesCompareRequest = {
  sessionId: SessionId;
  /**
   * The texture to re-encode, named the way `describe` names one.
   *
   * A source rather than an engine reference, because a file outside every tree has no reference and is addressed by
   * its path. The label a surface shows still comes from the description; this is the address.
   */
  source: TextureSource;
  roots: XrayRoots;
  /**
   * Kernel the chain is reduced with, by its SDK name, or `None` to weigh the base level alone.
   *
   * Not read from the descriptor. A comparison answers "what would this texture cost in each format", and the answer
   * has to be about one chain built one way, or the figures are not comparable with each other.
   */
  mipFilter: string | null;
  quality: TextureEncodingQuality;
};

/**
 * What a bump pair generation was asked to do.
 *
 * The height source is required and everything else refines it, exactly as the SDK's generator has it: normals are
 * derived from the height alone and gloss is a separate plane, so a caller with only a height map still gets a pair.
 */
export type TexturesMakeBumpRequest = {
  /** Path of the texture the pair belongs to, without the `_bump` suffix or an extension. */
  destination: string;
  /** Path of the image the relief is read from, averaged across its colour channels. */
  height: string;
  /**
   * Path of a gloss mask, averaged the same way.
   *
   * When absent the whole surface takes [`Self::gloss_constant`], which is what a texture authored without a mask
   * needs and what the SDK's own dialog offers.
   */
  gloss: string | null;
  glossConstant: number | null;
  /** Path of a normal map to use instead of deriving one from the height, of the same size. */
  normalMap: string | null;
  /** `bump_virtual_height` of the descriptor, read here and nowhere at runtime. */
  virtualHeight: number | null;
  /**
   * Kernel the pair's chain is reduced with, by its SDK name.
   *
   * Defaults to `Box` at the caller, because that is what the SDK's generator leaves it at: `DXTCompressBump` builds
   * its `STextureParams` and overrides only the flags, the type and the format.
   */
  mipFilter: string | null;
  quality: TextureEncodingQuality;
};

/**
 * What one node's save was asked to write.
 *
 * Both halves are optional and independent: a node may be dirty in its descriptor, in its pixels, or in both, and a
 * save that could only do the pair would make the common case - a flag changed on a texture nobody re-encoded -
 * impossible to express.
 */
export type TexturesSaveRequest = {
  descriptor: TextureDescriptorSave | null;
  texture: TextureEncodingSave | null;
};

/**
 * What a build was asked to do.
 *
 * One argument rather than five, because a Tauri command's parameters are its wire signature and five of them plus a
 * job's own two is more than a reader can hold. It is also exactly what the registry retains, so a window adopting
 * this run after a reload sees the request rather than a summary of it.
 */
export type TranslationBuildRequest = {
  /** Where the sources are read from, through the VFS. */
  roots: XrayRoots;
  /** Where inside those roots to look, or nothing for the whole set. */
  prefix: string | null;
  /** The language to build, or `all`. */
  language: string;
  /** Directory the string tables are written into, which is always a host path. */
  outputDir: string;
  /** Whether to sort entries within each table. */
  isSorted: boolean;
};

/**
 * What a build reports back to the desktop surface.
 *
 * A row per language rather than the 272 files behind a full run, which is the natural grain of a
 * build whose job is one string table per language.
 */
export type TranslationBuildSummary = {
  /** Whether the run compiled every source or was stopped between them. */
  outcome: JobOutcome;
  /** The language built, or `all`. */
  language: string;
  /** Sources read. */
  sources: number;
  /** String tables written, across every language. */
  files: number;
  languages: Array<TranslationBuildLanguageSummary>;
};

/** One thing worth reporting about a file the run met. */
export type TranslationParseFinding = {
  rule: string;
  subject: string | null;
  message: string;
};

/**
 * What an import was asked to do.
 *
 * One argument rather than seven, because a Tauri command's parameters are its wire signature and seven of them plus
 * a job's own two is more than a reader can hold. It is also exactly what the registry retains, so a window adopting
 * this run after a reload sees the request rather than a summary of it.
 */
export type TranslationParseRequest = {
  /** Roots holding the raw XML, read through the VFS so an installation imports like a loose tree. */
  roots: XrayRoots;
  /** The language every entry this run reads is filed under. Never `all`. */
  language: string;
  /** Where inside those roots to look, or nothing to let the run resolve it. */
  prefix: string | null;
  /** Directory the JSON sources are written to, which may already hold some. */
  outputDir: string;
  /** Restrict the run to one table, by the file name it has in the scope. */
  file: string | null;
  /** Let incoming text replace existing text that differs, instead of keeping what is there. */
  isOverwrite: boolean;
  /** Do everything except write, so a caller can see what a run would change. */
  isDryRun: boolean;
};

/** What an import run reports back to the desktop surface. */
export type TranslationParseSummary = {
  /** Whether the run read every table or was stopped between them. */
  outcome: JobOutcome;
  /** The language every entry this run read was filed under. */
  language: string;
  /** Whether the run computed its answer without writing it. */
  isDryRun: boolean;
  census: TranslationParseCensus;
  findings: Array<TranslationParseFinding>;
};

/** Every `kind` the `TranslationSaveOutcome` union is told apart by, so a switch or a comparison names one. */
export enum ETranslationSaveOutcome {
  /** The edits are on disk, and this is the project as it now reads. */
  SAVED = "saved",
  /** The edits are on disk, but another project replaced this one while they were being written. */
  STALE = "stale",
}

/** How a save ended, once its edits were on disk. */
export type TranslationSaveOutcome =
  /** The edits are on disk, and this is the project as it now reads. */
  | { kind: "saved"; project: SessionSnapshot<TranslationProjectDescriptor> }
  /** The edits are on disk, but another project replaced this one while they were being written. */
  | { kind: "stale" };

/** What a completeness check reports back to the desktop surface. */
export type TranslationVerifySummary = {
  /**
   * Whether the run checked every source or was stopped between them.
   *
   * A stopped check reports the rows it reached; its silence about the rest is not a verdict.
   */
  outcome: JobOutcome;
  /** The language the check was narrowed to, or `all`. */
  language: string;
  /** Ids checked across every source. */
  checked: number;
  /** Ids with no text, counted once per language that lacks them. */
  missing: number;
  languages: Array<TranslationVerifyLanguageSummary>;
};

/** What a translation formatting run, or a check of one, was asked to do. */
export type TranslationsFormatRequest = {
  /** Project directory to format. */
  directory: string;
  /** Line endings to write, or nothing to keep what each file already uses. */
  lineEndings: string | null;
};

/** Inputs and caller-owned identity of one translations opening. */
export type TranslationsOpenRequest = {
  sessionId: SessionId;
  roots: XrayRoots;
  mode: TranslationProjectMode;
  prefix: string | null;
};

/** What a translation verification was asked to do. */
export type TranslationsVerifyRequest = {
  /** Trees to search, and how each is read. */
  roots: XrayRoots;
  /** Scope inside those trees, or nothing for all of them. */
  prefix: string | null;
  /** Language the check is about. */
  language: string;
};

/** Every `kind` the `VisualSource` union is told apart by, so a switch or a comparison names one. */
export enum EVisualSource {
  /** A loose `.ogf` file on disk, named by its filesystem path. */
  FILE = "file",
  /** An asset of the roots, loose or archived, named by its engine identity. */
  ASSET = "asset",
}

/**
 * Where a visual is read from.
 *
 * Both variants are self-describing, and neither is a handle into mount state: an asset is named by its engine
 * identity, which any surface can spell without having opened anything. The roots it is looked for in travels beside
 * the source on every command that takes one, so one call can never mix two roots.
 */
export type VisualSource =
  /** A loose `.ogf` file on disk, named by its filesystem path. */
  | { kind: "file"; path: string }
  /** An asset of the roots, loose or archived, named by its engine identity. */
  | { kind: "asset"; logicalPath: string };
