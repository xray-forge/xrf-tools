import { clamp, float, vec4 } from "three/tsl";
import { TextureNode } from "three/webgpu";

import { ISurfaceInputs } from "#/material/surface-inputs";
import { ISurfaceShader } from "#/material/surface-shader";
import { ESurfaceSlot } from "#/material/surface-slot";
import { toSurfaceCoordinates, toTintedColor } from "#/material/surface-texel.tsl";
import { ISurfaceVariant } from "#/material/surface-variant";

/**
 * `wmark` and `simple`: the base alone, sampled through `smp_rtlinear` - its top level, clamped - and composited into
 * the albedo by its draw. Nothing clips it: DX10 routes `aref` to a shader constant, and `simple.ps` reads none.
 *
 * @param variant - The surfaces drawn.
 * @param inputs - What the material drawing carries.
 * @returns Their shader.
 */
export function toWallmarkSurfaceShader(variant: ISurfaceVariant, inputs: ISurfaceInputs): ISurfaceShader {
  const base: TextureNode = inputs.sample(ESurfaceSlot.BASE, clamp(toSurfaceCoordinates(inputs), 0, 1), false);
  const top: TextureNode = base.level(float(0));

  return { colorNode: vec4(toTintedColor(top.xyz, variant, inputs), top.w) };
}
