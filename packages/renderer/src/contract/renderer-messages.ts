import { IRendererDevice } from "#/contract/renderer-device";
import { IRendererReport } from "#/contract/renderer-report";
import { IOffscreenRenderSize } from "#/frame/offscreen-render-target";
import { TFrameRateLimit } from "#/frame/render-frame-limit";

/**
 * What a consumer tells the renderer.
 */
export enum ERendererRequest {
  /** Here is the canvas, how big it is and how to draw on it: nothing is drawn before this. */
  START = "@renderer/start",
  /** The element the canvas fills is a different size, or draws a different number of pixels. */
  RESIZE = "@renderer/resize",
  /** The consumer's settings for drawing changed. */
  CONFIGURE = "@renderer/configure",
  /** Let everything go. */
  DISPOSE = "@renderer/dispose",
}

/**
 * What the renderer tells its consumer.
 */
export enum ERendererResponse {
  /** The device is up and the first frame is scheduled. */
  READY = "@renderer/ready",
  /** The renderer cannot draw here, and why; there is no fallback. */
  FAILED = "@renderer/failed",
  /** What the frames have been costing. */
  REPORT = "@renderer/report",
}

/**
 * How the consumer wants frames drawn.
 */
export interface IRendererConfiguration {
  /** How often a frame may be drawn. */
  frameRateLimit: TFrameRateLimit;
  /** What the canvas shows where nothing is drawn, as a hex colour. */
  backdrop: number;
}

/** Every message a consumer sends. */
export type TRendererRequest =
  | ({
      kind: ERendererRequest.START;
      canvas: OffscreenCanvas;
      configuration: IRendererConfiguration;
    } & IOffscreenRenderSize)
  | ({ kind: ERendererRequest.RESIZE } & IOffscreenRenderSize)
  | { kind: ERendererRequest.CONFIGURE; configuration: IRendererConfiguration }
  | { kind: ERendererRequest.DISPOSE };

/** Every message the renderer sends. */
export type TRendererResponse =
  | { kind: ERendererResponse.READY; device: IRendererDevice }
  | { kind: ERendererResponse.FAILED; reason: string }
  | { kind: ERendererResponse.REPORT; report: IRendererReport };

/**
 * What a request carries that has to be moved rather than copied.
 *
 * @param request - The message about to be posted.
 * @returns What to move with it.
 */
export function listRendererTransfers(request: TRendererRequest): Array<Transferable> {
  return request.kind === ERendererRequest.START ? [request.canvas] : [];
}
