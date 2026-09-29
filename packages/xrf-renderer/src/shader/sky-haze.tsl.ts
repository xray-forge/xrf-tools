import { cos, float, Fn, length, max, normalize, screenUV, sin, vec3, vec4 } from "three/tsl";
import { Node } from "three/webgpu";

import { toSkyWithClouds } from "#/shader/clouds.tsl";
import { ISkyWithCloudsUniforms } from "#/shader/sky-with-clouds-uniforms";
import { HAZE_TOP, toSkyColor } from "#/shader/sky.tsl";

/** Taps the blur averages: a centre and rings of eight around it. */
const RINGS: ReadonlyArray<number> = [0.33, 0.66, 1];

/** Taps a ring holds. */
const RING_TAPS: number = 8;

/**
 * How far the blur reaches either way, half its angle: across, wide enough that no cloud's shape survives; up and down,
 * little, so the sky's own gradient stays and the horizon never reads the darker rim under it.
 */
const ACROSS: number = (15 * Math.PI) / 180;
const UP: number = (3 * Math.PI) / 180;

/**
 * The sky as the frame draws it, both skies and the clouds over them, blurred into a haze map: each texel's direction
 * in renderer space, the bearing across and the height up, averaged over a flat ellipse about it.
 *
 * @param uniforms - What the sky and its clouds are drawn with.
 * @returns The fragment, the blurred colour in rgb, as the frame shows it.
 */
export function toSkyHazeFragment(uniforms: ISkyWithCloudsUniforms): Node<"vec4"> {
  return Fn(() => {
    const bearing: Node<"float"> = screenUV.x.sub(0.5).mul(Math.PI * 2);
    // As `toHazeCoordinates` reads it back: a target sampled where it was drawn.
    const height: Node<"float"> = screenUV.y.sub(0.5).mul(Math.PI);
    const direction: Node<"vec3"> = vec3(cos(height).mul(sin(bearing)), sin(height), cos(height).mul(cos(bearing)));
    // A frame about it, whatever its height: the vertical never lies along it but at the poles, where any frame will do.
    const side: Node<"vec3"> = normalize(vec3(direction.z, float(0), direction.x.negate()).add(vec3(1e-4, 0, 0)));
    const up: Node<"vec3"> = normalize(direction.cross(side));

    let sum: Node<"vec3"> = sample(direction, uniforms);
    let taps: number = 1;

    for (const ring of RINGS) {
      const across: number = Math.tan(ACROSS * ring);
      const upward: number = Math.tan(UP * ring);

      for (let tap: number = 0; tap < RING_TAPS; tap += 1) {
        const angle: number = ((tap + ring) / RING_TAPS) * Math.PI * 2;
        const turned: Node<"vec3"> = normalize(
          direction.add(side.mul(Math.cos(angle) * across)).add(up.mul(Math.sin(angle) * upward))
        );

        sum = sum.add(sample(turned, uniforms));
        taps += 1;
      }
    }

    return vec4(sum.div(taps), 1);
  })();
}

/**
 * The sky along a direction as the frame draws it, the clouds laid over it, lifted above the fold: the box's rim under
 * it is what the sky draws below the horizon, and no haze of it.
 */
function sample(direction: Node<"vec3">, uniforms: ISkyWithCloudsUniforms): Node<"vec3"> {
  const lifted: Node<"vec3"> = normalize(
    vec3(direction.x, max(direction.y, length(direction.xz).mul(HAZE_TOP)), direction.z)
  );

  return toSkyWithClouds(lifted, toSkyColor(lifted, uniforms.sky, uniforms.scale), uniforms);
}
