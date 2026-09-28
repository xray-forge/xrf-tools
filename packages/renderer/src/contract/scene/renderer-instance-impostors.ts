/**
 * The impostors an instanced object's places belong to.
 */
export interface IRendererInstanceImpostors {
  /** The key the set was put under. */
  key: string;
  /** One integer a place: its impostor's position in the set, or -1 for a place none stands in for. */
  indices: Int32Array;
}
