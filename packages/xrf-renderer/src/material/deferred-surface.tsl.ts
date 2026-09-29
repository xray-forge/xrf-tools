import { vec4 } from "three/tsl";
import { Node } from "three/webgpu";

import { ERendererDraw } from "#/contract/scene/renderer-draw";
import { toImpostorSurfaceShader } from "#/material/impostor-surface.tsl";
import { ISurfaceInputs } from "#/material/surface-inputs";
import { ISurfaceShader } from "#/material/surface-shader";
import { ISurfaceTexel } from "#/material/surface-texel";
import { toSurfaceTexel } from "#/material/surface-texel.tsl";
import { ISurfaceVariant } from "#/material/surface-variant";
import { toHashedAlphaCut } from "#/shader/alpha-cut.tsl";
import { toGBufferOutput } from "#/shader/gbuffer.tsl";
import { toSurfaceMotion } from "#/shader/motion.tsl";
import { RendererUniforms } from "#/uniforms/renderer-uniforms";

/**
 * `deffer_base` and `deffer_base_bump`: raw albedo and gloss, the view normal, and the baked occlusions.
 *
 * @param variant - The surfaces drawn.
 * @param inputs - What the material drawing carries.
 * @param uniforms - What the frame's shaders read.
 * @returns Their shader.
 */
export function toDeferredSurfaceShader(
  variant: ISurfaceVariant,
  inputs: ISurfaceInputs,
  uniforms: RendererUniforms
): ISurfaceShader {
  if (variant.isImpostor) {
    return toImpostorSurfaceShader(variant, inputs, uniforms);
  }

  const texel: ISurfaceTexel = toSurfaceTexel(variant, inputs, uniforms);
  const albedo: Node<"vec4"> = vec4(texel.albedo, texel.gloss);

  return {
    fragmentNode: toGBufferOutput(
      variant.draw === ERendererDraw.CUT_OUT
        ? toHashedAlphaCut({
            alpha: texel.coverage,
            output: albedo,
            reference: inputs.alphaReference,
            settings: uniforms.settings,
          })
        : albedo,
      texel.normal,
      texel.hemi,
      texel.sun,
      texel.slice,
      toSurfaceMotion(uniforms)
    ),
  };
}
