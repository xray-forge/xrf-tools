import { PerspectiveCamera, Scene, WebGPURenderer } from "three/webgpu";

import { IRendererSettings } from "#/contract/renderer-settings";
import { RendererTargets } from "#/graph/renderer-targets";

/**
 * What every pass of one frame reads.
 */
export interface IRendererFrame {
  renderer: WebGPURenderer;
  camera: PerspectiveCamera;
  targets: RendererTargets;
  settings: IRendererSettings;
  /** What the deferred passes draw into the G-buffer. */
  deferred: Scene;
  /** What is composited after them. */
  forward: Scene;
}
