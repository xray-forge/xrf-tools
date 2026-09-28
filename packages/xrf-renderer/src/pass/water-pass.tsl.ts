import { float, Fn, getViewPosition, screenUV, select, texture, vec4 } from "three/tsl";
import { Node, Texture } from "three/webgpu";

import { WATER_NEUTRAL_DISTORTION } from "#/material/water-surface.tsl";
import { packOutputs, unpackOutputs } from "#/shader/packed-outputs.tsl";
import { CameraUniforms } from "#/uniforms/camera-uniforms";

/** What the depth behind the water reads where nothing was drawn: farther than any water is deep. */
const FAR_BEHIND: number = 1e6;

/**
 * @param depth - The G-buffer's depth.
 * @param camera - The drawing camera's uniforms, which the view position is rebuilt with.
 * @returns The depth behind the water at every pixel, in metres along the view, and the distortion cleared to nothing
 *   as the engine clears `rt_Generic_1`, in the order `RendererTargets` attaches them.
 */
export function toWaterPrepareFragment(depth: Texture, camera: CameraUniforms): Node {
  const packed: Node<"mat4"> = Fn(() => {
    const stored: Node<"float"> = texture(depth, screenUV).x;
    const distance: Node<"float"> = getViewPosition(screenUV, stored, camera.projectionInverse).z.negate();

    return packOutputs(
      vec4(select(stored.lessThanEqual(0), float(FAR_BEHIND), distance), 0, 0, 1),
      vec4(WATER_NEUTRAL_DISTORTION, WATER_NEUTRAL_DISTORTION, 0, WATER_NEUTRAL_DISTORTION)
    );
  })();

  return unpackOutputs(packed, 2);
}
