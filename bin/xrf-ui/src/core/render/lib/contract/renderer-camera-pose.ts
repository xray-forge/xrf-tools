import { TRendererVector } from "@/core/render/lib/contract/renderer-vector";

/** Where the camera is, as a report states it. */
export interface IRendererCameraPose {
  position: TRendererVector;
  /** The point it looks at. */
  target: TRendererVector;
}
