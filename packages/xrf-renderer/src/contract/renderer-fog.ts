import { TRendererColor } from "#/contract/renderer-color";

/**
 * Distance fog, as a weather keyframe states it.
 */
export interface IRendererFog {
  /** `fog_color`. */
  color: TRendererColor;
  /** `fog_distance`, in metres: where fog is total. */
  distance: number;
  /** `fog_density`, from zero (fog starts at 85% of the distance) to one (it starts at the eye). */
  density: number;
  /** `far_plane`, in metres: where the view ends. */
  farPlane: number;
}
