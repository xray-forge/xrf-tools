import { IRenderFrameCost } from "@/core/render/lib/frame/render-frame-cost";

/** Told what a viewport's frames are costing, a few times a second rather than every frame. */
export type TRenderCostReporter = (cost: IRenderFrameCost) => void;
