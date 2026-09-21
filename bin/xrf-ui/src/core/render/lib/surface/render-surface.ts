import {
  AdditiveBlending,
  Blending,
  BlendingDstFactor,
  BlendingSrcFactor,
  CustomBlending,
  DstColorFactor,
  NormalBlending,
  OneFactor,
  SrcColorFactor,
  ZeroFactor,
} from "three";

import {
  EXraySurfaceDeclaration,
  EXraySurfaceDraw,
  XraySurfaceDeclaration,
  XraySurfaceDescriptor,
  XraySurfaceDraw,
} from "@/core/ipc/types/xrf-material";
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
 * Source times zero over destination times one: what is behind the surface, and nothing of the surface.
 */
const BLANK_BLENDING: IRenderBlending = {
  blendDst: OneFactor,
  blendSrc: ZeroFactor,
  blending: CustomBlending,
};

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
  /**  Whether the scene's light reaches the surface at all. */
  isLit: boolean;
  /** Whether the surface is a wall mark, which decides how its texture is sampled rather than how it is drawn. */
  isWallmark: boolean;
}

/** How a surface with nothing said about it is drawn, which is how the engine draws one whose shader it cannot find. */
export const OPAQUE_RENDER_SURFACE: IRenderSurface = {
  alphaTest: 0,
  blend: NORMAL_BLENDING,
  detail: null,
  isDepthWritten: true,
  isLit: true,
  isTransparent: false,
  isWallmark: false,
};

/** The range an alpha reference is stated in on the wire, so `200` becomes `200 / 255`. */
const ALPHA_REFERENCE_SCALE: number = 255;

/**
 * The alpha every cut-out deferred shader tests against, `def_aref`, two hundred of two hundred and fifty five
 * (`shaders/r2/common.h`).
 */
export const XRAY_DEFAULT_AREF: number = 200 / ALPHA_REFERENCE_SCALE;

/**
 * An engine alpha reference, as the threshold three.js discards against.
 *
 * The engine compares with `D3DCMP_GREATER` and keeps a texel whose alpha is greater than the reference; three.js
 * discards below `alphaTest` and keeps what is equal to it. One step of the eight bit channel is the difference, and
 * it is the whole difference at a reference of zero: `aref(true, 0)` discards every fully transparent texel, where
 * an `alphaTest` of zero discards nothing at all.
 *
 * @param reference - What the pass tests against.
 * @returns The threshold to discard below.
 */
export function toXrayAlphaTest(reference: number): number {
  return (reference + 1) / ALPHA_REFERENCE_SCALE;
}

/**
 * Turn one resolved surface into the material state that draws it.
 *
 * @param descriptor - What the backend resolved for the surface, or null when none was declared or resolved.
 * @returns The material state, opaque and undetailed for anything the backend could not describe.
 */
export function toRenderSurface(descriptor: Nullable<XraySurfaceDescriptor>): IRenderSurface {
  const draw: Nullable<XraySurfaceDraw> = descriptor?.draw ?? null;
  const detail: Nullable<IRenderDetail> = toRenderDetail(descriptor);
  const isLit: boolean = isLitRenderSurface(descriptor);
  const isWallmark: boolean = isWallmarkRenderSurface(descriptor);

  if (!draw || draw.kind === EXraySurfaceDraw.OPAQUE) {
    return detail || !isLit || isWallmark
      ? { ...OPAQUE_RENDER_SURFACE, detail, isLit, isWallmark }
      : OPAQUE_RENDER_SURFACE;
  }

  if (draw.kind === EXraySurfaceDraw.INVISIBLE) {
    return {
      alphaTest: 0,
      blend: BLANK_BLENDING,
      detail,
      isDepthWritten: false,
      isLit,
      isTransparent: true,
      isWallmark,
    };
  }

  if (draw.kind === EXraySurfaceDraw.ALPHA_TESTED) {
    return {
      ...OPAQUE_RENDER_SURFACE,
      alphaTest: toAlphaTest(draw, descriptor?.declaration),
      detail,
      isLit,
      isWallmark,
    };
  }

  // Everything else leaves the opaque pass, so none of it writes depth: a mark laid on a wall that wrote depth would
  // hide the wall it is a mark on, and two glows would cut holes in each other.
  return {
    alphaTest: toAlphaTest(draw, descriptor?.declaration),
    blend: toRenderBlending(draw),
    detail,
    isDepthWritten: false,
    isLit,
    isTransparent: true,
    isWallmark,
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

/**
 * Whether the surface is a wall mark, which its script says and which decides how its texture is sampled.
 *
 * @param descriptor - What the backend resolved for the surface.
 * @returns Whether the engine would bind it through the wall mark sampler.
 */
function isWallmarkRenderSurface(descriptor: Nullable<XraySurfaceDescriptor>): boolean {
  const declaration: Maybe<XraySurfaceDeclaration> = descriptor?.declaration;

  return declaration?.kind === EXraySurfaceDeclaration.SCRIPTED && declaration.isWallmark;
}

/**
 * Whether the scene's light reaches a surface, which only a scripted pass answers no to.
 *
 * @param descriptor - What the backend resolved for the surface.
 * @returns Whether to shade it with the scene's lights.
 */
function isLitRenderSurface(descriptor: Nullable<XraySurfaceDescriptor>): boolean {
  const declaration: Maybe<XraySurfaceDeclaration> = descriptor?.declaration;

  return !(declaration?.kind === EXraySurfaceDeclaration.SCRIPTED && declaration.isBlended);
}

/**
 * What a surface discards.
 */
function toAlphaTest(draw: XraySurfaceDraw, declaration: Maybe<XraySurfaceDeclaration>): number {
  if (declaration?.kind === EXraySurfaceDeclaration.SCRIPTED) {
    return declaration.isAlphaTested ? toXrayAlphaTest(declaration.alphaReference) : 0;
  }

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
