/** What the LOD cull decided a clump draws as, bits of its terms' fourth word. */
export enum EStaticLodState {
  /** Its trees, near enough to be drawn in full. */
  TREES = 1,
  /** Its impostor, far enough to be drawn in their place. */
  IMPOSTOR = 2,
}
