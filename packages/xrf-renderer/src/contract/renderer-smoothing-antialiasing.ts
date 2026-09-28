import { ERendererAntialiasing } from "#/contract/renderer-antialiasing";

/** A mode smoothing the finished frame alone. */
export type TRendererSmoothingAntialiasing = ERendererAntialiasing.FXAA | ERendererAntialiasing.SMAA;

/**
 * @param mode - An antialiasing mode.
 * @returns Whether it smooths the finished frame alone.
 */
export function isRendererSmoothing(mode: ERendererAntialiasing): mode is TRendererSmoothingAntialiasing {
  return mode === ERendererAntialiasing.FXAA || mode === ERendererAntialiasing.SMAA;
}
