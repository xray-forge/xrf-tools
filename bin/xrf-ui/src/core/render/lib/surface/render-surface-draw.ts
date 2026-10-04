import { assertExhaustive, Maybe, Nullable } from "@xrf/types";

import {
  EXraySurfaceDeclaration,
  EXraySurfaceDraw,
  XraySurfaceDeclaration,
  XraySurfaceDescriptor,
  XraySurfaceDraw,
} from "@/core/ipc/types/xrf-material";
import { ERenderDraw } from "@/core/render/lib/surface/render-draw";

/**
 * How a surface reaches the renderer's frame, from what the backend resolved for its shader.
 */
export interface IRenderSurfaceDraw {
  draw: ERenderDraw;
  /** Whether the scene's light reaches it, which only a scripted blended pass answers no to. */
  isLit: boolean;
}

/** A surface whose shader nobody resolved: drawn opaque and lit, as the plain base shader is. */
export const OPAQUE_RENDER_SURFACE_DRAW: IRenderSurfaceDraw = { draw: ERenderDraw.OPAQUE, isLit: true };

/**
 * @param descriptor - What the backend resolved for the surface, or nothing.
 * @returns How the renderer draws it.
 */
export function toRenderSurfaceDraw(descriptor: Nullable<XraySurfaceDescriptor>): IRenderSurfaceDraw {
  const draw: Nullable<XraySurfaceDraw> = descriptor?.draw ?? null;
  const isLit: boolean = isLitSurface(descriptor);

  if (!draw) {
    return { ...OPAQUE_RENDER_SURFACE_DRAW, isLit };
  }

  const rendererDraw: ERenderDraw = toRenderDraw(draw);

  // Water is lit by its own programs, whatever its blend says.
  return { draw: rendererDraw, isLit: rendererDraw === ERenderDraw.WATER || isLit };
}

/**
 * @param draw - How the backend resolved a shader to composite.
 * @returns The same, as the renderer draws it; opaque for a kind it does not know.
 */
export function toRenderDraw(draw: XraySurfaceDraw): ERenderDraw {
  switch (draw.kind) {
    case EXraySurfaceDraw.ALPHA_TESTED:
      return ERenderDraw.CUT_OUT;

    case EXraySurfaceDraw.BLENDED:
      return ERenderDraw.BLENDED;

    case EXraySurfaceDraw.ADDED:
      return draw.isWeighted ? ERenderDraw.ALPHA_ADDED : ERenderDraw.ADDED;

    case EXraySurfaceDraw.MULTIPLIED:
      return draw.isDoubled ? ERenderDraw.MULTIPLIED_2X : ERenderDraw.MULTIPLIED;

    case EXraySurfaceDraw.INVISIBLE:
      return ERenderDraw.INVISIBLE;

    case EXraySurfaceDraw.WATER:
      return ERenderDraw.WATER;

    default:
      return ERenderDraw.OPAQUE;
  }
}

/**
 * @param surfaces - What the backend resolved, in the order the model or level declares its surfaces.
 * @param index - The surface's position.
 * @returns How the renderer draws it.
 */
export function getRenderSurfaceDraw(
  surfaces: ReadonlyArray<XraySurfaceDescriptor>,
  index: number
): IRenderSurfaceDraw {
  return toRenderSurfaceDraw(surfaces[index] ?? null);
}

/**
 * @param surface - How a surface is drawn.
 * @returns Whether its draw reads the base texture's alpha: to cut it out, to blend it, or to weigh what it adds.
 */
export function isAlphaRenderSurfaceDraw(surface: IRenderSurfaceDraw): boolean {
  switch (surface.draw) {
    case ERenderDraw.CUT_OUT:
    case ERenderDraw.BLENDED:
    case ERenderDraw.ALPHA_ADDED:
      return true;

    case ERenderDraw.OPAQUE:
    case ERenderDraw.ADDED:
    case ERenderDraw.MULTIPLIED:
    case ERenderDraw.MULTIPLIED_2X:
    case ERenderDraw.INVISIBLE:
    case ERenderDraw.WATER:
      return false;

    default:
      return assertExhaustive(surface.draw);
  }
}

/**
 * Whether the scene's light reaches a surface, which only a scripted blended pass answers no to.
 *
 * @param descriptor - What the backend resolved for the surface.
 * @returns Whether to shade it with the scene's lights.
 */
export function isLitSurface(descriptor: Nullable<XraySurfaceDescriptor>): boolean {
  const declaration: Maybe<XraySurfaceDeclaration> = descriptor?.declaration;

  return !(declaration?.kind === EXraySurfaceDeclaration.SCRIPTED && declaration.isBlended);
}

/**
 * Whether a surface is a wall mark, which a scripted pass says it is.
 *
 * @param descriptor - What the backend resolved for the surface.
 * @returns Whether it is a wall mark.
 */
export function isWallmarkSurface(descriptor: Nullable<XraySurfaceDescriptor>): boolean {
  const declaration: Maybe<XraySurfaceDeclaration> = descriptor?.declaration;

  return declaration?.kind === EXraySurfaceDeclaration.SCRIPTED && declaration.isWallmark;
}
