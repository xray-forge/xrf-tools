import { Sphere, Vector3 } from "three/webgpu";

import { TRendererColor, TRendererVector } from "#/contract/renderer-lighting";
import { ERendererLightKind, IRendererSpotLight, TRendererLight } from "#/contract/scene/renderer-lights";

/** `ps_r2_slight_fade`: what a shadowed light's screen area is scaled by before it fades (`xrRender_console.cpp`). */
const SHADOWED_FADE: number = 0.5;

/** `EPS`: what `light::get_LOD` adds to the squared distance, so a camera inside the light divides by no zero. */
const EPS: number = 0.00001;

/** A spot `compute_xf_spot` gives up the given right of, and takes the world's up in its place. */
const RIGHT_EPSILON: number = EPS;

/** How near parallel to the world's up a spot may point before `compute_xf_spot` takes the world's forward instead. */
const PARALLEL: number = 0.99;

/** A light's frame in world space: where it stands, and a spot's direction with its right and up square to it. */
export interface ILightBasis {
  readonly position: Vector3;
  readonly direction: Vector3;
  readonly right: Vector3;
  readonly up: Vector3;
}

/** @returns A basis to write a light's into. */
export function createLightBasis(): ILightBasis {
  return { direction: new Vector3(), position: new Vector3(), right: new Vector3(), up: new Vector3() };
}

/**
 * `compute_xf_spot`'s basis: the direction, and the right the lamp gives made square to it through the up they make,
 * or the world's up where it gives none. Crossed in the renderer's mirrored space, the engine's cross products turn
 * their sign.
 *
 * @param light - The light.
 * @param out - Where its basis is written.
 * @returns The basis written; a point's is its position alone, facing down `-z`.
 */
export function toLightBasis(light: TRendererLight, out: ILightBasis): ILightBasis {
  const { position, direction, right, up } = out;

  toVector(position, light.position);

  if (light.kind !== ERendererLightKind.SPOT) {
    direction.set(0, 0, -1);
    right.set(1, 0, 0);
    up.set(0, 1, 0);

    return out;
  }

  toVector(direction, light.direction).normalize();
  toVector(right, light.right);

  if (right.lengthSq() > RIGHT_EPSILON) {
    up.crossVectors(direction, right.normalize()).negate().normalize();
    right.crossVectors(up, direction).negate().normalize();
  } else {
    // The engine's world forward, `+z`, is the renderer's `-z`.
    up.set(0, 1, 0);

    if (Math.abs(up.dot(direction)) > PARALLEL) {
      up.set(0, 0, -1);
    }

    right.crossVectors(up, direction).negate().normalize();
    up.crossVectors(direction, right).negate().normalize();
  }

  return out;
}

/**
 * The least sphere around all a light reaches, as far as its range strays, which it is culled and binned by: a
 * point's range; a narrow spot's the one through its apex and its rim, a wide one's its rim's.
 *
 * @param light - The light.
 * @param out - Where the sphere is written.
 * @returns The sphere written.
 */
export function toLightBound(light: TRendererLight, out: Sphere): Sphere {
  const reach: number = light.range + (light.rangeJitter ?? 0);

  toVector(out.center, light.position);
  out.radius = reach;

  if (light.kind !== ERendererLightKind.SPOT || light.cone >= Math.PI) {
    return out;
  }

  const half: number = light.cone / 2;
  const isNarrow: boolean = half <= Math.PI / 4;
  // The two meet at a quarter turn's cone: centre and radius both `reach / sqrt(2)`.
  const along: number = isNarrow ? reach / (2 * Math.cos(half)) : reach * Math.cos(half);

  out.radius = isNarrow ? along : reach * Math.sin(half);
  moveAlong(out.center, light, along);

  return out;
}

/**
 * `light::spatial_move`'s sphere, which the engine fades a shadowed light and sizes its maps by: a point's range; a
 * spot's past its cone, wider than it reaches.
 *
 * @param light - The light.
 * @param out - Where the sphere is written.
 * @returns The sphere written.
 */
export function toLightSpatialSphere(light: TRendererLight, out: Sphere): Sphere {
  toVector(out.center, light.position);
  out.radius = light.range;

  if (light.kind !== ERendererLightKind.SPOT) {
    return out;
  }

  const half: number = light.cone / 2;
  const isWide: boolean = light.cone >= Math.PI / 2;

  out.radius = isWide ? light.range * Math.tan(half) : light.range / (2 * Math.cos(half) ** 2);
  moveAlong(out.center, light, isWide ? light.range : out.radius);

  return out;
}

/**
 * `light::get_LOD`: how far a shadowed light has faded, by its sphere's share of the screen, as the engine's does.
 *
 * @param spatial - Its spatial sphere.
 * @param eye - The camera's position.
 * @param start - `r_ssaGLOD_start`, the share it starts fading below.
 * @param end - `r_ssaGLOD_end`, the share it is gone at.
 * @returns Its level of detail, one whole, zero gone.
 */
export function toLightLod(spatial: Sphere, eye: Vector3, start: number, end: number): number {
  const area: number = (SHADOWED_FADE * spatial.radius) / (eye.distanceToSquared(spatial.center) + EPS);

  return start > end ? Math.sqrt(Math.min(Math.max((area - end) / (start - end), 0), 1)) : 1;
}

/**
 * @param color - A light's colour.
 * @returns `(mean + luminance) / 2`, which `compute_xf_spot` sizes a brighter light's maps larger by.
 */
export function toLightIntensity(color: TRendererColor): number {
  const [red, green, blue] = color;

  return ((red + green + blue) / 3 + (red * 0.2125 + green * 0.7154 + blue * 0.0721)) / 2;
}

/** Moves a point along a spot's direction, as far as given. */
function moveAlong(point: Vector3, light: IRendererSpotLight, distance: number): void {
  const [x, y, z] = light.direction;
  const scale: number = distance / (Math.hypot(x, y, z) || 1);

  point.set(point.x + x * scale, point.y + y * scale, point.z + z * scale);
}

function toVector(out: Vector3, vector: TRendererVector): Vector3 {
  return out.set(vector[0], vector[1], vector[2]);
}
