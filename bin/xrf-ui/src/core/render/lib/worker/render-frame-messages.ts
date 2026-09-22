import { IOffscreenRenderSize } from "@/core/render/lib/frame/offscreen-render-target";
import { IRenderInputEvent } from "@/core/render/lib/worker/render-input";

/**
 * What every viewport drawn on another thread is told, whatever it draws.
 */
export enum ERenderFrame {
  /** Here is the canvas, and how big it is: everything before this has nowhere to go. */
  START = "@frame/start",
  /** The element the canvas fills is a different size, or draws a different number of pixels. */
  RESIZE = "@frame/resize",
  /** Somebody did this to the canvas, which only the thread holding it can see. */
  INPUT = "@frame/input",
  /** Let everything go. */
  DISPOSE = "@frame/dispose",
}

/** What it says back about the frame rather than about what is drawn in it. */
export enum ERenderFrameResponse {
  /** What the controls want the cursor to be, which only the side with a canvas can show. */
  CURSOR = "@frame/cursor",
}

/**
 * The messages the frame is made of.
 */
export type TRenderFrameRequest =
  | ({ kind: ERenderFrame.START; canvas: OffscreenCanvas } & IOffscreenRenderSize)
  | ({ kind: ERenderFrame.RESIZE } & IOffscreenRenderSize)
  | { kind: ERenderFrame.INPUT; event: IRenderInputEvent }
  | { kind: ERenderFrame.DISPOSE };

/** What the frame says back. */
export type TRenderFrameResponse = { kind: ERenderFrameResponse.CURSOR; cursor: string };

/**
 * Whether a message is the frame's rather than a viewport's own.
 *
 * @param request - Any message on the wire.
 * @returns Whether the frame answers it.
 */
export function isRenderFrameRequest(request: TRenderFrameRequest | { kind: string }): request is TRenderFrameRequest {
  return request.kind.startsWith("@frame/");
}

/**
 * What a frame message carries that has to be moved rather than copied.
 *
 * @param request - The message about to be posted.
 * @returns What to move with it.
 */
export function listRenderFrameTransfers(request: TRenderFrameRequest | { kind: string }): Array<Transferable> {
  return isRenderFrameRequest(request) && request.kind === ERenderFrame.START ? [request.canvas] : [];
}
