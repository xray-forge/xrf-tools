import { dFdx, dFdy, dot, float, int, log2, max, vec2 } from "three/tsl";
import { Node, TextureNode } from "three/webgpu";

/** How much alpha each mip level down adds, so a cut-out's coverage survives its mips averaging the alpha away. */
const ALPHA_MIP_SCALE: number = 0.25;

/**
 * A cut-out's alpha scaled up by the mip level it is sampled at. A leaf card's mips average its leaves with the gaps
 * between them, so far away the alpha falls under the reference and the canopy thins to specks; scaled by the level,
 * the coverage a cut keeps stays near what the top level has.
 *
 * @param alpha - The alpha the base texture gives.
 * @param coordinates - Where the base is sampled.
 * @param base - The base texture's sampler, whose size says the level.
 * @returns The alpha to cut by.
 */
export function toCoverageAlpha(alpha: Node<"float">, coordinates: Node<"vec2">, base: TextureNode): Node<"float"> {
  // The size three types loosely: the top level's texels across and down.
  const size: Node<"vec2"> = vec2(base.size(int(0)) as unknown as Node<"ivec2">);
  const texels: Node<"vec2"> = coordinates.mul(size);
  const x: Node<"vec2"> = dFdx(texels);
  const y: Node<"vec2"> = dFdy(texels);
  const level: Node<"float"> = max(float(0), log2(max(dot(x, x), dot(y, y))).mul(0.5));

  return alpha.mul(level.mul(ALPHA_MIP_SCALE).add(1));
}
