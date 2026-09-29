import { cos, float, Fn, normalize, screenUV, sin, vec3, vec4 } from "three/tsl";
import { CubeTextureNode, Node } from "three/webgpu";

import { toBoxLookup, toCubeLookup } from "#/shader/sky.tsl";

/** Taps the blur averages: a centre and rings of eight around it. */
const RINGS: ReadonlyArray<number> = [0.33, 0.66, 1];

/** Taps a ring holds. */
const RING_TAPS: number = 8;

/** The cone the blur averages over, half its angle: wide enough that no cloud's shape survives. */
const CONE: number = (15 * Math.PI) / 180;

/**
 * One sky cube blurred into a haze map: each texel's direction in the box's axes, the bearing across and the height up,
 * averaged over a cone about it through the half box, as the sky reads the cube there.
 *
 * @param cube - The cube.
 * @returns The fragment, the blurred colour in rgb.
 */
export function toSkyHazeFragment(cube: CubeTextureNode): Node<"vec4"> {
  return Fn(() => {
    const bearing: Node<"float"> = screenUV.x.sub(0.5).mul(Math.PI * 2);
    // As `toHazeCoordinates` reads it back: a target sampled where it was drawn.
    const height: Node<"float"> = screenUV.y.sub(0.5).mul(Math.PI);
    const direction: Node<"vec3"> = vec3(cos(height).mul(sin(bearing)), sin(height), cos(height).mul(cos(bearing)));
    // A frame about it, whatever its height: the vertical never lies along it but at the poles, where any frame will do.
    const side: Node<"vec3"> = normalize(vec3(direction.z, float(0), direction.x.negate()).add(vec3(1e-4, 0, 0)));
    const up: Node<"vec3"> = normalize(direction.cross(side));

    let sum: Node<"vec3"> = sample(cube, direction);
    let taps: number = 1;

    for (const ring of RINGS) {
      const spread: number = Math.tan(CONE * ring);

      for (let tap: number = 0; tap < RING_TAPS; tap += 1) {
        const angle: number = ((tap + ring) / RING_TAPS) * Math.PI * 2;
        const turned: Node<"vec3"> = normalize(
          direction.add(side.mul(Math.cos(angle) * spread)).add(up.mul(Math.sin(angle) * spread))
        );

        sum = sum.add(sample(cube, turned));
        taps += 1;
      }
    }

    return vec4(sum.div(taps), 1);
  })();
}

/** The cube along a box direction, through the half box as the sky reads it. */
function sample(cube: CubeTextureNode, box: Node<"vec3">): Node<"vec3"> {
  return cube.sample(toCubeLookup(toBoxLookup(box))).level(float(0)).xyz;
}
