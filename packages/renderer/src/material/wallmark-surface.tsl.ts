import { clamp, float, vec4 } from "three/tsl";
import { TextureNode } from "three/webgpu";

import { IRendererSurface } from "#/contract/scene/renderer-surface";
import { MaterialSamplers } from "#/material/material-samplers";
import { ISurfaceShader } from "#/material/surface-shader";
import { toSurfaceCoordinates, toTintedColor } from "#/material/surface-texel.tsl";
import { getWhiteTexture } from "#/texture/placeholder-textures";

/**
 * `wmark` and `simple`: the base alone, sampled through `smp_rtlinear` - its top level, clamped - and composited into
 * the albedo by its draw. Nothing clips it: DX10 routes `aref` to a shader constant, and `simple.ps` reads none.
 *
 * @param surface - The surface drawn.
 * @param samplers - Where its slots are bound.
 * @returns Its shader.
 */
export function toWallmarkSurfaceShader(surface: IRendererSurface, samplers: MaterialSamplers): ISurfaceShader {
  const base: TextureNode = samplers.bind(
    surface.textures.base,
    getWhiteTexture(),
    clamp(toSurfaceCoordinates(surface), 0, 1)
  );
  const top: TextureNode = base.level(float(0));

  return { colorNode: vec4(toTintedColor(top.xyz, surface), top.w) };
}
