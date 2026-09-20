import { EXraySurfaceDraw, XraySurfaceDescriptor, XraySurfaceDraw } from "@/core/ipc/types/xrf-material";
import { Nullable } from "@/lib/types/general";

/**
 * The material state one surface is drawn with, translated out of what the engine compiles for its shader.
 */
export interface IRenderSurface {
  /**  Alpha below which a texel is discarded, `0` for a surface that reads no alpha. */
  alphaTest: number;
  /**  Whether the surface is composited over what is behind it rather than written opaquely. */
  isTransparent: boolean;
  /**  Whether the surface writes depth. */
  isDepthWritten: boolean;
}

/** How a surface with nothing said about it is drawn, which is how the engine draws one whose shader it cannot find. */
export const OPAQUE_RENDER_SURFACE: IRenderSurface = {
  alphaTest: 0,
  isDepthWritten: true,
  isTransparent: false,
};

/** The range an alpha reference is stated in on the wire, so `200` becomes `200 / 255`. */
const ALPHA_REFERENCE_SCALE: number = 255;

/**
 * Turn one resolved surface into the material state that draws it.
 *
 * @param descriptor - What the backend resolved for the shader name, or null when none was declared or resolved.
 * @returns The material state, opaque for anything the backend could not describe.
 */
export function toRenderSurface(descriptor: Nullable<XraySurfaceDescriptor>): IRenderSurface {
  const draw: Nullable<XraySurfaceDraw> = descriptor?.draw ?? null;

  if (!draw || draw.kind === EXraySurfaceDraw.OPAQUE) {
    return OPAQUE_RENDER_SURFACE;
  }

  return {
    alphaTest: draw.reference / ALPHA_REFERENCE_SCALE,
    isDepthWritten: draw.kind !== EXraySurfaceDraw.BLENDED,
    isTransparent: draw.kind === EXraySurfaceDraw.BLENDED,
  };
}

/**
 * Whether a material state reads its texture's alpha channel at all.
 *
 * @param surface - Material state of one surface.
 * @returns Whether anything is discarded or composited.
 */
export function isAlphaRenderSurface(surface: IRenderSurface): boolean {
  return surface.isTransparent || surface.alphaTest > 0;
}

/**
 * The state a shader name comes to, for a caller holding the whole table the backend resolved.
 *
 * @param surfaces - What the backend resolved, by shader name.
 * @param shaderName - The name a surface declares, or null for one that declares none.
 * @returns The material state, opaque when the name is absent from the table.
 */
export function getRenderSurface(
  surfaces: Readonly<Record<string, XraySurfaceDescriptor>>,
  shaderName: Nullable<string>
): IRenderSurface {
  return toRenderSurface(shaderName ? (surfaces[shaderName] ?? null) : null);
}
