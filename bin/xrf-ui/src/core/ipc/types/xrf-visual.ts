// Auto-generated rust bindings. Do not edit it manually.

import { Vector3d } from "@/core/ipc/types/xrf-math";
import { XrayResolution } from "@/core/ipc/types/xrf-vfs";

/** Everything about a packed sector except the bytes themselves. */
export type SectorDescription = {
  /** The sector packed, by its index in the sectors chunk. */
  sector: number;
  /** Everything the level bakes in place, packed onto one vertex array and drawn section by section. */
  geometry: SectorGeometry;
  sections: Array<SectorSection>;
  /** Meshes the sector draws many times over, each packed once with the places it stands. */
  instances: Array<SectorInstanceGroup>;
  /** Drawables that produced no geometry, which is none for every level measured. */
  skipped: Array<SectorSkip>;
  /** Extent the packed vertices span, absent when the sector packed none. */
  bounds: VisualBounds | null;
  bufferLength: number;
};

/** Where one packed mesh's attributes sit inside a sector's buffer, and what to draw from them. */
export type SectorGeometry = {
  vertexCount: number;
  indexCount: number;
  positions: VisualSection;
  normals: VisualSection | null;
  /** The authored tangent of every vertex, mirrored with the normal. */
  tangents: VisualSection | null;
  /** The authored binormal of every vertex, mirrored with the normal. */
  binormals: VisualSection | null;
  uvs: VisualSection | null;
  /** The lightmap coordinate of every vertex, for a surface xrLC lit from lightmaps. */
  lightmapUvs: VisualSection | null;
  /** The baked vertex colour of every vertex, as three floats in zero to one. */
  colors: VisualSection | null;
  /** The hemisphere term of every vertex, which rides in the normal and is present with it. */
  hemi: VisualSection | null;
  /** Every index, as 32-bit elements: a sector reaches past what sixteen bits address. */
  indices: VisualSection;
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

/** One draw of a sector's own geometry: the indices to draw, and the surface they are drawn with. */
export type SectorSection = {
  surface: SectorSurface;
  /** Drawables packed into this section, by their index in the visuals run. */
  drawables: Array<number>;
  draw: VisualDrawRange;
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
  /** The lightmaps the same entry names after the base, sampled with the second uv set. */
  lightmaps: Array<string>;
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

/**
 * Everything a visual needs from outside itself, resolved.
 *
 * The crate that parses a visual is the crate that knows what it references, so extraction lives beside the parser. It
 * resolves through a borrowed probe and never mounts or plans: which sources exist, and in what order, is the calling
 * binary's policy, and a viewer, a sweep and a level editor each answer it differently.
 */
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

/**
 * The slice of an index buffer that draws one detail level.
 *
 * Element offsets into the index buffer, not bytes, because that is what a draw call takes.
 */
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

/**
 * One motion file set a visual animates from, and what the reference came to.
 *
 * A reference may be a mask — `wpn\wpn_ak74_*.omf` names every matching file — so one entry can hold several located
 * assets. Embedded motions are not here: they are inside the visual and there is nothing to resolve.
 */
export type VisualMotionDependency = {
  reference: string;
  resolution: XrayResolution;
};

/**
 * Byte range of one packed attribute inside a visual's geometry buffer.
 *
 * Both values are byte counts rather than element counts, so a consumer builds a typed array view
 * directly from them. The packer aligns every offset to four bytes for `Float32Array` and
 * `Uint16Array` views.
 */
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

/**
 * One texture a visual's submesh declares, and what the reference came to.
 *
 * Paired with the submesh index rather than positioned in a list, so an outcome cannot be joined to the wrong
 * submesh by a caller that reorders or resolves in parallel.
 *
 * A submesh declaring no texture has no entry here at all — that is the normal case for a skeleton's own record, and
 * absence says it more plainly than a variant meaning "nothing was asked".
 */
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
