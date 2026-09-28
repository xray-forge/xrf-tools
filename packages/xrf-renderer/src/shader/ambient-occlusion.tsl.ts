import {
  abs,
  clamp,
  float,
  floor,
  int,
  ivec2,
  max,
  screenCoordinate,
  select,
  texture,
  textureLoad,
  vec2,
} from "three/tsl";
import { Node, Texture } from "three/webgpu";

/**
 * The occlusion searched at half resolution, brought up to the frame's pixel: the four searched pixels around it by
 * how near each lies, and each by how near its distance lies to the pixel's, so an edge never takes the other side's.
 * A searched pixel sits on the frame's even texels, so a pixel on one of them reads exactly what was searched there.
 *
 * @param occlusion - Visibility in red, distance along the view in green, as the denoise wrote it.
 * @param distance - The pixel's distance along the view.
 * @param isDrawn - Whether anything was drawn at the pixel.
 * @returns How much of the hemisphere and ambient light reaches the pixel: all where nothing was drawn.
 */
export function toUpsampledAmbientOcclusion(
  occlusion: Texture,
  distance: Node<"float">,
  isDrawn: Node<"bool">
): Node<"float"> {
  const lastPixel: Node<"vec2"> = vec2(texture(occlusion).size(int(0)) as Node<"uvec2">).sub(1);
  const frameTexel: Node<"vec2"> = floor(screenCoordinate.xy);
  const base: Node<"vec2"> = floor(frameTexel.div(2));
  const fraction: Node<"vec2"> = frameTexel.sub(base.mul(2)).mul(0.5);

  let sum: Node<"float"> = float(0);
  let weights: Node<"float"> = float(0);

  for (const [x, y] of [
    [0, 0],
    [1, 0],
    [0, 1],
    [1, 1],
  ]) {
    const texel: Node<"vec4"> = textureLoad(occlusion, ivec2(clamp(base.add(vec2(x, y)), vec2(0), lastPixel)));
    const bilinear: Node<"float"> = (x ? fraction.x : fraction.x.oneMinus()).mul(
      y ? fraction.y : fraction.y.oneMinus()
    );
    const difference: Node<"float"> = abs(texel.y.sub(distance)).div(max(distance, 1e-3));
    const weight: Node<"float"> = bilinear.div(difference.mul(difference).add(1e-3));

    sum = sum.add(texel.x.mul(weight));
    weights = weights.add(weight);
  }

  return select(isDrawn, sum.div(max(weights, 1e-6)), float(1));
}
