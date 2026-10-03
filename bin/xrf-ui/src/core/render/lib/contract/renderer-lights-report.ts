import { IRendererPoolUse } from "@/core/render/lib/contract/renderer-pool-use";

/**
 * What the local lights came to in the last frame.
 */
export interface IRendererLightsReport {
  /** Lights standing in view, binned and lit: a shadowed one only once its faces are drawn. */
  inView: number;
  /** Of them, the ones drawn with their shadows. */
  shadowed: number;
  /** Lights in view past the most a frame holds, the farthest, left unlit. */
  excessLights: number;
  /** Texels of the shadow atlas the faces hold. */
  atlas: IRendererPoolUse;
  /** What every wanted shadow size is scaled by, below one while the atlas is short of room. */
  shadowScale: number;
  /** Clusters more lights reached than they hold, as the last binning read back found them. */
  fullClusters: number;
  /** Lights left out of a cluster they reached, as the last binning read back found them: one left out of two counts twice. */
  droppedLights: number;
}

/** No lights, which is what a renderer reports before it has any. */
export const EMPTY_RENDERER_LIGHTS_REPORT: IRendererLightsReport = {
  atlas: { capacity: 0, used: 0 },
  droppedLights: 0,
  excessLights: 0,
  fullClusters: 0,
  inView: 0,
  shadowScale: 1,
  shadowed: 0,
};
