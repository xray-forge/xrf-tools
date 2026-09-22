import { ILevelCamera } from "@/core/level/lib/camera/level-camera";
import { ILevelFlyMotion } from "@/core/level/lib/camera/level-fly-motion";
import { ILevelSectorChange, ILevelTextureSupplyChange } from "@/core/level/lib/render/level-render-protocol";
import { ILevelRenderLevel, ILevelRenderView } from "@/core/level/lib/render/level-renderer";
import { ILevelPoint } from "@/core/level/lib/residency/level-residency";
import { ILevelStats } from "@/core/level/lib/stats/level-stats";
import { ILevelSurfaceGeometry } from "@/core/level/lib/surface/level-surface-geometry";
import { ILevelTextureReport } from "@/core/level/lib/texture/level-texture-report";
import { IOffscreenRenderSize } from "@/core/render/lib/frame/offscreen-render-target";
import { Nullable } from "@/lib/types/general";

/** What a renderer on another thread can be told. */
export enum ELevelRenderRequest {
  /** Here is the canvas, and how big it is: everything before this has nowhere to go. */
  START = "start",
  /** The element the canvas fills is a different size, which only a thread with a document can notice. */
  RESIZE = "resize",
  /** Draw this level, or no level at all. */
  OPEN = "open",
  /** These sectors have arrived, and these have gone. */
  DELIVER = "deliver",
  /** These texture files have been read, and this much is worth keeping. */
  SUPPLY = "supply",
  /** Draw it like this. */
  VIEW = "view",
  /** This is what the person is doing, gathered by the thread that has the input. */
  MOTION = "motion",
  /** What does each shader table entry draw? Answered with `MEASURED`. */
  MEASURE = "measure",
  /** Let everything go. */
  DISPOSE = "dispose",
}

/** What it says back. */
export enum ELevelRenderResponse {
  /** The camera has moved far enough to change what is near it. */
  CAMERA = "camera",
  /** What the viewport costs and where its camera is, a few times a second. */
  REPORT = "report",
  /** What the level's textures came to, whenever uploading changes it. */
  TEXTURES = "textures",
  /** The answer to one `MEASURE`, carrying the number it was asked with. */
  MEASURED = "measured",
}

/**
 * What a renderer on another thread is told, as messages.
 */
export type TLevelRenderRequest =
  | ({ kind: ELevelRenderRequest.START; canvas: OffscreenCanvas } & IOffscreenRenderSize)
  | ({ kind: ELevelRenderRequest.RESIZE } & IOffscreenRenderSize)
  | { kind: ELevelRenderRequest.OPEN; level: Nullable<ILevelRenderLevel> }
  | { kind: ELevelRenderRequest.DELIVER; change: ILevelSectorChange }
  | { kind: ELevelRenderRequest.SUPPLY; change: ILevelTextureSupplyChange }
  | { kind: ELevelRenderRequest.VIEW; view: ILevelRenderView }
  | { kind: ELevelRenderRequest.MOTION; motion: ILevelFlyMotion }
  | { kind: ELevelRenderRequest.MEASURE; id: number }
  | { kind: ELevelRenderRequest.DISPOSE };

/** What it says back, as messages. */
export type TLevelRenderResponse =
  | { kind: ELevelRenderResponse.CAMERA; point: ILevelPoint }
  | { kind: ELevelRenderResponse.REPORT; stats: ILevelStats; camera: ILevelCamera }
  | { kind: ELevelRenderResponse.TEXTURES; report: ILevelTextureReport }
  | { kind: ELevelRenderResponse.MEASURED; id: number; geometry: ReadonlyMap<number, ILevelSurfaceGeometry> };

/**
 * The buffers a request carries, which are moved rather than copied.
 *
 * @param request - The message about to be posted.
 * @returns Every buffer it owns, for the transfer list.
 */
export function listRenderTransfers(request: TLevelRenderRequest): Array<Transferable> {
  switch (request.kind) {
    case ELevelRenderRequest.START:
      return [request.canvas];

    case ELevelRenderRequest.DELIVER:
      return request.change.delivered.map((it) => it.buffer);

    case ELevelRenderRequest.SUPPLY:
      return request.change.delivered.map((it) => it.bytes);

    default:
      return [];
  }
}
