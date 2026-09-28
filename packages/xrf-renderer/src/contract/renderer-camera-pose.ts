import { TRendererVector } from "#/contract/renderer-vector";

/** Where the camera is, as a report states it. */
export interface IRendererCameraPose {
  position: TRendererVector;
  /** The point it looks at. */
  target: TRendererVector;
}
