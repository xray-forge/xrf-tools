import { IRenderFrameCost, TFrameRateLimit } from "@xrf/renderer";

import { IRenderLighting } from "@/core/render/lib/lighting/render-lighting";
import { ITextureSurfaceFiles, ITextureSurfaceOptions } from "@/core/textures/lib/texture-surface";

/** What a lit surface says back, whichever thread it is drawn on. */
export interface ITextureSurfaceRendererEvents {
  /** Where a drag over the body has put the light, which the viewer keeps. */
  onLighting(lighting: IRenderLighting): void;
  /** What frames are costing, a few times a second. */
  onReport(cost: IRenderFrameCost): void;
}

/**
 * Everything a lit surface is told.
 */
export interface ITextureSurfaceRenderer {
  /**
   * Takes the files to draw.
   *
   * @param files - The base and the pair, as they were read.
   */
  setTextures(files: ITextureSurfaceFiles): void;
  /**
   * Takes how the surface is being looked at.
   *
   * @param options - Shape, lighting, bump switch and tiling.
   */
  setOptions(options: ITextureSurfaceOptions): void;
  /**
   * Takes what the body is lit with.
   *
   * @param lighting - The direction, its strength and colour, and the fill.
   */
  setLighting(lighting: IRenderLighting): void;
  /**
   * Caps how often the surface is redrawn.
   *
   * @param limit - Frames a second to allow.
   */
  setFrameRateLimit(limit: TFrameRateLimit): void;
  /**
   * Swings the light by a drag across the body, answering with where it put it.
   *
   * @param deltaX - Horizontal movement in pixels.
   * @param deltaY - Vertical movement in pixels.
   */
  dragLight(deltaX: number, deltaY: number): void;
  /**
   * Moves the camera towards the body or away from it.
   *
   * @param step - What to multiply the distance by.
   */
  dolly(step: number): void;
  /** Back to the distance and the angle the body is first seen from. */
  reset(): void;
  /** Releases the renderer and everything it holds. */
  dispose(): void;
}
