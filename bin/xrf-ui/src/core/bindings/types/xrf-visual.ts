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
  /** Index of the parent in this same list, or `None` for a root or a parent no bone carries. */
  parentIndex: number | null;
  /**
   * The bone's whole bind transform in model space, or `None` when the file carries no IK chunk.
   *
   * The whole transform rather than only the joint position, because skinning needs its inverse: a vertex is posed as
   * `animated_model * inverse(bind_model)` (`SkeletonCustom.cpp:508`), and the position alone cannot produce that.
   * `c` is the joint, which is what a skeleton overlay draws.
   */
  bindTransform: VisualTransform | null;
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
  /** Skinning links, or `None` for geometry that carries none and is therefore drawn as it is stored. */
  skin: VisualSkin | null;
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
 * What one baked motion is, beside the frames themselves.
 *
 * Baked rather than sampled on demand because playback runs at thirty frames a second and every frame would otherwise
 * be a round trip. A measured motion averages 78 frames, so a 47 bone skeleton bakes to about 44 kilobytes - cheaper
 * to send once than to ask for repeatedly.
 */
export type VisualMotionBake = {
  name: string;
  /**
   * Frames the buffer holds: the longest key stream the payload carries, not the count the motion declares.
   *
   * The two agree whenever any bone is keyed, because a keyed stream stores one key a frame. They part only for a
   * motion of nothing but held bones, which is constant however many frames it declares and so bakes to the one
   * frame that answers all of them. `duration` follows the frames baked rather than the frames declared, as it
   * already does for a motion declaring none.
   */
  frameCount: number;
  boneCount: number;
  /**
   * Seconds playing the motion takes: its frames at the format's sample rate, over its playback speed.
   *
   * The time the engine spends on it rather than the span of its keyframes, so two motions of the same length that
   * play at different speeds report different durations. The raw span is `frame_count` over the sample rate, which a
   * consumer indexing frames already holds, so only the speed it was divided by is reported beside this.
   */
  duration: number | null;
  /**
   * The playback speed the motion's definition declares, as stored.
   *
   * A value that is not positive is not what `duration` was divided by; see
   * [`OgfMotionDefinition::get_playback_speed`].
   */
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

/**
 * Where one submesh's skinning links sit in the geometry buffer.
 *
 * Four per vertex whatever the source layout stores, because that is the width a renderer's skin attributes have:
 * a vertex with fewer links is padded with bone zero at weight zero, which contributes nothing. Indices are `u16`
 * into the visual's own bone list - the engine looks a link up as `LL_GetBoneInstance(v.matrix)`
 * (`xray-16/src/Layers/xrRender/SkeletonX.cpp:359`), so they are global to the model rather than local to the
 * submesh - and each vertex's weights sum to one, the last one having been reconstructed by the reader.
 */
export type VisualSkin = {
  indices: VisualSection;
  weights: VisualSection;
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

/**
 * One transform in renderer space: three basis vectors and a translation.
 *
 * Four vectors rather than sixteen floats because that is what it is - the fourth row of a 4x4 is never anything but
 * `0 0 0 1` here - and because `i`, `j`, `k`, `c` are the names the engine's own `Fmatrix` uses, so a value crossing
 * the wire reads against the source it was composed from. Laid out in this order, the floats are already a
 * column-major 4x4's first three columns and its translation, which is the layout a renderer's matrix expects.
 */
export type VisualTransform = {
  i: Vector3d;
  j: Vector3d;
  k: Vector3d;
  c: Vector3d;
};
