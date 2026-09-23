import { ERendererDraw } from "@xrf/renderer";
import { Maybe, Nullable } from "@xrf/types";

import {
  EXraySurfaceDeclaration,
  EXraySurfaceDraw,
  XraySurfaceDeclaration,
  XraySurfaceDescriptor,
  XraySurfaceDraw,
} from "@/core/ipc/types/xrf-material";

/** What an alpha reference is stated out of. */
const ALPHA_REFERENCE_SCALE: number = 255;

/**
 * How a surface reaches the renderer's frame, from what the backend resolved for its shader.
 */
export interface IRendererSurfaceDraw {
  draw: ERendererDraw;
  /** The authored reference in `[0, 1]`, where the blender states one. */
  alphaReference?: number;
  /** Whether the scene's light reaches it, which only a scripted blended pass answers no to. */
  isLit: boolean;
}

/** A surface whose shader nobody resolved: drawn opaque and lit, as the plain base shader is. */
export const OPAQUE_RENDERER_SURFACE_DRAW: IRendererSurfaceDraw = { draw: ERendererDraw.OPAQUE, isLit: true };

/**
 * @param descriptor - What the backend resolved for the surface, or nothing.
 * @returns How the renderer draws it.
 */
export function toRendererSurfaceDraw(descriptor: Nullable<XraySurfaceDescriptor>): IRendererSurfaceDraw {
  const draw: Nullable<XraySurfaceDraw> = descriptor?.draw ?? null;
  const isLit: boolean = isLitSurface(descriptor);

  if (!draw) {
    return { ...OPAQUE_RENDERER_SURFACE_DRAW, isLit };
  }

  const alphaReference: Maybe<number> = "reference" in draw ? draw.reference / ALPHA_REFERENCE_SCALE : undefined;

  switch (draw.kind) {
    case EXraySurfaceDraw.OPAQUE:
      return { draw: ERendererDraw.OPAQUE, isLit };

    case EXraySurfaceDraw.ALPHA_TESTED:
      return { alphaReference, draw: ERendererDraw.CUT_OUT, isLit };

    case EXraySurfaceDraw.BLENDED:
      return { alphaReference, draw: ERendererDraw.BLENDED, isLit };

    case EXraySurfaceDraw.ADDED:
      return { alphaReference, draw: ERendererDraw.ADDED, isLit };

    case EXraySurfaceDraw.MULTIPLIED:
      return { draw: draw.isDoubled ? ERendererDraw.MULTIPLIED_2X : ERendererDraw.MULTIPLIED, isLit };

    case EXraySurfaceDraw.INVISIBLE:
      return { draw: ERendererDraw.INVISIBLE, isLit };

    default:
      return { ...OPAQUE_RENDERER_SURFACE_DRAW, isLit };
  }
}

/**
 * @param surfaces - What the backend resolved, in the order the model or level declares its surfaces.
 * @param index - The surface's position.
 * @returns How the renderer draws it.
 */
export function getRendererSurfaceDraw(
  surfaces: ReadonlyArray<XraySurfaceDescriptor>,
  index: number
): IRendererSurfaceDraw {
  return toRendererSurfaceDraw(surfaces[index] ?? null);
}

/**
 * @param surface - How a surface is drawn.
 * @returns Whether its draw reads the base texture's alpha.
 */
export function isAlphaRendererSurfaceDraw(surface: IRendererSurfaceDraw): boolean {
  return surface.draw !== ERendererDraw.OPAQUE;
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
