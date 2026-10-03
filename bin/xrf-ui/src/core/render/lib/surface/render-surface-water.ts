import { Nullable } from "@xrf/types";

import { IRenderAnomalyWater } from "@/core/render/lib/surface/render-anomaly-water";

/**
 * How a water surface is drawn, as its script's programs say.
 */
export interface IRenderSurfaceWater {
  /** A program that blends over what is behind it by how deep the water there is. Plain `water` is drawn whole. */
  isSoft: boolean;
  /** Anomaly's model where the program is one of its own, or null for OpenXRay's. */
  anomaly: Nullable<IRenderAnomalyWater>;
}
