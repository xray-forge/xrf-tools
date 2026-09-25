import { TRendererVector } from "#/contract/renderer-lighting";

/** One face of a point light's shadow: where it looks, and which way is up in it. */
export interface ILightShadowFaceBasis {
  direction: TRendererVector;
  up: TRendererVector;
}

/**
 * A point light's six shadow faces, `light::Export`'s omni parts, along the world's axes: `+x`, `-x`, `+y`, `-y`,
 * `+z`, `-z`. Its right is `direction x up`, the camera's `+x`, so a face and its lookup agree.
 */
export const LIGHT_SHADOW_POINT_FACES: ReadonlyArray<ILightShadowFaceBasis> = [
  { direction: [1, 0, 0], up: [0, 1, 0] },
  { direction: [-1, 0, 0], up: [0, 1, 0] },
  { direction: [0, 1, 0], up: [0, 0, -1] },
  { direction: [0, -1, 0], up: [0, 0, 1] },
  { direction: [0, 0, 1], up: [0, 1, 0] },
  { direction: [0, 0, -1], up: [0, 1, 0] },
];

/** How far every face's projection is widened past its cone: `tan_shift` (`Light_Render_Direct_ComputeXFS.cpp`). */
export const LIGHT_SHADOW_WIDENING: number = (3.5 * Math.PI) / 180;

/** An omni part's cone: a quarter turn. */
export const LIGHT_SHADOW_POINT_CONE: number = Math.PI / 2;

/**
 * @param cone - A face's cone, in radians.
 * @returns What a view coordinate over its depth is scaled by to reach the face's clip space: `cot` of half the widened
 *   cone.
 */
export function toLightShadowScale(cone: number): number {
  return 1 / Math.tan((cone + LIGHT_SHADOW_WIDENING) / 2);
}
