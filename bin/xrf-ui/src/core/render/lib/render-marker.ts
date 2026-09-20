import { Material, Object3D } from "three";

/** After everything solid, since a marker drawn through geometry has to be drawn after it too. */
const MARKER_RENDER_ORDER: number = 1;

/**
 * Draws a marker over the scene rather than in it, so geometry cannot hide the thing it is there to point at.
 *
 * @param marker - The helper to draw through, which carries its material either way three types one.
 */
export function markThrough(marker: Object3D & { material: Material | Array<Material> }): void {
  for (const material of Array.isArray(marker.material) ? marker.material : [marker.material]) {
    material.depthTest = false;
  }

  marker.renderOrder = MARKER_RENDER_ORDER;
}
