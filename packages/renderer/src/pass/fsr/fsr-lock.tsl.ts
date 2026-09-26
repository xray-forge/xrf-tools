import { bool, float, Fn, max, min, screenCoordinate, select, vec2 } from "three/tsl";
import { Node, Texture } from "three/webgpu";

import { IFsrConstants, toScalarTexel } from "#/pass/fsr/fsr-common.tsl";
import { loadClamped, NEIGHBOURHOOD } from "#/shader/texel.tsl";

// `ffx_fsr2_lock.h`: the thin features to lock, found at the render size. FSR writes each to the display pixel its
// sample falls in; the accumulation gathers it from there instead.

/** A neighbour's luma within this share of the centre's is similar to it. */
const SIMILAR_THRESHOLD: number = 1.05;

/** The quadrants of the neighbourhood, row by row, any of which all similar surrounds the centre. */
const QUADRANTS: ReadonlyArray<ReadonlyArray<number>> = [
  [0, 1, 3, 4],
  [1, 2, 4, 5],
  [3, 4, 6, 7],
  [4, 5, 7, 8],
];

/**
 * `ComputeThinFeatureConfidence`: a texel whose luma stands above or below all it differs from around it, and that no
 * quadrant of similar texels surrounds, is a thin feature to lock.
 *
 * @param lockLuma - The luma the locks read.
 * @param constants - The frame's FSR constants.
 * @returns One where a thin feature is to lock, zero elsewhere.
 */
export function toFsrLock(lockLuma: Texture, constants: IFsrConstants): Node<"vec4"> {
  return Fn(() => {
    const position: Node<"vec2"> = screenCoordinate.xy.floor();
    const nucleus: Node<"float"> = loadClamped(lockLuma, position, constants.renderSize).x.toVar();
    const similar: Array<Node<"bool">> = [];
    let dissimilarMin: Node<"float"> = float(3.402823466e38);
    let dissimilarMax: Node<"float"> = float(0);

    for (const [x, y] of NEIGHBOURHOOD) {
      if (x === 0 && y === 0) {
        similar.push(bool(true));
        continue;
      }

      const luma: Node<"float"> = loadClamped(lockLuma, position.add(vec2(x, y)), constants.renderSize).x.toVar();
      // `max / min < 1.05` without the division, which the reference leaves to IEEE's NaN for a black texel.
      const least: Node<"float"> = min(luma, nucleus);
      const isSimilar: Node<"bool"> = least
        .greaterThan(0)
        .and(max(luma, nucleus).lessThan(least.mul(SIMILAR_THRESHOLD)))
        .toVar();

      similar.push(isSimilar);
      dissimilarMin = select(isSimilar, dissimilarMin, min(dissimilarMin, luma));
      dissimilarMax = select(isSimilar, dissimilarMax, max(dissimilarMax, luma));
    }

    const isRidge: Node<"bool"> = nucleus.greaterThan(dissimilarMax).or(nucleus.lessThan(dissimilarMin));
    let isSurrounded: Node<"bool"> = bool(false);

    for (const quadrant of QUADRANTS) {
      isSurrounded = isSurrounded.or(
        similar[quadrant[0]].and(similar[quadrant[1]]).and(similar[quadrant[2]]).and(similar[quadrant[3]])
      );
    }

    return toScalarTexel(float(isRidge.and(isSurrounded.not())));
  })();
}
