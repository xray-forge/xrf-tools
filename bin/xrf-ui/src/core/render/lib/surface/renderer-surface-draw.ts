import { ERendererDraw, IRendererAnomalyWater, IRendererSurfaceWater } from "@xrf/renderer";
import { assertExhaustive, Maybe, Nullable } from "@xrf/types";

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
  /** How a surface drawn as water is drawn. */
  water?: IRendererSurfaceWater;
}

/**
 * Anomaly's water programs (`shaders/r2/water_*.ps` in its gamedata), by the switches each defines before including
 * its `water.ps`. Any other water program is OpenXRay's.
 */
const ANOMALY_WATER_PROGRAMS: Readonly<Record<string, IRendererAnomalyWater>> = {
  water_regular: { isFoamed: false, isReflecting: true, isSpecular: true, isTransparent: false },
  water_ryaska: { isFoamed: false, isReflecting: true, isSpecular: true, isTransparent: true },
  water_studen: { isFoamed: true, isReflecting: true, isSpecular: true, isTransparent: false },
  water_underground: { isFoamed: false, isReflecting: false, isSpecular: false, isTransparent: false },
};

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
      return { alphaReference, draw: draw.isWeighted ? ERendererDraw.ALPHA_ADDED : ERendererDraw.ADDED, isLit };

    case EXraySurfaceDraw.MULTIPLIED:
      return { draw: draw.isDoubled ? ERendererDraw.MULTIPLIED_2X : ERendererDraw.MULTIPLIED, isLit };

    case EXraySurfaceDraw.INVISIBLE:
      return { draw: ERendererDraw.INVISIBLE, isLit };

    // Lit by the water's own programs, whatever its blend says.
    case EXraySurfaceDraw.WATER:
      return { draw: ERendererDraw.WATER, isLit: true, water: toRendererSurfaceWater(descriptor, draw.isSoft) };

    default:
      return { ...OPAQUE_RENDERER_SURFACE_DRAW, isLit };
  }
}

/**
 * @param descriptor - What the backend resolved for a water surface.
 * @param isSoft - Whether its program blends over the depth behind it.
 * @returns How the renderer draws it: by Anomaly's model for one of Anomaly's programs, OpenXRay's for any other.
 */
function toRendererSurfaceWater(descriptor: Nullable<XraySurfaceDescriptor>, isSoft: boolean): IRendererSurfaceWater {
  const declaration: Maybe<XraySurfaceDeclaration> = descriptor?.declaration;
  const program: Maybe<string> =
    declaration?.kind === EXraySurfaceDeclaration.SCRIPTED ? declaration.program : undefined;

  return { anomaly: (program && ANOMALY_WATER_PROGRAMS[program]) || null, isSoft };
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
 * @returns Whether its draw reads the base texture's alpha: to cut it out, to blend it, or to weigh what it adds.
 */
export function isAlphaRendererSurfaceDraw(surface: IRendererSurfaceDraw): boolean {
  switch (surface.draw) {
    case ERendererDraw.CUT_OUT:
    case ERendererDraw.BLENDED:
    case ERendererDraw.ALPHA_ADDED:
      return true;

    case ERendererDraw.OPAQUE:
    case ERendererDraw.ADDED:
    case ERendererDraw.MULTIPLIED:
    case ERendererDraw.MULTIPLIED_2X:
    case ERendererDraw.INVISIBLE:
    case ERendererDraw.WATER:
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
