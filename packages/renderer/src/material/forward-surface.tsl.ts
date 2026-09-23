import { positionView, uniform, vec4 } from "three/tsl";
import { Node } from "three/webgpu";

import { ERendererDraw, IRendererSurface } from "#/contract/scene/renderer-surface";
import { MaterialSamplers } from "#/material/material-samplers";
import { ISurfaceShader } from "#/material/surface-shader";
import { ISurfaceTexel } from "#/material/surface-texel";
import { toSurfaceTexel } from "#/material/surface-texel.tsl";
import { toBaseLitColor, toSunLight } from "#/shader/base-lighting.tsl";
import { IBaseShadingPoint } from "#/shader/base-shading-point";
import { RendererUniforms } from "#/uniforms/renderer-uniforms";

/**
 * A surface composited over the tonemapped frame. Blended surfaces are lit per pixel by the deferred passes' model,
 * bump included; added and multiplied stay unlit.
 *
 * @param surface - The surface drawn.
 * @param samplers - Where its slots are bound.
 * @param uniforms - What the frame's shaders read.
 * @returns Its shader.
 */
export function toForwardSurfaceShader(
  surface: IRendererSurface,
  samplers: MaterialSamplers,
  uniforms: RendererUniforms
): ISurfaceShader {
  const texel: ISurfaceTexel = toSurfaceTexel(surface, samplers, uniforms);
  let color: Node<"vec3"> = texel.albedo;

  if (surface.draw === ERendererDraw.BLENDED && surface.isLit !== false) {
    const point: IBaseShadingPoint = { normal: texel.normal, position: positionView, slice: texel.slice };

    color = toBaseLitColor(texel.albedo, texel.gloss, texel.hemi, toSunLight(point, uniforms), point, uniforms);
  }

  return {
    alphaTestNode: surface.alphaReference === undefined ? undefined : uniform(surface.alphaReference),
    colorNode: vec4(color, texel.alpha),
  };
}
