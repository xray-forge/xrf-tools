import { IRenderFrameCost } from "@/core/render/lib/frame/render-frame-cost";
import { TFrameRateLimit } from "@/core/render/lib/frame/render-frame-limit";
import { IRenderLighting } from "@/core/render/lib/lighting/render-lighting";
import { ITextureSurfaceFiles, ITextureSurfaceOptions } from "@/core/textures/lib/texture-surface";

/**
 * What a lit surface on another thread can be told about what it draws.
 */
export enum ETextureSurfaceRequest {
  /** Draw these files. */
  TEXTURES = "textures",
  /** Draw them like this. */
  OPTIONS = "options",
  /** Light them like this. */
  LIGHTING = "lighting",
  /** Redraw no more often than this. */
  FRAME_RATE = "frameRate",
  /** Swing the light by a drag, and say where it went. */
  DRAG_LIGHT = "dragLight",
  /** Move the camera towards the body or away from it. */
  DOLLY = "dolly",
  /** Back to the distance and angle the body is first seen from. */
  RESET = "reset",
}

/** What it says back about what it draws. */
export enum ETextureSurfaceResponse {
  /** Where a drag has put the light. */
  LIGHTING = "lighting",
  /** What frames are costing. */
  REPORT = "report",
}

/**
 * What a lit surface on another thread is told, as messages.
 */
export type TTextureSurfaceRequest =
  | { kind: ETextureSurfaceRequest.TEXTURES; files: ITextureSurfaceFiles }
  | { kind: ETextureSurfaceRequest.OPTIONS; options: ITextureSurfaceOptions }
  | { kind: ETextureSurfaceRequest.LIGHTING; lighting: IRenderLighting }
  | { kind: ETextureSurfaceRequest.FRAME_RATE; limit: TFrameRateLimit }
  | { kind: ETextureSurfaceRequest.DRAG_LIGHT; deltaX: number; deltaY: number }
  | { kind: ETextureSurfaceRequest.DOLLY; step: number }
  | { kind: ETextureSurfaceRequest.RESET };

/** What it says back, as messages. */
export type TTextureSurfaceResponse =
  | { kind: ETextureSurfaceResponse.LIGHTING; lighting: IRenderLighting }
  | { kind: ETextureSurfaceResponse.REPORT; cost: IRenderFrameCost };
