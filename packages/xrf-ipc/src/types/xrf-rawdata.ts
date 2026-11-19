// Auto-generated rust bindings. Do not edit it manually.

/** What a build took from the scene and left out. */
export type LevelBuildCensus = {
  /** Static objects whose meshes went into the geometry. */
  staticObjects: number;
  /** Objects of no static geometry: dynamic ones, and occluders, which `level.hom` and `level.som` take. */
  otherObjects: number;
  faces: number;
  /** Faces of occluder, breakable and climbable surfaces, which other exports take. */
  skippedFaces: number;
  /**
   * Faces of no area left out: the objects' faces, which the Build drops, and the wall marks' triangles, which xrLC
   * would stop on.
   */
  invalidFaces: number;
  staticLights: number;
  dynamicLights: number;
  glows: number;
  portals: number;
  occluderPolygons: number;
  soundOccluderPolygons: number;
  /** Multi-use models and the placements referencing them. */
  multiUseModels: number;
  multiUseReferences: number;
  /** Multi-use models drawn from afar as impostors. */
  impostors: number;
  aiMapNodes: number;
  /** Detail objects the slots plant, and the textures their atlas merges. */
  detailObjects: number;
  detailTextures: number;
  /** Breakables and ladders made of breakable and climbable faces, and parts of either refused. */
  breakableObjects: number;
  climbableObjects: number;
  refusedParts: number;
};

/** How large a build lets the impostor atlas, `level_lods.dds` and `_nm`, grow. */
export enum ELevelBuildImpostorLimit {
  /** SDK 0.7's, 2048 a side or 128 strips of 512 by 64; a level of more is refused, as that Level Editor refuses it. */
  SDK = "sdk",
  /**
   * 4096 a side or 512 strips, as Anomaly's and GAMMA's editors grew it: their `jupiter` and `pripyat` ship a 4096 by
   * 2048 atlas of 130 to 153 strips.
   */
  EXTENDED = "extended",
}

/** Every `ELevelBuildImpostorLimit` as the spelling it crosses IPC as, for a value no member has narrowed. */
export type LevelBuildImpostorLimit = `${ELevelBuildImpostorLimit}`;

/** What [`crate::LevelBuildExporter::build_to_directory`] wrote, for the front-ends to report. */
export type LevelBuildReport = {
  source: string;
  destination: string;
  /** Each file written, its size and what it replaced. */
  files: Array<LevelExportedFile>;
  /** Files an earlier build wrote that this one did not, removed, by path in the directory. */
  removed: Array<string>;
  /** What `build.prj` took from the scene, where the build wrote one. */
  geometry: LevelBuildCensus | null;
  /** The parts the build made nothing of, and why. */
  refused: Array<LevelSceneRefusal>;
};

/**
 * A front-end's request to build a scene into a level directory: the game files always, and the level compiler's
 * files where the game material library is named.
 */
export type LevelBuildRequest = {
  /** The scene's main file, `<name>.level`, its parts in the directory beside it named after it. */
  source: string;
  /** The level directory the files are written into. */
  destination: string;
  /**
   * The objects roots (`$objects$`), each object read from the first holding it; empty for the default
   * ([`Self::get_objects_roots`]).
   */
  objects: Array<string>;
  /** The game material library, `gamemtl.xr`; the level compiler's files are written only where it is named. */
  gameMaterials: string | null;
  /**
   * The textures root (`$game_textures$`) of the details' textures and the impostors' strips; none for the default
   * ([`Self::get_textures_root`]).
   */
  textures: string | null;
  impostorLimit: LevelBuildImpostorLimit;
  /**
   * Whether the build replaces files the directory holds and removes its own stale ones; unforced, it refuses a
   * directory holding a file it owns before reading the scene.
   */
  isForced: boolean;
};

/** The static geometry a decompile wrote, counted. */
export type LevelDecompileGeometry = {
  /** The objects written, one per sector drawing static geometry. */
  objects: number;
  /** How the surfaces were dressed, where they were inferred; the sidecar lists each compiler shader chosen. */
  surfaces: LevelDecompileSurfaces | null;
} & LevelGeometryCensus;

/**
 * What a decompile into a scene on disk wrote ([`crate::LevelSceneDecompiler::decompile_to_path`]), as the front-ends
 * report it.
 */
export type LevelDecompileReport = {
  source: string;
  destination: string;
  sidecar: string;
  /** The compiled files the scene came from. */
  sources: Array<string>;
  tools: Array<LevelDecompileReportTool>;
  inferred: Array<LevelSceneInference>;
  /** What the scene leaves out, such as an entity of a class the decompile cannot type. */
  refused: Array<LevelSceneRefusal>;
  /** The root spawn read, where one was; the sidecar lists what is counted here. */
  rootSpawn: LevelDecompileRootSpawn | null;
  /** The objects root the scene's objects were written under. */
  objectsRoot: string;
  /** The game textures root the objects' textures were written under, and how many. */
  texturesRoot: string;
  textures: number;
  /** The parts and objects of the level's own an earlier decompile wrote and this forced one does not, removed. */
  removed: Array<string>;
  /** The static geometry, counted, where the level held `level.geom`; the sidecar lists each object. */
  geometry: LevelDecompileGeometry | null;
};

/** One tool of the scene a decompile wrote, and what its part holds. */
export type LevelDecompileReportTool = {
  name: string;
  /** The objects its text part holds; none for a binary part. */
  objects: number | null;
};

/**
 * A front-end's request to decompile a level into a scene on disk; [`Self::get_options`] and the root getters resolve
 * the defaults as the SDK lays out a level and its rawdata.
 */
export type LevelDecompileRequest = {
  /** The compiled level's directory, `<gamedata>/levels/<level>`. */
  source: string;
  /** The scene's main file, `<rawdata>/levels/<name>.level`; its name is the level's. */
  destination: string;
  /** The objects root the scene's objects are written under; `<rawdata>/objects` beside the scene by default. */
  objects: string | null;
  /** The game textures root the objects' textures are written under; `<rawdata>/../gamedata/textures` by default. */
  textures: string | null;
  /**
   * The root spawn whose objects stand in for `level.spawn`'s entities, by name in the level's `gamedata/spawns`;
   * `all` where that exists by default.
   */
  rootSpawn: string | null;
  surfaces: LevelSurfaces;
  /** The game material library; the level's gamedata's `gamemtl.xr` where it exists by default. */
  gameMaterials: string | null;
  /** The compiler shader library; the level's gamedata's `shaders_xrlc.xr` where it exists by default. */
  compilerShaders: string | null;
  /**
   * Object library roots, such as an SDK's `rawdata/objects`: the static geometry's placed objects, its trees and the
   * level's detail models are looked up among their objects, and their surfaces break ties between compiler shaders.
   */
  libraries: Array<string>;
  /**
   * Whether existing files are overwritten and an earlier decompile's stale parts and objects removed. Unforced, an
   * existing scene or level object is refused before the level is read.
   */
  isForced: boolean;
};

/** The root spawn a decompile read, counted. */
export type LevelDecompileRootSpawn = {
  file: string;
  /** Its objects on the level, which the scene's entities came from; none where it does not hold the level. */
  objects: number;
  disagreements: number;
  updatePackets: number;
};

/** The surfaces a decompile inferred, counted. */
export type LevelDecompileSurfaces = {
  /** Triangles on a collision face of the same three points. */
  exactTriangles: number;
  /** Triangles a short ray found a collision face beneath. */
  tracedTriangles: number;
  /** Triangles on no collision face. */
  uncollidedTriangles: number;
  /** Compiler shaders chosen, one per engine shader and kind of evidence. */
  compilerShaders: number;
  /** Collision materials the game material library does not define. */
  unknownGameMaterials: number;
};

/** One level file an export wrote. */
export type LevelExportedFile = {
  name: string;
  size: number;
  previous: LevelExportedPrevious;
};

/**
 * What a file [`LevelSceneExporter::write_to_directory`](crate::parts::LevelSceneExporter::write_to_directory) wrote
 * replaced.
 */
export enum ELevelExportedPrevious {
  /** The directory held no such file. */
  NEW = "new",
  /** The directory held the same bytes. */
  SAME = "same",
  /** The directory held other bytes, now replaced. */
  CHANGED = "changed",
}

/** Every `ELevelExportedPrevious` as the spelling it crosses IPC as, for a value no member has narrowed. */
export type LevelExportedPrevious = `${ELevelExportedPrevious}`;

/**
 * What the static geometry's decompile counts, so every triangle the compiled level draws is either in a written
 * object or accounted for.
 */
export type LevelGeometryCensus = {
  /** Full-detail triangles of the sectors' static visuals, read. */
  staticTriangles: number;
  /** Faces the written objects hold. */
  faces: number;
  /** Faces of those that are two-sided, each folded from a triangle and its mirror. */
  twoSidedFaces: number;
  /** Triangles dropped as an exact repeat of another of their surface. */
  repeatedTriangles: number;
  /** Full-detail triangles of tree instances, every placement's counted; each model holds one placement's. */
  treeTriangles: number;
  /** Tree models found to be library trees, which their placements reference instead. */
  libraryTreeModels?: number;
  /** Multi-use models the trees come back as, and their placements. */
  treeModels?: number;
  treePlacements?: number;
  /**
   * Library objects found placed in the static geometry, the distinct objects they place, and the static triangles
   * their placements took out of the shells, counted in the static triangles read.
   */
  instances?: number;
  instanceObjects?: number;
  instanceTriangles?: number;
  /** Impostors standing for trees afar, which the build makes again from the trees. */
  impostors: number;
  /** Tree models given their impostor strips back, cut from the level's atlas; a library tree's named for it. */
  treeImpostors?: number;
  /**
   * Static triangles dropped as copies of wall marks ([`LevelDecalIndex`](crate::parts::LevelDecalIndex)), counted
   * in the static triangles read.
   */
  decalTriangles: number;
  /**
   * Static and collision-only triangles dropped as needles, which xrLC stops on; the static ones are counted in the
   * static triangles read.
   */
  needleTriangles?: number;
  /** Sectors drawing no static geometry, such as one of trees alone, which no shell stands for. */
  emptySectors: number;
  /** Faces of the collision form, where the level's directory held one. */
  collisionFaces: number;
  /** Collision faces only a tree's triangles lie on: trunks, which come back with their trees. */
  treeCollisionFaces: number;
  /** Collision faces only a placed library object's collision-only triangles lie on, which come back with it. */
  instanceCollisionFaces?: number;
  /** Collision faces no render triangle lies on, which the collision object holds. */
  collisionOnlyFaces: number;
  /** Polygons of `level.hom` and `level.som`, which `<level>_hom` and `<level>_som` hold. */
  occluderPolygons: number;
  soundOccluderPolygons: number;
  /** Sound occlusion factors no game material of the library has, whose polygons take `default`. */
  unmatchedSoundOcclusionFactors: number;
  /** Ladders folded back into objects, and those of them whose packet kept no game material. */
  ladders?: number;
  laddersOfDefaultMaterial?: number;
  /**
   * Breakables folded back into objects, and those whose part the level's directory does not hold, which stay
   * entities.
   */
  breakables?: number;
  breakablesWithoutPart?: number;
};

/**
 * What a decompiled scene holds that its sources do not say, which the decompile chose; a part's inferences are
 * recorded only where it holds what they were chosen for.
 */
export enum ELevelSceneInference {
  /** Objects are named as the editor names new ones, `<prefix>_NNNN`, the prefix their file's name or their tool's. */
  OBJECT_NAMES = "objectNames",
  /**
   * The level options are a new level's, built into and prefixed by the level's name, but for the light qualities,
   * which the count of `build.lights`' generated sky and sun gives.
   */
  LEVEL_OPTIONS = "levelOptions",
  /**
   * The level serves the game modes its respawn points answer to, or single player where it has none; made only where
   * no `level.ltx` names them.
   */
  GAME_TYPES = "gameTypes",
  /** The build parameters are the editor's defaults but for a weld distance of 0, for geometry already welded. */
  BUILD_PARAMETERS = "buildParameters",
  /** The camera stands at the origin, unturned. */
  CAMERA = "camera",
  /** Sound sources take a new source's distances, which the editor reads again from each sound file. */
  SOUND_SOURCE_DISTANCES = "soundSourceDistances",
  /** The wall mark tool takes a new tool's brush, and each decal the brush's size and turn. */
  WALLMARK_BRUSH = "wallmarkBrush",
  /** Glows stand unturned with no fixed-size flag, which the compiled level does not keep. */
  GLOW_TRANSFORMS_AND_FLAGS = "glowTransformsAndFlags",
  /**
   * Each light's brightness is the channel of its diffuse of the largest magnitude, sign kept, and its colour the rest;
   * the compiled level keeps only their product.
   */
  LIGHT_BRIGHTNESS = "lightBrightness",
  /** A fuzzy light comes back as one light per point it was spread over. */
  LIGHT_FUZZY_POINTS = "lightFuzzyPoints",
  /** A light only dynamic answers to `$static`; the compiled level keeps no control's name. */
  LIGHT_CONTROLS = "lightControls",
  /**
   * An entity's volume comes back as a shape attached to it at its position, unturned and unscaled, under a generated
   * name; the spawn list keeps only the shapes.
   */
  ENTITY_SHAPES = "entityShapes",
  /**
   * The static geometry, which the compiled level merged per sector, comes back as one object per sector, named after
   * it and placed at the origin; sectors take a new sector's colour, and sectors and portals the editor's new names.
   */
  SECTOR_SHELLS = "sectorShells",
  /**
   * Each static surface takes the game material of the collision faces under it, split where those differ, and a
   * `shaders_xrlc.xr` compiler shader fitting what the level shows, as the sidecar's `geometry.surfaces` records.
   */
  SURFACE_DRESS = "surfaceDress",
  /**
   * The collision faces no render triangle lies on come back as one object, `<level>_collision`, a mesh per sector,
   * dressed as GSC dressed its collision-only props (`prop\prop_fake_kollision`, `def_shaders\def_trans_v`,
   * `def_shaders\def_kolizion_vertex`).
   */
  COLLISION_OBJECT = "collisionObject",
  /**
   * `level.hom` comes back as `<level>_hom`, a surface of `materials\occ` as GSC's `labx8` keeps its occluders, and
   * `level.som` as `<level>_som`, flagged a sound occluder, each polygon's game material the library's of the lowest
   * number whose sound occlusion factor is the polygon's; both dressed with `prop\prop_fake_occ`.
   */
  OCCLUDERS = "occluders",
  /**
   * Each ladder (`climable_object`) comes back as an object, `<level>_ladder_<N>`, of its box less the back face,
   * standing at the middle of its lowest end and dressed as GSC's ladders (`prop\prop_fake_ladder`,
   * `def_shaders\def_trans_v`, `def_shaders\def_kolizion_vertex`, `materials\fake_ladders` where the packet keeps no
   * game material). Its packet is no spawn point, since the Build exports it again; the authored faces are lost.
   */
  LADDERS = "ladders",
  /**
   * Each breakable part (`meshes\brkbl#<N>.ogf`) comes back as a static object, `<level>_breakable_<N>`, where its
   * `breakable_object` placed it, its fragments' bones dropped and each surface of its fragments' game material. Its
   * packet is no spawn point, since the Build exports it again, cut into new random fragments.
   */
  BREAKABLE_PARTS = "breakableParts",
  /**
   * Library objects found in the static geometry come back as scene objects referencing them at their best-fitting
   * transform (one scale on every axis), their triangles taken out of the shells, each copied beside the scene's
   * objects as the sidecar's `geometry.instances` records. Of two drawing the same geometry, the larger stands for
   * both, then the one whose reference sorts first.
   */
  LIBRARY_INSTANCES = "libraryInstances",
  /**
   * Trees come back as multi-use objects, `<level>_tree_<N>`, one per set of subdivisions, dressed as their first
   * placement lies on the collision form, with a scene object per placement. Impostor strips are cut from the level's
   * atlas as `lod\lod_<model>` and its `_nm`, the model flagged `eoUsingLOD`; a model of no strip draws no impostor
   * after a rebuild.
   */
  TREE_MODELS = "treeModels",
  /**
   * Each detail model comes back as the library object drawing its geometry (the first by path), at the level's scale
   * range and flags and a density factor of 1; the tool takes a density of 0.3, no base texture, colour indices or
   * snap objects, and each slot's light is zeroed, as the Build zeroes it before lighting.
   */
  DETAIL_OBJECTS = "detailObjects",
  /**
   * The AI map's nodes keep their cells, heights, planes and links, packed against a box of the map's floor and
   * height; its generator settings but the patch size, and the editor's own settings, are a new tool's, since
   * `level.ai` keeps none.
   */
  AI_MAP_SETTINGS = "aiMapSettings",
  /**
   * The map's rectangle (`level.ltx`'s `bound_rect`) comes back as a level-bound shape, `level_bound_NNNN`, a box over
   * it one metre tall around height 0; the file keeps only its x and z.
   */
  MAP_BOUND = "mapBound",
  /**
   * The `level.ltx` sections the Build does not write itself (a multiplayer map's anomaly sets) come back as the build
   * options' text, which the Build writes back, without their blank and `;` lines.
   */
  BUILD_OPTIONS = "buildOptions",
}

/** Every `ELevelSceneInference` as the spelling it crosses IPC as, for a value no member has narrowed. */
export type LevelSceneInference = `${ELevelSceneInference}`;

/** What the decompile read but left out of the scene, with why. */
export type LevelSceneRefusal = {
  /** The compiled file holding it, such as `level.spawn`, or an object library's file that could not be used. */
  file: string;
  /** Its name there, such as an entity's, or the library file's name. */
  object: string;
  reason: string;
};

/** How a level's decompile dresses its surfaces: from what the level shows, or with the editor's defaults. */
export enum ELevelSurfaces {
  /** Each face's game material from the collision face it lies on, and a compiler shader fitting what the level shows. */
  INFER = "infer",
  /** Every surface takes `default` for both, as a new surface in the Actor Editor does. */
  DEFAULTS = "defaults",
}

/** Every `ELevelSurfaces` as the spelling it crosses IPC as, for a value no member has narrowed. */
export type LevelSurfaces = `${ELevelSurfaces}`;

/** Something a decompiled object holds that its OGF did not store, so the decompiler inferred or defaulted it. */
export enum EObjectInference {
  /** Surfaces are named after their texture; the authored names are not exported. */
  SURFACE_NAMES = "surfaceNames",
  /** Every surface's compiler shader is the Actor Editor's `default`. */
  SURFACE_COMPILER_SHADERS = "surfaceCompilerShaders",
  /** Every surface's game material is the Actor Editor's `default`. */
  SURFACE_GAME_MATERIALS = "surfaceGameMaterials",
  /**
   * The mesh is named `skin`, as the SDK names a skinned object's mesh, or `mesh` in a static object, whose mesh the
   * SDK names after the object.
   */
  MESH_NAME = "meshName",
  /** Every bone is the SDK's default 0.5 long; an OGF stores no length. */
  BONE_LENGTHS = "boneLengths",
  /** Smoothing edge flags come from where welded corners' normals agree. */
  SMOOTHING = "smoothing",
  /** A surface is two-sided where its faces came in mirrored pairs. */
  TWO_SIDED_SURFACES = "twoSidedSurfaces",
  /**
   * The object's own motions are keyed at every frame, linearly between them: the OGF keeps the sampled poses, not
   * the authored curves or which bones kept their world orientation.
   */
  MOTION_CURVES = "motionCurves",
  /**
   * A motion holding one key per bone more than it declares is as long as its keys: `blender-xray` declares one frame
   * fewer than it samples for an action starting between frames, which the engine misreads.
   */
  MOTION_LENGTHS = "motionLengths",
  /**
   * The mesh held one face, which the Level Editor refuses to load (it builds a picking model of every mesh and asserts
   * 4 points and 2 faces), so it comes back halved through its longest edge: the same area in two faces.
   */
  HALVED_FACE = "halvedFace",
}

/** Every `EObjectInference` as the spelling it crosses IPC as, for a value no member has narrowed. */
export type ObjectInference = `${EObjectInference}`;
