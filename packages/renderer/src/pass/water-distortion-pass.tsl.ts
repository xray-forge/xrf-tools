import { screenUV, texture } from "three/tsl";
import { Node, Texture } from "three/webgpu";

import { WATER_NEUTRAL_DISTORTION } from "#/material/water-surface.tsl";
import { WaterUniforms } from "#/uniforms/water-uniforms";

/**
 * `combine_2`'s `USE_DISTORT`: the frame read where the distortion target moves each pixel, `(distort.xy - .5) *
 * def_distort`. Measured from the cleared value rather than a half, so a pixel nothing distorts is read where it is.
 *
 * @param frame - A copy of the frame as it stands.
 * @param distortion - The distortion target.
 * @param water - What the water's shaders read, its strength among it.
 * @returns The frame, distorted.
 */
export function toWaterDistortionFragment(frame: Texture, distortion: Texture, water: WaterUniforms): Node<"vec4"> {
  const offset: Node<"vec2"> = texture(distortion, screenUV).xy.sub(WATER_NEUTRAL_DISTORTION).mul(water.distortion);

  return texture(frame, screenUV.add(offset));
}
