// Auto-generated rust bindings. Do not edit it manually.

import { Vector3d } from "@/core/ipc/types/xrf-math";
import { XrayResolution } from "@/core/ipc/types/xrf-vfs";

/**
 * Everything about a level's packed grass except the bytes themselves.
 *
 * The slots and the collision triangles stay in the engine's own space, because the renderer plants them with the
 * engine's own arithmetic and only converts what it planted.
 */
export type DetailsDescription = {
  /**
   * The grid's size in slots, and the world slot its first cell stands for, negated: cell `(x, z)` is world slot
   * `(x - offset_x, z - offset_z)`.
   */
  sizeX: number;
  sizeZ: number;
  offsetX: number;
  offsetZ: number;
  models: Array<DetailsModel>;
  /** One `u32` a cell, `z * size_x + x`: the planted slot's record plus one, zero for a cell with nothing to plant. */
  grid: VisualSection;
  /**
   * Eight `u32` a planted slot: its stored sixteen bytes as four words, the first entry of its triangle bin, the
   * bin's length, and its world slot's `x` and `z`.
   */
  slots: VisualSection;
  slotCount: number;
  /** One `u32` an entry: the triangle, by index into the triangles. */
  bins: VisualSection;
  binLength: number;
  /** Nine floats a triangle: its corners in the engine's space and winding, passable ones left out. */
  triangles: VisualSection;
  triangleCount: number;
  bufferLength: number;
};

/** One detail model of a level's library, packed for the renderer to plant. */
export type DetailsModel = {
  /** The shader and texture it is dressed with, as the library names them. */
  shader: string;
  texture: string;
  /** Whether the wind moves it: no `DO_NO_WAVING` flag. */
  isWaving: boolean;
  /** The scale range it is planted at, before the engine narrows it to half the least and nine tenths the most. */
  minScale: number | null;
  maxScale: number | null;
  /** Its bounding box's height, which a waving vertex's share of the sway is measured against. */
  height: number | null;
  /** The radius of the sphere around its bounding box, which its distance cull is measured by. */
  radius: number | null;
  vertexCount: number;
  indexCount: number;
  /** Three floats a vertex, in renderer space. */
  positions: VisualSection;
  /** Two floats a vertex. */
  uvs: VisualSection;
  /** Sixteen-bit indices, wound for renderer space. */
  indices: VisualSection;
};

/** Everything about a packed sector except the bytes themselves. */
export type SectorDescription = {
  /** The sector packed, by its index in the sectors chunk. */
  sector: number;
  /** Everything the level bakes in place, packed onto one vertex array and drawn section by section. */
  geometry: SectorGeometry;
  sections: Array<SectorSection>;
  /** Meshes the sector draws many times over, each packed once with the places it stands. */
  instances: Array<SectorInstanceGroup>;
  /** What its clumps of trees draw as from far enough away, absent for a sector with no `MT_LOD` visual. */
  impostors: SectorImpostors | null;
  /** Drawables that produced no geometry, which is none for every level measured. */
  skipped: Array<SectorSkip>;
  /** Extent the packed vertices span, absent when the sector packed none. */
  bounds: VisualBounds | null;
  bufferLength: number;
};

/**
 * Where one packed mesh's attributes sit inside a sector's buffer, and what to draw from them: positions as floats
 * in renderer space, and the rest in the 32-byte vertex xrLC wrote, byte for byte but for a direction's z.
 */
export type SectorGeometry = {
  vertexCount: number;
  indexCount: number;
  /** Three floats a vertex, in renderer space. */
  positions: VisualSection;
  /**
   * Four bytes a vertex: the normal's z, y and x as `D3DCOLOR` stores them, z negated into renderer space, then the
   * hemisphere term.
   */
  normals: VisualSection | null;
  /** Four bytes a vertex the same way: the authored tangent, then the low byte of the base `u`. */
  tangents: VisualSection | null;
  /** Four bytes a vertex the same way: the authored binormal, then the low byte of the base `v`. */
  binormals: VisualSection | null;
  /** The base coordinate as xrLC quantised it, as many shorts a vertex as its components say. */
  uvs: VisualSection | null;
  /**
   * Shorts a base coordinate takes a vertex: two (`SHORT2`, over 1024 with its low bytes), or four for a tree
   * (`SHORT4`, over 2048, then its wind terms). Zero where there are no base coordinates.
   */
  uvComponents: number;
  /** The lightmap coordinate: two shorts a vertex, over 32768. */
  lightmapUvs: VisualSection | null;
  /** Every index, as 32-bit elements: a sector reaches past what sixteen bits address. */
  indices: VisualSection;
};

/** A run of a sector's impostors dressed by one surface. */
export type SectorImpostorGroup = {
  surface: SectorSurface;
  /** The first impostor of the run. */
  start: number;
  count: number;
};

/**
 * The impostors of a sector's `MT_LOD` visuals: what the engine draws in place of a clump of trees seen from far
 * enough away, each eight facets looking at it from eight sides, in renderer space.
 */
export type SectorImpostors = {
  count: number;
  /** Runs of impostors a surface each, in impostor order. */
  groups: Array<SectorImpostorGroup>;
  /** Four floats an impostor: its visual's sphere, centre then radius. */
  spheres: VisualSection;
  /** One float an impostor: `FLOD::lod_factor`, what its sphere's screen area is scaled by. */
  factors: VisualSection;
  /**
   * Eight floats for each of an impostor's 32 corners, facet by facet: the position, the atlas `u` and `v`, then the
   * hemisphere and sun terms as the bytes it stores them over 255, then nothing.
   */
  corners: VisualSection;
  /** Four floats for each of an impostor's eight facets: its normal, then nothing. */
  normals: VisualSection;
};

/** One mesh a sector draws many times, packed once with the places it stands. */
export type SectorInstanceGroup = {
  surface: SectorSurface;
  /** The drawables this group stands in for, by their index in the visuals run. */
  drawables: Array<number>;
  /** The mesh itself, in its own space, its indices counting from its own first vertex. */
  geometry: SectorGeometry;
  instanceCount: number;
  /** Sixteen floats for each instance, exactly as the engine stores a matrix. */
  transforms: VisualSection;
  /** Two floats for each instance: what scales and then offsets its vertices' hemisphere term. */
  hemi: VisualSection;
  /**
   * One signed integer for each instance: the sector's impostor standing in for the clump it belongs to, or -1.
   * Absent where no place of the group belongs to one.
   */
  impostors: VisualSection | null;
  /**
   * The bands a progressive tree's places pick among, over the whole of its windows' indices; absent for a mesh of
   * one detail, whose indices are its one window.
   */
  progressive: SectorProgressive | null;
};

/** What one sector is and where it sits, before any of its geometry is read. */
export type SectorOutline = {
  /** The sector, by its index in the sectors chunk. */
  sector: number;
  /** The visual the sector names, which is the root its drawables are reached through. */
  root: number;
  /** Drawables the root reaches, which is what packing the sector would pack. */
  drawables: number;
  /** Extent the sector declares, absent when it reaches no drawable. */
  bounds: VisualBounds | null;
};

/** The bands a progressive mesh is drawn in: a few of the engine's slide windows, one of which each place draws. */
export type SectorProgressive = {
  /** Windows the engine's table has, which a place's detail picks among (`FTreeVisual_PM::Render`). */
  windows: number;
  /**
   * A range of the mesh's indices per band, the whole detail first. Band `b` is window `floor(b * windows / bands)`,
   * so a place drawing the band its window falls in is never coarser than the engine would draw it.
   */
  bands: Array<VisualDrawRange>;
};

/** One draw of a sector's own geometry: the indices to draw, and the surface they are drawn with. */
export type SectorSection = {
  surface: SectorSurface;
  /** Drawables packed into this section, by their index in the visuals run. */
  drawables: Array<number>;
  draw: VisualDrawRange;
  /** Extent its own vertices span, which it is culled by; absent when it reaches none. */
  bounds: VisualBounds | null;
};

/** A drawable of a sector that produced no geometry, and why. */
export type SectorSkip = {
  /** The visual left out, by its index in the visuals run. */
  drawable: number;
  cause: VisualSkipCause;
  reason: string;
};

/** How one part of a sector is dressed, as the level's shader table names it. */
export type SectorSurface = {
  /** Entry of the level's shader table this is dressed by. */
  shaderId: number;
  /** The engine shader that entry names, absent when the level carries no table. */
  shaderName: string | null;
  /** The base texture that entry names, absent for the same reason. */
  textureName: string | null;
  /** The one the deferred renderer binds as `s_hemi`, out of which it reads hemisphere and sun occlusion. */
  hemi: string | null;
};

/** One bone of a visual's skeleton, as a name and the name of its parent. */
export type VisualBone = {
  name: string;
  parent: string;
  /** Index of the parent in this same list, or `None` for a root or a parent no bone carries. */
  parentIndex: number | null;
  /** The bone's whole bind transform in model space, or `None` when the file carries no IK chunk. */
  bindTransform: VisualTransform | null;
};

/** A visual's extent, as a box and a sphere. */
export type VisualBounds = {
  boundingBox: VisualBox;
  boundingSphere: VisualSphere;
};

/** Axis aligned box in three.js space. */
export type VisualBox = {
  min: Vector3d;
  max: Vector3d;
};

/** Everything a visual needs from outside itself, resolved. */
export type VisualDependencies = {
  textures: Array<VisualTextureDependency>;
  motions: Array<VisualMotionDependency>;
};

/** Everything about a packed visual except the bytes themselves. */
export type VisualDescription = {
  version: number;
  modelType: number;
  modelTypeLabel: string;
  shaderId: number;
  /** Source object the OGF was built from, when the file records one. */
  sourceFile: string | null;
  /** Extent the header declares, converted into three.js space for comparison with the computed extent. */
  declaredBounds: VisualBounds;
  /** Extent the packed geometry actually spans, absent when no submesh produced any. */
  computedBounds: VisualBounds | null;
  submeshes: Array<VisualSubmesh>;
  bones: Array<VisualBone>;
  /** Logical paths of the omf files this visual animates from. */
  motionRefs: Array<string>;
  /** Names of motions stored inside the visual itself, for a self animated model. */
  embeddedMotions: Array<string>;
  bufferLength: number;
};

/** The slice of an index buffer that draws one detail level. */
export type VisualDrawRange = {
  start: number;
  count: number;
};

/** Where one submesh's attributes sit inside the geometry buffer, and what to draw from them. */
export type VisualGeometry = {
  vertexCount: number;
  indexCount: number;
  positions: VisualSection;
  normals: VisualSection;
  /** The authored tangent of every vertex, mirrored with the normal. */
  tangents: VisualSection;
  /** The authored binormal of every vertex, mirrored with the normal; see [`Self::tangents`]. */
  binormals: VisualSection;
  uvs: VisualSection;
  indices: VisualSection;
  /** Skinning links, or `None` for geometry that carries none and is therefore drawn as it is stored. */
  skin: VisualSkin | null;
  /** Every range a consumer may draw, finest first, and never empty. */
  detailLevels: Array<VisualDrawRange>;
  bounds: VisualBounds;
};

/** What one baked motion is, beside the frames themselves. */
export type VisualMotionBake = {
  name: string;
  /** Frames the buffer holds: the longest key stream the payload carries, not the count the motion declares. */
  frameCount: number;
  boneCount: number;
  /** Seconds playing the motion takes: its frames at the format's sample rate, over its playback speed. */
  duration: number | null;
  /** The playback speed the motion's definition declares, as stored. */
  speed: number | null;
  /** How many bones the motion actually drives, the rest holding their bind pose. */
  animatedBoneCount: number;
  /** Floats one bone's transform occupies in the baked buffer, so a consumer indexes it without agreeing a constant. */
  floatsPerBone: number;
};

/** One motion file set a visual animates from, and what the reference came to. */
export type VisualMotionDependency = {
  reference: string;
  resolution: XrayResolution;
};

/** Byte range of one packed attribute inside a visual's geometry buffer. */
export type VisualSection = {
  byteOffset: number;
  byteLength: number;
};

/** Where one submesh's skinning links sit in the geometry buffer. */
export type VisualSkin = {
  indices: VisualSection;
  weights: VisualSection;
};

/** Why a submesh produced no geometry, graded so a caller does not read the message to find out. */
export enum EVisualSkipCause {
  /**
   * Geometry is stored in a form the packer does not handle, such as a shared vertex or index
   * container living outside the file.
   */
  UNSUPPORTED = "unsupported",
  /** Geometry contradicts itself, such as a detail level reaching past the index buffer it indexes. */
  MALFORMED = "malformed",
}

/** Every `EVisualSkipCause` as the spelling it crosses IPC as, for a value no member has narrowed. */
export type VisualSkipCause = `${EVisualSkipCause}`;

/** Enclosing sphere in three.js space. */
export type VisualSphere = {
  center: Vector3d;
  radius: number | null;
};

/** One drawable piece of a visual: a child of a skeleton, or a whole single level visual. */
export type VisualSubmesh = {
  index: number;
  modelType: number;
  modelTypeLabel: string;
  /**
   * X-Ray logical texture path, without an extension. A skeleton keeps these on its children rather
   * than at the top level, which is why a skeleton's own texture chunk is usually absent.
   */
  textureName: string | null;
  shaderName: string | null;
  content: VisualSubmeshContent;
};

/** Every `kind` the `VisualSubmeshContent` union is told apart by, so a switch or a comparison names one. */
export enum EVisualSubmeshContent {
  PACKED = "packed",
  SKIPPED = "skipped",
}

/** Whether a submesh produced drawable geometry, and why not when it did not. */
export type VisualSubmeshContent =
  { kind: "packed"; geometry: VisualGeometry } | { kind: "skipped"; cause: VisualSkipCause; reason: string };

/** One texture a visual's submesh declares, and what the reference came to. */
export type VisualTextureDependency = {
  submeshIndex: number;
  reference: string;
  resolution: XrayResolution;
};

/** One transform in renderer space: three basis vectors and a translation. */
export type VisualTransform = {
  i: Vector3d;
  j: Vector3d;
  k: Vector3d;
  c: Vector3d;
};
