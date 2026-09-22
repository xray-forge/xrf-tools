import { TRendererCamera, TRendererCameraCommand } from "#/contract/renderer-camera";
import { IRendererDevice } from "#/contract/renderer-device";
import { IRendererLighting } from "#/contract/renderer-lighting";
import { IRendererReport } from "#/contract/renderer-report";
import { ERendererDebugView, IRendererSettings } from "#/contract/renderer-settings";
import { IRendererGeometry, listRendererGeometryTransfers } from "#/contract/scene/renderer-geometry";
import { IRendererObject } from "#/contract/scene/renderer-object";
import { IRendererSurface } from "#/contract/scene/renderer-surface";
import { TRendererTextureSource } from "#/contract/scene/renderer-texture-source";
import { IOffscreenRenderSize } from "#/frame/offscreen-render-target";
import { IRenderInputEvent } from "#/input/render-input";
import { IDdsRefusal } from "#/texture/dds/dds-refusal";

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
  /** Hold this texture under this key, replacing whatever held it. */
  PUT_TEXTURE = "@renderer/putTexture",
  RELEASE_TEXTURE = "@renderer/releaseTexture",
  /** Hold this geometry under this key. */
  PUT_GEOMETRY = "@renderer/putGeometry",
  RELEASE_GEOMETRY = "@renderer/releaseGeometry",
  /** Hold this surface under this key. */
  PUT_SURFACE = "@renderer/putSurface",
  RELEASE_SURFACE = "@renderer/releaseSurface",
  /** Draw this object under this key. */
  PUT_OBJECT = "@renderer/putObject",
  RELEASE_OBJECT = "@renderer/releaseObject",
  /** Light the scene like this. */
  LIGHTING = "@renderer/lighting",
  /** Drive the camera like this. */
  CAMERA = "@renderer/camera",
  /** Do this with the camera. */
  CAMERA_COMMAND = "@renderer/cameraCommand",
  /** Somebody did this to the canvas, which only the thread holding it can see. */
  INPUT = "@renderer/input",
  /** Draw a frame and hand back a picture of it. */
  CAPTURE = "@renderer/capture",
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
  /** A texture's file cannot be uploaded as stored; the consumer can put its decoded picture instead. */
  TEXTURE_REFUSED = "@renderer/textureRefused",
  /** What the camera controls want the cursor to be, which only the side with a canvas can show. */
  CURSOR = "@renderer/cursor",
  /** The picture a capture asked for. */
  CAPTURED = "@renderer/captured",
}

/** Every message a consumer sends. */
export type TRendererRequest =
  | ({ kind: ERendererRequest.START; canvas: OffscreenCanvas; settings: IRendererSettings } & IOffscreenRenderSize)
  | ({ kind: ERendererRequest.RESIZE } & IOffscreenRenderSize)
  | { kind: ERendererRequest.CONFIGURE; settings: IRendererSettings }
  | { kind: ERendererRequest.DISPOSE }
  | { kind: ERendererRequest.PUT_TEXTURE; key: string; source: TRendererTextureSource }
  | { kind: ERendererRequest.RELEASE_TEXTURE; key: string }
  | { kind: ERendererRequest.PUT_GEOMETRY; key: string; geometry: IRendererGeometry }
  | { kind: ERendererRequest.RELEASE_GEOMETRY; key: string }
  | { kind: ERendererRequest.PUT_SURFACE; key: string; surface: IRendererSurface }
  | { kind: ERendererRequest.RELEASE_SURFACE; key: string }
  | { kind: ERendererRequest.PUT_OBJECT; key: string; object: IRendererObject }
  | { kind: ERendererRequest.RELEASE_OBJECT; key: string }
  | { kind: ERendererRequest.LIGHTING; lighting: IRendererLighting }
  | { kind: ERendererRequest.CAMERA; camera: TRendererCamera }
  | { kind: ERendererRequest.CAMERA_COMMAND; command: TRendererCameraCommand }
  | { kind: ERendererRequest.INPUT; event: IRenderInputEvent }
  | { kind: ERendererRequest.CAPTURE; id: number; view: ERendererDebugView };

/** Every message the renderer sends. */
export type TRendererResponse =
  | { kind: ERendererResponse.READY; device: IRendererDevice }
  | { kind: ERendererResponse.FAILED; reason: string }
  | { kind: ERendererResponse.REPORT; report: IRendererReport }
  | { kind: ERendererResponse.TEXTURE_REFUSED; key: string; refusal: IDdsRefusal }
  | { kind: ERendererResponse.CURSOR; cursor: string }
  | { kind: ERendererResponse.CAPTURED; id: number; image: ImageBitmap };

/**
 * What a request carries that has to be moved rather than copied.
 *
 * @param request - The message about to be posted.
 * @returns What to move with it.
 */
export function listRendererTransfers(request: TRendererRequest): Array<Transferable> {
  switch (request.kind) {
    case ERendererRequest.START:
      return [request.canvas];

    case ERendererRequest.PUT_TEXTURE:
      return [request.source.bytes];

    case ERendererRequest.PUT_GEOMETRY:
      return listRendererGeometryTransfers(request.geometry);

    default:
      return [];
  }
}
