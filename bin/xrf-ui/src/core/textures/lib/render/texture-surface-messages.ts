import { IOffscreenRenderSize } from "@/core/render/lib/frame/offscreen-render-target";
import { IRenderFrameCost } from "@/core/render/lib/frame/render-frame-cost";
import { TFrameRateLimit } from "@/core/render/lib/frame/render-frame-limit";
import { IRenderLighting } from "@/core/render/lib/lighting/render-lighting";
import { IRenderInputEvent } from "@/core/render/lib/worker/render-input";
import { ITextureSurfaceFiles, ITextureSurfaceOptions } from "@/core/textures/lib/texture-surface";

/** What a lit surface on another thread can be told. */
export enum ETextureSurfaceRequest {
  /** Here is the canvas, and how big it is: everything before this has nowhere to go. */
  START = "start",
  /** The element the canvas fills is a different size, or draws a different number of pixels. */
  RESIZE = "resize",
  /** Draw these files. */
  TEXTURES = "textures",
  /** Draw them like this. */
  OPTIONS = "options",
  /** Light them like this. */
  LIGHTING = "lighting",
  /** Redraw no more often than this. */
  FRAME_RATE = "frameRate",
  /** Somebody did this to the canvas. */
  INPUT = "input",
  /** Swing the light by a drag, and say where it went. */
  DRAG_LIGHT = "dragLight",
  /** Move the camera towards the body or away from it. */
  DOLLY = "dolly",
  /** Back to the distance and angle the body is first seen from. */
  RESET = "reset",
  /** Let everything go. */
  DISPOSE = "dispose",
}

/** What it says back. */
export enum ETextureSurfaceResponse {
  /** Where a drag has put the light. */
  LIGHTING = "lighting",
  /** What frames are costing. */
  REPORT = "report",
  /** What the controls want the cursor to be, which only the side with a canvas can show. */
  CURSOR = "cursor",
}

/**
 * What a lit surface on another thread is told, as messages.
 */
export type TTextureSurfaceRequest =
  | ({ kind: ETextureSurfaceRequest.START; canvas: OffscreenCanvas } & IOffscreenRenderSize)
  | ({ kind: ETextureSurfaceRequest.RESIZE } & IOffscreenRenderSize)
  | { kind: ETextureSurfaceRequest.TEXTURES; files: ITextureSurfaceFiles }
  | { kind: ETextureSurfaceRequest.OPTIONS; options: ITextureSurfaceOptions }
  | { kind: ETextureSurfaceRequest.LIGHTING; lighting: IRenderLighting }
  | { kind: ETextureSurfaceRequest.FRAME_RATE; limit: TFrameRateLimit }
  | { kind: ETextureSurfaceRequest.INPUT; event: IRenderInputEvent }
  | { kind: ETextureSurfaceRequest.DRAG_LIGHT; deltaX: number; deltaY: number }
  | { kind: ETextureSurfaceRequest.DOLLY; step: number }
  | { kind: ETextureSurfaceRequest.RESET }
  | { kind: ETextureSurfaceRequest.DISPOSE };

/** What it says back, as messages. */
export type TTextureSurfaceResponse =
  | { kind: ETextureSurfaceResponse.LIGHTING; lighting: IRenderLighting }
  | { kind: ETextureSurfaceResponse.REPORT; cost: IRenderFrameCost }
  | { kind: ETextureSurfaceResponse.CURSOR; cursor: string };

/**
 * What a request carries that has to be moved rather than copied.
 *
 * @param request - The message about to be posted.
 * @returns What to move with it.
 */
export function listTextureSurfaceTransfers(request: TTextureSurfaceRequest): Array<Transferable> {
  return request.kind === ETextureSurfaceRequest.START ? [request.canvas] : [];
}
