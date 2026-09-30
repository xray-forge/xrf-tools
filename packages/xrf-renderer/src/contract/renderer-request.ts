import { Nullable } from "@xrf/types";

import { IRenderInputEvent } from "#/contract/render-input-event";
import { TRendererCamera } from "#/contract/renderer-camera";
import { TRendererCameraCommand } from "#/contract/renderer-camera-command";
import { TRendererCaptureSource } from "#/contract/renderer-capture-source";
import { IRendererLighting } from "#/contract/renderer-lighting";
import { IRendererSettings } from "#/contract/renderer-settings";
import { IRendererViewPoint } from "#/contract/renderer-view-point";
import { IRendererViewSize } from "#/contract/renderer-view-size";
import { IRendererGeometry, listRendererGeometryTransfers } from "#/contract/scene/renderer-geometry";
import { IRendererGrass, listRendererGrassTransfers } from "#/contract/scene/renderer-grass";
import { IRendererImpostors, listRendererImpostorsTransfers } from "#/contract/scene/renderer-impostors";
import { IRendererLights } from "#/contract/scene/renderer-lights";
import { IRendererMotion, listRendererMotionTransfers } from "#/contract/scene/renderer-motion";
import { IRendererObject, listRendererObjectTransfers } from "#/contract/scene/renderer-object";
import { listRendererOverlayTransfers, TRendererOverlay } from "#/contract/scene/renderer-overlay";
import { IRendererPose } from "#/contract/scene/renderer-pose";
import { IRendererSkeleton, listRendererSkeletonTransfers } from "#/contract/scene/renderer-skeleton";
import { IRendererSurface } from "#/contract/scene/renderer-surface";
import { ERendererTextureEncoding, TRendererTextureSource } from "#/contract/scene/renderer-texture-source";
import { TRendererWeatherChange } from "#/contract/weather/renderer-weather-change";
import { IRendererWeatherControl } from "#/contract/weather/renderer-weather-control";
import { ERendererWeatherTransition } from "#/contract/weather/renderer-weather-transition";

/**
 * What a consumer tells the renderer.
 */
export enum ERendererRequest {
  /** Bring the device up, with no canvas yet: textures, geometry and captures need none. */
  START = "@renderer/start",
  /** Here is a canvas to show frames on, and how big it is: frames are drawn only while one is attached. */
  ATTACH_VIEW = "@renderer/attachView",
  /** The canvas is going; stop drawing frames. */
  DETACH_VIEW = "@renderer/detachView",
  /** The element the canvas fills is a different size, or draws a different number of pixels. */
  RESIZE = "@renderer/resize",
  /** The consumer's settings for drawing changed. */
  CONFIGURE = "@renderer/configure",
  /** Let everything go. */
  DISPOSE = "@renderer/dispose",
  /** Hold this texture under this key, replacing whatever held it, a fetch of it in flight included. */
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
  /** Hold these impostors under this key, for objects to stand trees and impostors by. */
  PUT_IMPOSTORS = "@renderer/putImpostors",
  PUT_GRASS = "@renderer/putGrass",
  RELEASE_GRASS = "@renderer/releaseGrass",
  /** Light the scene with these local lights, replacing any put before. */
  PUT_LIGHTS = "@renderer/putLights",
  RELEASE_LIGHTS = "@renderer/releaseLights",
  RELEASE_IMPOSTORS = "@renderer/releaseImpostors",
  /** Hold this skeleton under this key, for objects to skin to. */
  PUT_SKELETON = "@renderer/putSkeleton",
  RELEASE_SKELETON = "@renderer/releaseSkeleton",
  /** Hold this baked motion under this key, for poses to name. */
  PUT_MOTION = "@renderer/putMotion",
  RELEASE_MOTION = "@renderer/releaseMotion",
  /** Stand this skeleton like this. */
  POSE = "@renderer/pose",
  /** Draw this helper under this key. */
  PUT_OVERLAY = "@renderer/putOverlay",
  RELEASE_OVERLAY = "@renderer/releaseOverlay",
  /** Light the scene like this, while no weather plays. */
  LIGHTING = "@renderer/lighting",
  /** Play this weather, lighting the scene by it in place of the lighting; null to light by the lighting again. */
  WEATHER = "@renderer/weather",
  /** Play the weather like this. */
  WEATHER_CONTROL = "@renderer/weatherControl",
  /** Play this weather effect over the cycle from the clock's time, or end the one playing. */
  WEATHER_EFFECT = "@renderer/weatherEffect",
  /** Drive the camera like this. */
  CAMERA = "@renderer/camera",
  /** Do this with the camera. */
  CAMERA_COMMAND = "@renderer/cameraCommand",
  /** Somebody did this to the canvas, which only the thread holding it can see. */
  INPUT = "@renderer/input",
  /** Draw a picture of the frame or of a texture, and hand it back. */
  CAPTURE = "@renderer/capture",
  /** Say what is drawn under this point of the view. */
  PICK = "@renderer/pick",
  /** Say when a frame has been drawn with everything asked for so far on the GPU and compiled. */
  SETTLE = "@renderer/settle",
  /** These requests, made in one page task, applied in one worker task so no frame shows half of them. */
  BATCH = "@renderer/batch",
}

/** Every message a consumer sends. */
export type TRendererRequest =
  | { kind: ERendererRequest.START; settings: IRendererSettings }
  | ({ kind: ERendererRequest.ATTACH_VIEW; canvas: OffscreenCanvas } & IRendererViewSize)
  | { kind: ERendererRequest.DETACH_VIEW }
  | ({ kind: ERendererRequest.RESIZE } & IRendererViewSize)
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
  | { kind: ERendererRequest.PUT_IMPOSTORS; key: string; impostors: IRendererImpostors }
  | { kind: ERendererRequest.RELEASE_IMPOSTORS; key: string }
  | { kind: ERendererRequest.PUT_GRASS; grass: IRendererGrass }
  | { kind: ERendererRequest.RELEASE_GRASS }
  | { kind: ERendererRequest.PUT_LIGHTS; lights: IRendererLights }
  | { kind: ERendererRequest.RELEASE_LIGHTS }
  | { kind: ERendererRequest.PUT_SKELETON; key: string; skeleton: IRendererSkeleton }
  | { kind: ERendererRequest.RELEASE_SKELETON; key: string }
  | { kind: ERendererRequest.PUT_MOTION; key: string; motion: IRendererMotion }
  | { kind: ERendererRequest.RELEASE_MOTION; key: string }
  | { kind: ERendererRequest.POSE; skeleton: string; pose: IRendererPose }
  | { kind: ERendererRequest.PUT_OVERLAY; key: string; overlay: TRendererOverlay }
  | { kind: ERendererRequest.RELEASE_OVERLAY; key: string }
  | { kind: ERendererRequest.LIGHTING; lighting: IRendererLighting }
  | {
      kind: ERendererRequest.WEATHER;
      weather: Nullable<TRendererWeatherChange>;
      transition: ERendererWeatherTransition;
    }
  | { kind: ERendererRequest.WEATHER_CONTROL; control: IRendererWeatherControl }
  | { kind: ERendererRequest.WEATHER_EFFECT; effect: Nullable<string> }
  | { kind: ERendererRequest.CAMERA; camera: TRendererCamera }
  | { kind: ERendererRequest.CAMERA_COMMAND; command: TRendererCameraCommand }
  | { kind: ERendererRequest.INPUT; event: IRenderInputEvent }
  | { kind: ERendererRequest.CAPTURE; id: number; source: TRendererCaptureSource }
  | { kind: ERendererRequest.PICK; id: number; point: IRendererViewPoint }
  | { kind: ERendererRequest.SETTLE; id: number }
  | { kind: ERendererRequest.BATCH; requests: ReadonlyArray<TRendererRequest> };

/**
 * What a request carries that has to be moved rather than copied.
 *
 * @param request - The message about to be posted.
 * @returns What to move with it.
 */
export function listRendererTransfers(request: TRendererRequest): Array<Transferable> {
  // Once each: several arrays can be views over one buffer, and a buffer listed twice fails the post.
  return [...new Set(listRequestTransfers(request))];
}

function listRequestTransfers(request: TRendererRequest): Array<Transferable> {
  switch (request.kind) {
    case ERendererRequest.ATTACH_VIEW:
      return [request.canvas];

    case ERendererRequest.PUT_TEXTURE:
      // A fetched texture's bytes are the renderer's own, and never were the page's to move.
      return request.source.encoding === ERendererTextureEncoding.FETCH ? [] : [request.source.bytes];

    case ERendererRequest.PUT_GEOMETRY:
      return listRendererGeometryTransfers(request.geometry);

    case ERendererRequest.PUT_OBJECT:
      return listRendererObjectTransfers(request.object);

    case ERendererRequest.PUT_IMPOSTORS:
      return listRendererImpostorsTransfers(request.impostors);

    case ERendererRequest.PUT_GRASS:
      return listRendererGrassTransfers(request.grass);

    case ERendererRequest.PUT_SKELETON:
      return listRendererSkeletonTransfers(request.skeleton);

    case ERendererRequest.PUT_MOTION:
      return listRendererMotionTransfers(request.motion);

    case ERendererRequest.PUT_OVERLAY:
      return listRendererOverlayTransfers(request.overlay);

    case ERendererRequest.BATCH:
      return request.requests.flatMap(listRequestTransfers);

    default:
      return [];
  }
}
