import { mix, positionView, uniform, vec3, vec4 } from "three/tsl";
import { Node } from "three/webgpu";

import { ERendererDraw, IRendererSurface } from "#/contract/scene/renderer-surface";
import { MaterialSamplers } from "#/material/material-samplers";
import { ISurfaceShader } from "#/material/surface-shader";
import { ISurfaceTexel } from "#/material/surface-texel";
import { toSurfaceTexel } from "#/material/surface-texel.tsl";
import { toBaseLitColor, toFogAmount, toFogColor, toSunLight } from "#/shader/base-lighting.tsl";
import { IBaseShadingPoint } from "#/shader/base-shading-point";
import { RendererUniforms } from "#/uniforms/renderer-uniforms";

/**
 * A surface composited over the tonemapped frame. Blended surfaces are lit per pixel by the deferred passes' model,
 * bump included; added and multiplied stay unlit. Every one fades into the fog as the engine's forward shaders do:
 * blended towards the fog's colour, added towards nothing, multiplied towards leaving what is under it alone.
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
  } else {
    // Raw albedo, as the frame shows it unlit, takes no fog either.
    color = mix(
      color,
      toFoggedColor(surface.draw, uniforms),
      toFogAmount(positionView, uniforms).mul(uniforms.settings.lit)
    );
  }

  return {
    alphaTestNode: surface.alphaReference === undefined ? undefined : uniform(surface.alphaReference),
    colorNode: vec4(color, texel.alpha),
  };
}

/** What an unlit surface of a draw comes to in total fog: whatever leaves the frame under it as the fog made it. */
function toFoggedColor(draw: ERendererDraw, uniforms: RendererUniforms): Node<"vec3"> {
  switch (draw) {
    case ERendererDraw.ADDED:
    case ERendererDraw.ALPHA_ADDED:
      return vec3(0);
    case ERendererDraw.MULTIPLIED:
      return vec3(1);
    case ERendererDraw.MULTIPLIED_2X:
      return vec3(0.5);
    default:
      return toFogColor(uniforms);
  }
}
