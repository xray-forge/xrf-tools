import { clamp, floor, int, ivec2, texture, textureLoad, vec2 } from "three/tsl";
import { DepthTexture, Node, Texture } from "three/webgpu";

import { IBilinearFootprint } from "#/shader/bilinear-footprint";

/** The 3x3 neighbourhood about a texel, row by row, the centre fifth. */
export const NEIGHBOURHOOD: ReadonlyArray<readonly [number, number]> = [-1, 0, 1].flatMap((y: number) =>
  [-1, 0, 1].map((x: number) => [x, y] as const)
);

/**
 * @param source - A texture.
 * @returns Its size at its top level, in texels.
 */
export function toTextureSize(source: Texture): Node<"vec2"> {
  return vec2(texture(source).size(int(0)) as Node<"uvec2">);
}

/**
 * @param position - A texel, held as floats.
 * @param size - The texture's size.
 * @returns The texel, kept on the texture.
 */
export function toClampedTexel(position: Node<"vec2">, size: Node<"vec2">): Node<"ivec2"> {
  return ivec2(clamp(position, vec2(0), size.sub(1)));
}

/**
 * @param source - A texture.
 * @param position - A texel, held as floats.
 * @param size - The texture's size.
 * @returns The texel, the nearest on the texture where it lies off it.
 */
export function loadClamped(source: Texture, position: Node<"vec2">, size: Node<"vec2">): Node<"vec4"> {
  return textureLoad(source, toClampedTexel(position, size));
}

/**
 * @param depth - A depth texture.
 * @param position - A texel, held as floats.
 * @param size - The texture's size.
 * @returns Its depth, reversed as every depth is.
 */
export function loadDepth(depth: DepthTexture, position: Node<"vec2">, size: Node<"vec2">): Node<"float"> {
  return textureLoad(depth, toClampedTexel(position, size)) as unknown as Node<"float">;
}

/**
 * `GetBilinearSamplingData`.
 *
 * @param uv - The point, in texture coordinates.
 * @param size - The texture's size.
 * @returns The four texels about it and their bilinear weights.
 */
export function toBilinearFootprint(uv: Node<"vec2">, size: Node<"vec2">): IBilinearFootprint {
  const position: Node<"vec2"> = uv.mul(size).sub(0.5);
  const base: Node<"vec2"> = floor(position).toVar();
  const fraction: Node<"vec2"> = position.sub(base).toVar();

  return {
    base,
    corners: [
      { offset: [0, 0], weight: fraction.x.oneMinus().mul(fraction.y.oneMinus()) },
      { offset: [1, 0], weight: fraction.x.mul(fraction.y.oneMinus()) },
      { offset: [0, 1], weight: fraction.x.oneMinus().mul(fraction.y) },
      { offset: [1, 1], weight: fraction.x.mul(fraction.y) },
    ],
  };
}
