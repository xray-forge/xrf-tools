import { float, Fn, int, screenCoordinate, select, texture, vec2 } from "three/tsl";
import { Node, Texture } from "three/webgpu";

import { IFsrConstants, IFsrInputs, toClampedUv, toLogLuma, toScalarTexel } from "#/pass/fsr/fsr-common.tsl";
import { loadClamped, toTextureSize } from "#/shader/texel.tsl";

// `ffx_fsr2_compute_luminance_pyramid.h`: SPD's box reduction of the frame's log luma, as far as the mip the locks watch
// for a change of shading, in two steps. No exposure is taken from it: the frame is in the display's range.

/** Drawn texels a side the first step reduces. */
export const LUMA_FIRST_STEP: number = 8;
/** First-step texels a side the second step reduces. */
export const LUMA_SECOND_STEP: number = 4;

/**
 * `SpdLoadSourceImage` and its reductions to the first step: each texel the mean log luma of the drawn texels under it,
 * read where they stand unjittered; off the screen they count as nothing, as SPD counts them.
 */
export function toLumaFirstStep(inputs: IFsrInputs, constants: IFsrConstants): Node<"vec4"> {
  return Fn(() => {
    const base: Node<"vec2"> = screenCoordinate.xy.floor().mul(LUMA_FIRST_STEP);
    let sum: Node<"float"> = float(0);

    for (let y: number = 0; y < LUMA_FIRST_STEP; y += 1) {
      for (let x: number = 0; x < LUMA_FIRST_STEP; x += 1) {
        const at: Node<"vec2"> = base.add(vec2(x, y));
        const uv: Node<"vec2"> = toClampedUv(
          at.add(0.5).add(constants.jitter).div(constants.renderSize),
          constants.renderSize
        );
        const luma: Node<"float"> = toLogLuma(texture(inputs.color, uv).level(int(0)).xyz);

        sum = sum.add(select(at.lessThan(constants.renderSize).all(), luma, float(0)));
      }
    }

    return toScalarTexel(sum.div(LUMA_FIRST_STEP * LUMA_FIRST_STEP));
  })();
}

/** The reduction on to the shading change mip: each texel the mean of the first-step texels under it. */
export function toLumaShadingChange(firstStep: Texture): Node<"vec4"> {
  return Fn(() => {
    const size: Node<"vec2"> = toTextureSize(firstStep);
    const base: Node<"vec2"> = screenCoordinate.xy.floor().mul(LUMA_SECOND_STEP);
    let sum: Node<"float"> = float(0);

    for (let y: number = 0; y < LUMA_SECOND_STEP; y += 1) {
      for (let x: number = 0; x < LUMA_SECOND_STEP; x += 1) {
        const at: Node<"vec2"> = base.add(vec2(x, y));

        sum = sum.add(select(at.lessThan(size).all(), loadClamped(firstStep, at, size).x, float(0)));
      }
    }

    return toScalarTexel(sum.div(LUMA_SECOND_STEP * LUMA_SECOND_STEP));
  })();
}
