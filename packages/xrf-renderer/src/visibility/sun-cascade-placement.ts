import { clamp, EPS_L, EPS_S } from "@xrf/math";
import { Vector3 } from "three/webgpu";

import { SunCascadeBasis } from "#/visibility/sun-cascade-basis";
import { ISunViewRay } from "#/visibility/sun-view-ray";
import { SunViewRays } from "#/visibility/sun-view-rays";

/** The engine's first guess at the nearest point behind a side, which any real one is nearer than. */
const FAR_BEHIND: number = 10000;

/** How deep from the near plane, in widths, a cascade holds the whole view whatever the engine's placement says. */
const HELD: number = 0.25;

/** The share of the width a held point is kept inside the square by: the sampling's own edge, and a little over. */
const HELD_MARGIN: number = 0.03;

/** How far the sides are brought up, reused. */
const TRANSLATION: Vector3 = new Vector3();

/** How far an edge running back out pulls them back, reused. */
const PUSH: Vector3 = new Vector3();

/** Across a side, in the plane of the view, reused. */
const ACROSS: Vector3 = new Vector3();

/** The sides the view looks away from, at most two, reused. */
const BEHIND: Array<number> = [];

/**
 * Places a cascade's square across the light as the engine does (`compute_caster_model_fixed`): the sides the view
 * looks away from brought up to where the view's edges start, the first stretch of the view held in it, and the edges
 * carried on to where they leave it, for the next cascade to start there.
 *
 * @param center - The square's centre, starting over the camera, moved in place.
 * @param look - Where the camera looks.
 * @param rays - The view's edges where this cascade starts, carried on in place.
 * @param basis - The square's axes.
 * @param width - Metres the square is across.
 */
export function placeSunCascade(
  center: Vector3,
  look: Vector3,
  rays: SunViewRays,
  basis: SunCascadeBasis,
  width: number
): void {
  // Looking along the light, no side faces away from the view and the edges stay where they are.
  if (Math.abs(1 - Math.abs(look.dot(basis.light))) < EPS_S) {
    hold(center, look, rays.near, basis, width);

    return;
  }

  align(center, look, rays.rays, basis, width);
  hold(center, look, rays.near, basis, width);
  advance(center, rays.rays, basis, width);
}

/**
 * Brings the sides the view looks away from up to the nearest point an edge starts from, then back by the share an
 * edge running out through them leaves at.
 */
function align(
  center: Vector3,
  look: Vector3,
  rays: ReadonlyArray<ISunViewRay>,
  { sides }: SunCascadeBasis,
  width: number
): void {
  const half: number = width / 2;

  // The one or two sides the view looks away from, behind the camera.
  BEHIND.length = 0;

  for (let side: number = 0; side < sides.length && BEHIND.length < 2; side += 1) {
    // A plane faces the view only past `EPS_L`.
    if (look.dot(sides[side]) > EPS_L) {
      BEHIND.push(side);
    }
  }

  // Each brought up to the nearest point an edge starts from.
  TRANSLATION.set(0, 0, 0);

  for (const side of BEHIND) {
    let nearest: number = FAR_BEHIND;

    for (const ray of rays) {
      nearest = Math.min(nearest, toInside(sides[side], ray.origin, center, half));
    }

    TRANSLATION.addScaledVector(sides[side], nearest);
  }

  // An edge running back out through a side it was brought up to pulls that side back by the share it leaves at.
  PUSH.set(0, 0, 0);

  for (const side of BEHIND) {
    const normal: Vector3 = sides[side];
    let share: number = 0;

    ACROSS.crossVectors(normal, look).cross(look);

    for (const ray of rays) {
      const along: number = ray.direction.dot(normal);

      if (along < 0) {
        share = Math.max(share, -along / ray.direction.dot(ACROSS));
      }
    }

    if (Math.abs(share) >= EPS_S) {
      PUSH.addScaledVector(normal, -normal.dot(TRANSLATION) * share);
    }
  }

  center.add(TRANSLATION).add(PUSH);
}

/**
 * Moves the square across the light no further than it must to hold the view from the near plane to `HELD` of its
 * width deep; a slice wider than the square, under a wide lens or a wide view, is centred in it instead.
 */
function hold(
  center: Vector3,
  look: Vector3,
  near: ReadonlyArray<ISunViewRay>,
  { right, up }: SunCascadeBasis,
  width: number
): void {
  holdAlong(center, look, near, right, width);
  holdAlong(center, look, near, up, width);
}

/** Holds the slice along one of the square's axes. */
function holdAlong(
  center: Vector3,
  look: Vector3,
  near: ReadonlyArray<ISunViewRay>,
  axis: Vector3,
  width: number
): void {
  const reach: number = width * HELD;
  const half: number = width * (0.5 - HELD_MARGIN);
  let least: number = Infinity;
  let most: number = -Infinity;

  for (const ray of near) {
    const start: number = axis.dot(ray.origin);
    const end: number = start + (axis.dot(ray.direction) * reach) / ray.direction.dot(look);

    least = Math.min(least, start, end);
    most = Math.max(most, start, end);
  }

  const at: number = axis.dot(center);
  const lowest: number = most - half;
  const highest: number = least + half;
  const held: number = lowest > highest ? (least + most) / 2 : clamp(at, lowest, highest);

  center.addScaledVector(axis, held - at);
}

/** Carries each edge to where it leaves the square, which is where the next cascade starts it. */
function advance(center: Vector3, rays: ReadonlyArray<ISunViewRay>, { sides }: SunCascadeBasis, width: number): void {
  const half: number = width / 2;

  for (const ray of rays) {
    let nearest: number = 2 * width;

    for (const normal of sides) {
      const along: number = normal.dot(ray.direction);
      let distance: number = width;

      if (along <= -0.1) {
        const leave: number = -toInside(normal, ray.origin, center, half) / along;

        distance = leave > 0 || Math.abs(leave) < EPS_S ? leave : 0;
      }

      // A ray leaves a plane only past `EPS_L`.
      if (distance > EPS_L && distance < nearest) {
        nearest = distance;
      }
    }

    ray.origin.addScaledVector(ray.direction, nearest);
  }
}

/** How far inside a side of a square the point stands: the engine's `classify` against it. */
function toInside(normal: Vector3, point: Vector3, center: Vector3, half: number): number {
  return normal.dot(point) - normal.dot(center) + half;
}
