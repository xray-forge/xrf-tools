// Auto-generated rust bindings. Do not edit it manually.

import { AssetTextureShape, SessionId } from "#/types/xrf-app-core";
import { ArchiveProject } from "#/types/xrf-archive";
import { ArchivePackConfig, ArchivePatchConfig } from "#/types/xrf-pack";
import { ImageShape } from "#/types/xrf-texture";
import { XrayAssetContainer, XrayPathCollision, XrayRoots, XraySourceKind } from "#/types/xrf-vfs";

/** Comparing two trees without writing. */
export const ARCHIVES_JOB_COMPARE = "archives.compare" as const;

/** Extracting a directory of the open subject. */
export const ARCHIVES_JOB_EXTRACT = "archives.extract" as const;

/** Packing a tree into volumes. */
export const ARCHIVES_JOB_PACK = "archives.pack" as const;

/** Publishing a tree's difference as patch volumes. */
export const ARCHIVES_JOB_PATCH = "archives.patch" as const;

/** Unpacking a volume set whole. */
export const ARCHIVES_JOB_UNPACK = "archives.unpack" as const;

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
  /** One entry per channel, in the engine's channel order whatever order the file's version stores them in. */
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
  /** Pairings naming a parent pairing the editor takes their unset properties from. */
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

/** What a level's collision mesh weighs, from the 36 bytes that say so. */
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

/** Everything the viewer says about a level's render geometry. */
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

/** Everything the viewer says about a level's baked decals. */
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
  /**
   * The blender the sprite is drawn with, which names a definition of `shaders.xr` rather than a file; absent for an
   * effect that draws no sprite.
   */
  shader: string | null;
  /** The texture the sprite is drawn from, resolved against the subject being browsed; absent with the shader. */
  texture: ArchiveReference | null;
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
 * Each carries the roots it reads through, so a reader that needs a whole search rather than this subject's own reads
 * (the visuals viewer, resolving a model's textures and shaders) mounts the same arrangement without the page knowing
 * which kind is open.
 */
export type ArchiveSubject =
  /** The volumes at one path, merged into a single name table. */
  | {
      kind: "volumes";
      project: ArchiveProject;
      /** Every volume beneath the project's root, as the project was read. */
      roots: XrayRoots;
    }
  /** A game folder read as the engine mounts it, archives and loose tree together. */
  | {
      kind: "world";
      world: ArchiveWorld;
      /** How the world was opened, so a commit's follow lists it again over the same mounts. */
      roots: XrayRoots;
    };

/** The bump declaration of a descriptor, `THM_CHUNK_BUMP`. */
export type ArchiveThmBump = {
  modeLabel: string;
  mode: number;
  /** Height the generator builds the pair against, read at generation time and never at runtime. */
  virtualHeight: number | null;
  /** The bump texture named, absent when the chunk names none. */
  texture: ArchiveReference | null;
  /** Whether the engine would try to resolve the name: a mode that uses one, and a name to use. */
  isUsed: boolean;
};

/** A declared size the texture beside the descriptor does not match. */
export type ArchiveThmDeclaredSize = {
  width: number;
  height: number;
  /** Whether the declaration is a cube map's source strip: six faces wide, one face tall. */
  isCubeStrip: boolean;
};

/** Everything the viewer says about one texture descriptor. */
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
  /** The flags that switch this association on, by the SDK's own spelling. */
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
  /** Chunk ids holding bytes past their fields, which no engine reads, in the order the reader met them. */
  trailingChunks: Array<number>;
};

/** One bit of the texture param flag word. */
export type ArchiveThmFlag = {
  /** SDK identifier, which is the name an author of a `.thm` would recognise. */
  label: string;
  isSet: boolean;
};

/** The shading declaration of a descriptor, `THM_CHUNK_MATERIAL`. */
export type ArchiveThmMaterial = {
  /** The two lighting models the surface sits between. */
  label: string;
  value: number;
  /** Where between them it sits. */
  weight: number | null;
};

/** The conversion parameters a descriptor carries, `THM_CHUNK_TEXTUREPARAM`. */
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
  /** Every bit the SDK names, in bit order, set or not. */
  flags: Array<ArchiveThmFlag>;
  /** Bits the word carries that the SDK has no name for. */
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
  /** Whether `CTextureDescrMngr::LoadTHM` reads the bump, detail and material of a descriptor of this type at all. */
  isReadByEngine: boolean;
  /** Whether the file declares a type, or the engine's zeroed default is what applies. */
  isDeclared: boolean;
};

/** The preview picture a descriptor carries, `THM_CHUNK_DATA`. */
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

/** One mounted world the explorer browses: an installation, or any tree read as the engine would read it. */
export type ArchiveWorld = {
  /** Sources searched, highest priority first, as each one names itself. */
  mounts: Array<string>;
  /** Winning entries, one per engine path, ordered by that path. */
  files: Array<ArchiveWorldEntry>;
  /** Unpacked bytes of the winning entries. What is shadowed is not counted; it is not what the engine would load. */
  sizeReal: number;
  /** Engine paths this world answers with more than one copy for. */
  shadowedCount: number;
  /** Unpacked bytes held by the copies no lookup reaches. */
  shadowedSizeReal: number;
};

/** One file of a mounted world, as the explorer lists it. */
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

/** Directory extraction request for whichever subject the explorer has open. */
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

/** What a sound is, once it has been located. */
export type AudioDescriptor = {
  /** Absent when the bytes carry no readable stream header. */
  channels: number | null;
  /** Absent when the bytes carry no readable stream header. */
  sampleRate: number | null;
  /** Absent for a sound carrying no recognized X-Ray comment, where the engine uses its own defaults. */
  parameters: AudioSourceParameters | null;
  /** What to hand the bytes over as, taken from the extension rather than from the content. */
  mediaType: string;
};

/** The X-Ray source parameters carried in a sound's first vorbis comment. */
export type AudioSourceParameters = {
  minDistance: number | null;
  maxDistance: number | null;
  baseVolume: number | null;
  gameType: number;
  maxAiDistance: number | null;
};

/** What a picture the webview renders itself turns out to be. */
export type ImageDescriptor = {
  /** Absent when the bytes carry no header this reads, which is a picture worth neither drawing nor measuring. */
  shape: ImageShape | null;
  /** What to hand the bytes over as, taken from the extension rather than from the content. */
  mediaType: string;
};
