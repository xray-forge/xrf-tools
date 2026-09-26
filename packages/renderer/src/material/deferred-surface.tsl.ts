import { vec4 } from "three/tsl";
import { Node } from "three/webgpu";

import { ERendererDraw, IRendererSurface } from "#/contract/scene/renderer-surface";
import { toImpostorSurfaceShader } from "#/material/impostor-surface.tsl";
import { MaterialSamplers } from "#/material/material-samplers";
import { ISurfaceShader } from "#/material/surface-shader";
import { ISurfaceTexel } from "#/material/surface-texel";
import { toSurfaceTexel } from "#/material/surface-texel.tsl";
import { toAlphaCut } from "#/shader/alpha-cut.tsl";
import { toGBufferOutput } from "#/shader/gbuffer.tsl";
import { toSurfaceMotion } from "#/shader/motion.tsl";
import { RendererUniforms } from "#/uniforms/renderer-uniforms";

/**
 * `deffer_base` and `deffer_base_bump`: raw albedo and gloss, the view normal, and the baked occlusions.
 *
 * @param surface - The surface drawn.
 * @param samplers - Where its slots are bound.
 * @param uniforms - What the frame's shaders read.
 * @returns Its shader.
 */
export function toDeferredSurfaceShader(
  surface: IRendererSurface,
  samplers: MaterialSamplers,
  uniforms: RendererUniforms
): ISurfaceShader {
  if (surface.isImpostor) {
    return toImpostorSurfaceShader(surface, samplers, uniforms);
  }

  const texel: ISurfaceTexel = toSurfaceTexel(surface, samplers, uniforms);
  const albedo: Node<"vec4"> = vec4(texel.albedo, texel.gloss);

  return {
    fragmentNode: toGBufferOutput(
      surface.draw === ERendererDraw.CUT_OUT ? toAlphaCut(texel.alpha, albedo, surface.alphaReference) : albedo,
      texel.normal,
      texel.hemi,
      texel.sun,
      texel.slice,
      toSurfaceMotion(uniforms)
    ),
  };
}
