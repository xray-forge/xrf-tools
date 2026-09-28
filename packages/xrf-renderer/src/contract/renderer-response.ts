import { Nullable } from "@xrf/types";

import { IRendererDevice } from "#/contract/renderer-device";
import { IRendererReport } from "#/contract/renderer-report";
import { IRendererTextureFetch } from "#/contract/scene/renderer-texture-fetch";
import { IDdsRefusal } from "#/dds/dds-refusal";

/**
 * What the renderer tells its consumer.
 */
export enum ERendererResponse {
  /** The device is up. */
  READY = "@renderer/ready",
  /** The renderer cannot draw here, and why; there is no fallback. */
  FAILED = "@renderer/failed",
  /** What the frames have been costing. */
  REPORT = "@renderer/report",
  /** A texture's file cannot be uploaded as stored; the consumer can put its decoded picture instead. */
  TEXTURE_REFUSED = "@renderer/textureRefused",
  /** A texture the renderer fetched arrived, or could not, and what it came to. */
  TEXTURE_FETCHED = "@renderer/textureFetched",
  /** What the camera controls want the cursor to be, which only the side with a canvas can show. */
  CURSOR = "@renderer/cursor",
  /** The picture a capture asked for, or nothing where there was none to draw. */
  CAPTURED = "@renderer/captured",
  /** A frame was drawn with everything asked for before a settle on the GPU and compiled. */
  SETTLED = "@renderer/settled",
}

/** Every message the renderer sends. */
export type TRendererResponse =
  | { kind: ERendererResponse.READY; device: IRendererDevice }
  | { kind: ERendererResponse.FAILED; reason: string }
  | { kind: ERendererResponse.REPORT; report: IRendererReport }
  | { kind: ERendererResponse.TEXTURE_REFUSED; key: string; refusal: IDdsRefusal }
  | { kind: ERendererResponse.TEXTURE_FETCHED; key: string; fetch: IRendererTextureFetch }
  | { kind: ERendererResponse.CURSOR; cursor: string }
  | { kind: ERendererResponse.CAPTURED; id: number; image: Nullable<ImageBitmap> }
  | { kind: ERendererResponse.SETTLED; id: number };
