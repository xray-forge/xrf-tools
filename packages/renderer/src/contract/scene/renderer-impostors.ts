/** Corners one impostor has: eight facets of four. */
export const RENDERER_IMPOSTOR_CORNERS: number = 32;

/** Floats one corner takes: position, the atlas `u` and `v`, the hemisphere and sun terms, then nothing. */
export const RENDERER_IMPOSTOR_CORNER_FLOATS: number = 8;

/** Facets one impostor has, looking at its trees from eight sides. */
export const RENDERER_IMPOSTOR_FACETS: number = 8;

/**
 * The impostors of clumps of trees, `MT_LOD` visuals: what the engine draws in place of a clump seen from far enough
 * away, a quad blended out of the two facets that face the camera best, in renderer space. Objects name a set by its
 * key, and an impostor by its position in it: the trees of a clump, to be drawn only while it is near enough, and the
 * places of the impostor's own draw.
 */
export interface IRendererImpostors {
  /** Four floats an impostor: its sphere's centre, then radius. */
  spheres: Float32Array;
  /** One float an impostor: what its sphere's screen area is scaled by (`FLOD::lod_factor`). */
  factors: Float32Array;
  /** `RENDERER_IMPOSTOR_CORNER_FLOATS` for each of an impostor's 32 corners, facet by facet. */
  corners: Float32Array;
  /** Four floats for each of an impostor's facets: its normal, facing back along the direction it is seen from. */
  normals: Float32Array;
}

/**
 * The impostors an instanced object's places belong to.
 */
export interface IRendererInstanceImpostors {
  /** The key the set was put under. */
  key: string;
  /** One integer a place: its impostor's position in the set, or -1 for a place none stands in for. */
  indices: Int32Array;
}

/**
 * What of a set moves between threads: its arrays, never copied.
 *
 * @param impostors - The set about to be posted.
 * @returns Its buffers.
 */
export function listRendererImpostorsTransfers(impostors: IRendererImpostors): Array<Transferable> {
  return [impostors.spheres, impostors.factors, impostors.corners, impostors.normals].map((array) => array.buffer);
}
