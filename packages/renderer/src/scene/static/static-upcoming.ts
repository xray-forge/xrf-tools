import { SceneGeometry } from "#/scene/geometry/scene-geometry";

/**
 * What one object still waiting to draw will take of the static draws: its geometry's room in an arena, a slot for
 * each section it draws statically, and, for an instanced one, its places and a row for each place of each section.
 */
export interface IStaticUpcoming {
  geometry: SceneGeometry;
  /** Its sections drawn as static draws. */
  sections: number;
  /** Places it stands in, none for an object that is not instanced. */
  places: number;
}
