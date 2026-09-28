import { IRendererCameraPose } from "#/contract/renderer-camera-pose";
import { IRendererLightsReport } from "#/contract/renderer-lights-report";
import { IRendererPassTimings } from "#/contract/renderer-pass-timings";
import { IRendererStaticDrawReport } from "#/contract/renderer-static-draw-report";
import { IRenderFrameCost } from "#/frame/render-frame-cost";

/**
 * What the renderer says about its frames, a few times a second.
 */
export interface IRendererReport extends IRendererPassTimings {
  /** Frame pacing and submission cost, measured on the thread that draws. */
  frame: IRenderFrameCost;
  /** Where the camera was when the report was taken. */
  camera: IRendererCameraPose;
  /** How full the static draws' pools are and what occlusion removed. */
  staticDraws: IRendererStaticDrawReport;
  lights: IRendererLightsReport;
  /** Bytes the renderer holds on the CPU of what it draws, each buffer counted once. */
  cpuMemory: number;
}
