import { Maybe } from "@xrf/types";
import { Discard, Fn, If, uniform, vec4 } from "three/tsl";
import { Node } from "three/webgpu";

import { ERendererDraw, IRendererSurface } from "#/contract/scene/renderer-surface";
import { toImpostorSurfaceShader } from "#/material/impostor-surface.tsl";
import { MaterialSamplers } from "#/material/material-samplers";
import { ISurfaceShader } from "#/material/surface-shader";
import { ISurfaceTexel } from "#/material/surface-texel";
import { toSurfaceTexel } from "#/material/surface-texel.tsl";
import { toGBufferOutput } from "#/shader/gbuffer.tsl";
import { RendererUniforms } from "#/uniforms/renderer-uniforms";

/** `def_aref`: where a cut-out surface without its own reference is cut. */
const DEFAULT_ALPHA_REFERENCE: number = 200 / 255;

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
      surface.draw === ERendererDraw.CUT_OUT ? toCutOut(texel.alpha, albedo, surface.alphaReference) : albedo,
      texel.normal,
      texel.hemi,
      texel.sun,
      texel.slice
    ),
  };
}

/**
 * `clip(D.w - def_aref)`: an output that discards the texel first where the base's alpha is at or below the
 * reference. Only a cut-out surface reads that alpha, so a DXT1 file's punch-through never holes an opaque one.
 */
function toCutOut(alpha: Node<"float">, output: Node<"vec4">, reference: Maybe<number>): Node<"vec4"> {
  return Fn(() => {
    If(alpha.lessThanEqual(uniform(reference ?? DEFAULT_ALPHA_REFERENCE)), () => {
      Discard();
    });

    return output;
  })();
}
