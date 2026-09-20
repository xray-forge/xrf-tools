import {
  AdditiveBlending,
  Blending,
  BlendingDstFactor,
  BlendingSrcFactor,
  CustomBlending,
  DstColorFactor,
  NormalBlending,
  SrcColorFactor,
  ZeroFactor,
} from "three";

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
 * How a surface is composited over what is already drawn.
 */
export interface IRenderBlending {
  blending: Blending;
  /** Set only for {@link CustomBlending}, where three.js takes the factors rather than naming the equation. */
  blendSrc: Maybe<BlendingSrcFactor>;
  blendDst: Maybe<BlendingDstFactor>;
}

const NORMAL_BLENDING: IRenderBlending = { blendDst: undefined, blendSrc: undefined, blending: NormalBlending };

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
  /**  How it is composited, for the surfaces that are. */
  blend: IRenderBlending;
  /**  The detail texture bound beside the diffuse, or null for a surface the engine details with none. */
  detail: Nullable<IRenderDetail>;
}

/** How a surface with nothing said about it is drawn, which is how the engine draws one whose shader it cannot find. */
export const OPAQUE_RENDER_SURFACE: IRenderSurface = {
  alphaTest: 0,
  blend: NORMAL_BLENDING,
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

  if (draw.kind === EXraySurfaceDraw.ALPHA_TESTED) {
    return {
      ...OPAQUE_RENDER_SURFACE,
      alphaTest: draw.reference / ALPHA_REFERENCE_SCALE,
      detail,
    };
  }

  // Everything else leaves the opaque pass, so none of it writes depth: a mark laid on a wall that wrote depth would
  // hide the wall it is a mark on, and two glows would cut holes in each other.
  return {
    alphaTest: toAlphaTest(draw),
    blend: toRenderBlending(draw),
    detail,
    isDepthWritten: false,
    isTransparent: true,
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

/** What a composited surface discards, which the multiplying equations state no reference for. */
function toAlphaTest(draw: XraySurfaceDraw): number {
  return "reference" in draw ? draw.reference / ALPHA_REFERENCE_SCALE : 0;
}

/** The equation a composited surface is drawn with. */
function toRenderBlending(draw: XraySurfaceDraw): IRenderBlending {
  switch (draw.kind) {
    case EXraySurfaceDraw.ADDED:
      return { blendDst: undefined, blendSrc: undefined, blending: AdditiveBlending };

    case EXraySurfaceDraw.MULTIPLIED:
      // `MUL` is `DESTCOLOR, ZERO` and `MUL_2X` is `DESTCOLOR, SRCCOLOR`, which three.js has no named blending for.
      return {
        blendDst: draw.isDoubled ? SrcColorFactor : ZeroFactor,
        blendSrc: DstColorFactor,
        blending: CustomBlending,
      };

    default:
      return NORMAL_BLENDING;
  }
}

/** The detail the descriptor names, dropped where it carries no tiling, since none can be invented for it. */
function toRenderDetail(descriptor: Nullable<XraySurfaceDescriptor>): Nullable<IRenderDetail> {
  const detail: Maybe<XraySurfaceDescriptor["detail"]> = descriptor?.detail;

  return detail && detail.scale !== null ? { reference: detail.reference, scale: detail.scale } : null;
}
