import { abs, float, Fn, max, screenCoordinate, select } from "three/tsl";
import { Node, Texture } from "three/webgpu";

import { toScalarTexel } from "#/pass/fsr/fsr-common.tsl";
import { IFsrConstants } from "#/pass/fsr/fsr-constants";
import { IFsrInputs } from "#/pass/fsr/fsr-inputs";
import { loadClamped } from "#/shader/texel.tsl";

// `ffx_fsr2_autogen_reactive_pass.hlsl`: the reactive mask from what the blended surfaces changed, with its tonemap,
// component-maximum and threshold flags on.

/** A change of a channel under which a texel is not reactive. */
const REACTIVE_THRESHOLD: number = 0.2;
/** How reactive a texel past the threshold counts. */
const REACTIVE_VALUE: number = 0.9;

/**
 * @param opaque - The frame before the blended surfaces drew.
 * @param inputs - What FSR 2 reads of the frame, which has them.
 * @param constants - The frame's FSR constants.
 * @returns The reactive mask.
 */
export function toFsrReactive(opaque: Texture, inputs: IFsrInputs, constants: IFsrConstants): Node<"vec4"> {
  return Fn(() => {
    const position: Node<"vec2"> = screenCoordinate.xy.floor();

    // `Tonemap`.
    function tonemap(rgb: Node<"vec3">): Node<"vec3"> {
      return rgb.div(max(max(rgb.x, max(rgb.y, rgb.z)), float(0)).add(1));
    }

    const before: Node<"vec3"> = tonemap(loadClamped(opaque, position, constants.renderSize).xyz);
    const after: Node<"vec3"> = tonemap(loadClamped(inputs.color, position, constants.renderSize).xyz);
    const delta: Node<"vec3"> = abs(after.sub(before));
    const reactive: Node<"float"> = max(delta.x, max(delta.y, delta.z));

    return toScalarTexel(select(reactive.lessThan(REACTIVE_THRESHOLD), float(0), float(REACTIVE_VALUE)));
  })();
}
