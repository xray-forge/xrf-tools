import { IRendererPassCost } from "@/core/render/lib/contract/renderer-pass-cost";

/**
 * What each pass of the frame cost on the GPU.
 */
export interface IRendererPassTimings {
  /** GPU cost per pass, in frame order. */
  passes: ReadonlyArray<IRendererPassCost>;
  /** Whether passes are being timed: the device grants timestamp queries and the features ask for them. */
  isGpuTimed: boolean;
}

/** No pass timed, for a viewport that has not reported. */
export const EMPTY_RENDERER_PASS_TIMINGS: IRendererPassTimings = { isGpuTimed: false, passes: [] };
