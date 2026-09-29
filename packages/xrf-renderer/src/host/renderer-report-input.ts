import { Nullable } from "@xrf/types";

import { IRendererCameraPose } from "#/contract/renderer-camera-pose";
import { IRendererLightsReport } from "#/contract/renderer-lights-report";
import { IRendererStaticDrawReport } from "#/contract/renderer-static-draw-report";
import { IRendererWeatherReport } from "#/contract/weather/renderer-weather-report";
import { RendererDevice } from "#/device/renderer-device";
import { IRendererFrameSize } from "#/sampling/renderer-frame-size";
import { IStaticCullCounts } from "#/scene/static/static-cull-counts";

/**
 * What a frame report is taken from, besides the frames' own timings.
 */
export interface IRendererReportInput {
  /** The device drawing. */
  device: RendererDevice;
  /** The canvas drawn on. */
  canvas: OffscreenCanvas;
  /** The frame's size, the scene's as drawn among it. */
  size: IRendererFrameSize;
  camera: IRendererCameraPose;
  /** The frame's passes, in frame order. */
  passes: ReadonlyArray<string>;
  /** What the static cull kept, which three's own counts leave out. */
  kept: IStaticCullCounts;
  staticDraws: IRendererStaticDrawReport;
  lights: IRendererLightsReport;
  /** Bytes the scene holds on the CPU of what it draws. */
  cpuMemory: number;
  weather: Nullable<IRendererWeatherReport>;
}
