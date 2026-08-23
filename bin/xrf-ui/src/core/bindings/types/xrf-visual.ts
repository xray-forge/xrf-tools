// Auto-generated rust bindings. Do not edit it manually.

import { Vector3d } from "@/core/bindings/types/xrf-db";
import { XrayResolution } from "@/core/bindings/types/xrf-vfs";

/**
 * One bone of a visual's skeleton, as a name and the name of its parent.
 *
 * A root bone carries an empty parent. Names rather than indices, because that is how OGF stores the
 * hierarchy and a tree can be rebuilt from them without further work.
 */
export type VisualBone = {
  name: string;
  parent: string;
};

/**
 * A visual's extent, as a box and a sphere.
 *
 * A description carries this twice, unreconciled: once as the values the OGF header declares and
 * once as the values its geometry actually spans. A file whose declared extent disagrees with its
 * vertices then shows the disagreement instead of silently mis-framing a camera.
 *
 * A computed sphere is centred on the computed box and reaches the furthest vertex from that
 * centre. That is an enclosing sphere rather than the minimal one, so a small disagreement with a
 * declared sphere is expected and only a large one is interesting.
 */
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

/**
 * Everything about a packed visual except the bytes themselves.
 *
 * The counterpart of the geometry buffer: a consumer reads this first, then asks for the buffer and
 * builds views from the byte ranges each submesh carries. The reported total buffer length makes a
 * mismatched description and buffer detectable.
 */
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

/**
 * Where one submesh's attributes sit inside the geometry buffer, and what to draw from them.
 *
 * Every section is a byte range into the one buffer the model ships as, so a consumer builds views
 * over it without copying. `indices` covers the whole index buffer, including the coarser detail
 * levels a progressive submesh carries; [`Self::detail_levels`] names which slices of it are
 * drawable, and a consumer that does not want to choose draws [`Self::get_default_level`].
 */
export type VisualGeometry = {
  vertexCount: number;
  indexCount: number;
  positions: VisualSection;
  normals: VisualSection;
  uvs: VisualSection;
  indices: VisualSection;
  /**
   * Every range a consumer may draw, finest first, and never empty.
   *
   * A static submesh has exactly one: its whole index buffer. A progressive one has a range per detail level of
   * its slide-window table, coarsening as the index rises. Each is validated here — inside the index buffer, and
   * reaching no vertex the submesh lacks — so choosing a level is a choice between drawable ranges rather than a
   * range check the consumer has to remember. A coarse level that fails validation is left out rather than
   * failing the submesh, so a model with one bad level still renders at the levels that are sound.
   */
  detailLevels: Array<VisualDrawRange>;
  bounds: VisualBounds;
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

/**
 * Why a submesh produced no geometry, graded so a caller does not read the message to find out.
 *
 * The distinction is what separates a gap in this crate's coverage from a file that contradicts
 * itself, which is the difference between a sweep noting something and a sweep failing.
 */
export type VisualSkipCause =
  /**
   * Geometry is stored in a form the packer does not handle, such as a shared vertex or index
   * container living outside the file.
   */
  | "unsupported"
  /** Geometry contradicts itself, such as a detail level reaching past the index buffer it indexes. */
  | "malformed";

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

/**
 * Whether a submesh produced drawable geometry, and why not when it did not.
 *
 * A child that cannot be packed is a value rather than an error so the rest of a model still
 * renders, and so the reason reaches the panel that lists it.
 */
export type VisualSubmeshContent =
  | { kind: "packed"; geometry: VisualGeometry }
  | { kind: "skipped"; cause: VisualSkipCause; reason: string };

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
