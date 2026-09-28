import { Nullable } from "@xrf/types";

import { IRendererAnomalyWater } from "#/contract/scene/renderer-anomaly-water";

/**
 * How a water surface is drawn, as its script's programs say.
 */
export interface IRendererSurfaceWater {
  /** A program that blends over what is behind it by how deep the water there is. Plain `water` is drawn whole. */
  isSoft: boolean;
  /** Anomaly's model where the program is one of its own, or null for OpenXRay's. */
  anomaly: Nullable<IRendererAnomalyWater>;
}
