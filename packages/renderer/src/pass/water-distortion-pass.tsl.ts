import {
  clamp,
  ivec2,
  max,
  screenCoordinate,
  screenSize,
  screenUV,
  select,
  texture,
  textureLoad,
  vec2,
} from "three/tsl";
import { Node, Texture } from "three/webgpu";

import { WATER_NEUTRAL_DISTORTION } from "#/material/water-surface.tsl";
import { WaterUniforms } from "#/uniforms/water-uniforms";

/** Metres, and the share of its own depth, what a move reads may stand nearer than what the pixel shows behind it. */
const NEARER_MARGIN: number = 0.25;
const NEARER_SHARE: number = 0.02;

/**
 * `combine_2`'s `USE_DISTORT`: the frame read where the distortion target moves each pixel, `(distort.xy - .5) *
 * def_distort`. Measured from the cleared value rather than a half, so a pixel nothing distorts is read where it is.
 * A move that would read something standing nearer than what the pixel shows behind the water reads the pixel itself:
 * a departure from the engine, whose water copies a railing standing in it into the water beside it.
 *
 * @param frame - A copy of the frame as it stands.
 * @param distortion - The distortion target.
 * @param water - What the water's shaders read, its strength and the depth behind it among it.
 * @returns The frame, distorted.
 */
export function toWaterDistortionFragment(frame: Texture, distortion: Texture, water: WaterUniforms): Node<"vec4"> {
  const offset: Node<"vec2"> = texture(distortion, screenUV).xy.sub(WATER_NEUTRAL_DISTORTION).mul(water.distortion);
  const moved: Node<"vec2"> = screenUV.add(offset);
  const at: Node<"vec2"> = clamp(moved.mul(screenSize), vec2(0), screenSize.sub(1));
  const here: Node<"float"> = textureLoad(water.depth, ivec2(screenCoordinate.xy)).x;
  const there: Node<"float"> = textureLoad(water.depth, ivec2(at)).x;
  const isNearer: Node<"bool"> = there.lessThan(here.sub(max(NEARER_MARGIN, here.mul(NEARER_SHARE))));

  return texture(frame, select(isNearer, screenUV, moved));
}
