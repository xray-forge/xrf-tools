import { Nullable } from "@xrf/types";
import {
  BlendingDstFactor,
  BlendingSrcFactor,
  CustomBlending,
  DstColorFactor,
  Material,
  OneFactor,
  OneMinusSrcAlphaFactor,
  SrcAlphaFactor,
  SrcColorFactor,
  ZeroFactor,
} from "three/webgpu";

import { ERendererDraw, IRendererSurface } from "#/contract/scene/renderer-surface";

/** Units a composited surface is pulled towards the eye by, scaled by its slope, so a decal never loses to its wall. */
const COMPOSITED_POLYGON_OFFSET: number = -1;

/**
 * How a composited surface blends into what is under it.
 */
export interface ISurfaceCompositing {
  source: BlendingSrcFactor;
  destination: BlendingDstFactor;
  alphaSource: BlendingSrcFactor;
  alphaDestination: BlendingDstFactor;
  /** Whether it writes colour at all: an invisible surface is submitted and drawn, writing nothing. */
  isColorWritten: boolean;
}

/**
 * Alpha keeps what is under it, except where a blended surface covers it: the canvas composites by it, and the
 * albedo keeps its gloss there.
 */
const KEPT_ALPHA: Pick<ISurfaceCompositing, "alphaSource" | "alphaDestination"> = {
  alphaDestination: OneFactor,
  alphaSource: ZeroFactor,
};

/**
 * @param surface - What a consumer puts, or as much of it as decides this.
 * @returns How it blends, or null for a surface the G-buffer takes whole.
 */
export function toSurfaceCompositing(
  surface: Pick<IRendererSurface, "draw" | "isWallmark">
): Nullable<ISurfaceCompositing> {
  const compositing: Nullable<ISurfaceCompositing> = toDrawCompositing(surface.draw);

  // `dx10color_write_enable(true, true, true, false)`: a wall mark leaves the gloss in the albedo's alpha alone.
  return compositing && surface.isWallmark ? { ...compositing, ...KEPT_ALPHA } : compositing;
}

/**
 * What every composited surface shares: no depth written, pulled towards the eye, and blended as it says.
 *
 * @param material - The surface's material.
 * @param compositing - How it blends.
 */
export function applySurfaceCompositing(material: Material, compositing: ISurfaceCompositing): void {
  material.depthWrite = false;
  material.transparent = true;
  material.polygonOffset = true;
  material.polygonOffsetFactor = COMPOSITED_POLYGON_OFFSET;
  material.polygonOffsetUnits = COMPOSITED_POLYGON_OFFSET;
  material.blending = CustomBlending;
  material.blendSrc = compositing.source;
  material.blendDst = compositing.destination;
  material.blendSrcAlpha = compositing.alphaSource;
  material.blendDstAlpha = compositing.alphaDestination;
  material.colorWrite = compositing.isColorWritten;
}

function toDrawCompositing(draw: ERendererDraw): Nullable<ISurfaceCompositing> {
  switch (draw) {
    case ERendererDraw.OPAQUE:
    case ERendererDraw.CUT_OUT:
      return null;

    case ERendererDraw.BLENDED:
      return {
        alphaDestination: OneMinusSrcAlphaFactor,
        alphaSource: OneFactor,
        destination: OneMinusSrcAlphaFactor,
        isColorWritten: true,
        source: SrcAlphaFactor,
      };

    case ERendererDraw.ADDED:
      return { ...KEPT_ALPHA, destination: OneFactor, isColorWritten: true, source: OneFactor };

    case ERendererDraw.ALPHA_ADDED:
      return { ...KEPT_ALPHA, destination: OneFactor, isColorWritten: true, source: SrcAlphaFactor };

    case ERendererDraw.MULTIPLIED:
      return { ...KEPT_ALPHA, destination: ZeroFactor, isColorWritten: true, source: DstColorFactor };

    case ERendererDraw.MULTIPLIED_2X:
      return { ...KEPT_ALPHA, destination: SrcColorFactor, isColorWritten: true, source: DstColorFactor };

    case ERendererDraw.INVISIBLE:
      return { ...KEPT_ALPHA, destination: ZeroFactor, isColorWritten: false, source: OneFactor };
  }
}
