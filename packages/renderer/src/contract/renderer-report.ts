import { IRendererCameraPose } from "#/contract/renderer-camera";
import { IRenderFrameCost } from "#/frame/render-frame-cost";

/**
 * What one pass of the frame cost on the GPU.
 */
export interface IRendererPassCost {
  /** The pass, as the frame names it. */
  name: string;
  /** Mean GPU milliseconds over the report window; zero while no timing has resolved. */
  gpuTime: number;
}

/**
 * What the renderer says about its frames, a few times a second.
 */
export interface IRendererReport {
  /** Frame pacing and submission cost, measured on the thread that draws. */
  frame: IRenderFrameCost;
  /** GPU cost per pass, in frame order. */
  passes: ReadonlyArray<IRendererPassCost>;
  /** Whether the device grants timestamp queries, without which `passes` stays at zero. */
  isGpuTimed: boolean;
  /** Where the camera was when the report was taken. */
  camera: IRendererCameraPose;
}
