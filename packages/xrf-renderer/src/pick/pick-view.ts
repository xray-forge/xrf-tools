import { PerspectiveCamera } from "three/webgpu";

import { IRendererViewSize } from "#/contract/renderer-view-size";

/**
 * The view a pick is drawn at: the camera the frame drew with, unjittered, and how big the view is.
 */
export interface IPickView {
  camera: PerspectiveCamera;
  size: IRendererViewSize;
}
