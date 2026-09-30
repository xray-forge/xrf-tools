import { toDirection, toRadians } from "@xrf/math";

import { TRendererVector } from "#/contract/renderer-vector";

/**
 * The direction sunlight travels, in renderer space, from a weather keyframe's two angles.
 *
 * The engine calls `sun_dir.setHP(altitude, longitude)` (`xrEngine/Environment_misc.cpp`), and `setHP` makes its second
 * argument the pitch (`utils/xrMiscMath/vector.cpp`). So `sun_longitude` is how high the sun stands and
 * `sun_altitude` is where around the horizon it stands - the names are the other way round from what they say.
 *
 * @param altitude - `sun_altitude`, in degrees.
 * @param longitude - `sun_longitude`, in degrees.
 * @returns The direction, normalised, with engine `z` negated into renderer space.
 */
export function toRendererSunDirection(altitude: number, longitude: number): TRendererVector {
  const [x, y, z] = toDirection({ heading: toRadians(altitude), pitch: toRadians(longitude) });

  return [x, y, -z];
}
