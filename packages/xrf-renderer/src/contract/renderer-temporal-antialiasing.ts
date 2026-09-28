import { ERendererAntialiasing } from "#/contract/renderer-antialiasing";

/** A mode resolving jittered frames with their history. */
export type TRendererTemporalAntialiasing = ERendererAntialiasing.TAA | ERendererAntialiasing.FSR2;

/**
 * @param mode - An antialiasing mode.
 * @returns Whether it resolves jittered frames with their history, which jitters every scene pass.
 */
export function isRendererTemporal(mode: ERendererAntialiasing): mode is TRendererTemporalAntialiasing {
  return mode === ERendererAntialiasing.TAA || mode === ERendererAntialiasing.FSR2;
}
