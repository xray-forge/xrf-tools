import { EXraySurfaceDraw, XraySurfaceDescriptor, XraySurfaceDraw } from "@/core/ipc/types/xrf-material";
import { Maybe, Nullable } from "@/lib/types/general";

/**
 * The detail texture a surface modulates its diffuse with, as the shader table's answer named it.
 */
export interface IRenderDetail {
  /** Texture reference, engine-style, to be looked up wherever the drawer's textures come from. */
  reference: string;
  /** Times it repeats across the surface's base coordinate. */
  scale: number;
}

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
  /**  The detail texture bound beside the diffuse, or null for a surface the engine details with none. */
  detail: Nullable<IRenderDetail>;
}

/** How a surface with nothing said about it is drawn, which is how the engine draws one whose shader it cannot find. */
export const OPAQUE_RENDER_SURFACE: IRenderSurface = {
  alphaTest: 0,
  detail: null,
  isDepthWritten: true,
  isTransparent: false,
};

/** The range an alpha reference is stated in on the wire, so `200` becomes `200 / 255`. */
const ALPHA_REFERENCE_SCALE: number = 255;

/**
 * Turn one resolved surface into the material state that draws it.
 *
 * @param descriptor - What the backend resolved for the surface, or null when none was declared or resolved.
 * @returns The material state, opaque and undetailed for anything the backend could not describe.
 */
export function toRenderSurface(descriptor: Nullable<XraySurfaceDescriptor>): IRenderSurface {
  const draw: Nullable<XraySurfaceDraw> = descriptor?.draw ?? null;
  const detail: Nullable<IRenderDetail> = toRenderDetail(descriptor);

  if (!draw || draw.kind === EXraySurfaceDraw.OPAQUE) {
    return detail ? { ...OPAQUE_RENDER_SURFACE, detail } : OPAQUE_RENDER_SURFACE;
  }

  return {
    alphaTest: draw.reference / ALPHA_REFERENCE_SCALE,
    detail,
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
 * The state one surface comes to, for a caller holding the whole table the backend resolved.
 *
 * @param surfaces - What the backend resolved, in the order of whatever declared them.
 * @param index - Position of the declaring thing, which is a level surface's shader id.
 * @returns The material state, opaque for an index the table does not reach.
 */
export function getRenderSurface(surfaces: ReadonlyArray<XraySurfaceDescriptor>, index: number): IRenderSurface {
  return toRenderSurface(surfaces[index] ?? null);
}

/** The detail the descriptor names, dropped where it carries no tiling, since none can be invented for it. */
function toRenderDetail(descriptor: Nullable<XraySurfaceDescriptor>): Nullable<IRenderDetail> {
  const detail: Maybe<XraySurfaceDescriptor["detail"]> = descriptor?.detail;

  return detail && detail.scale !== null ? { reference: detail.reference, scale: detail.scale } : null;
}
