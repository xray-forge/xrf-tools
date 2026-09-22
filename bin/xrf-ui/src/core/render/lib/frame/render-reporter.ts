import { IRenderFrameCost } from "@xrf/renderer";

/** Told what a viewport's frames are costing, a few times a second rather than every frame. */
export type TRenderCostReporter = (cost: IRenderFrameCost) => void;
