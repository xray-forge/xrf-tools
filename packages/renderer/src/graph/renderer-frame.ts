import { PerspectiveCamera, WebGPURenderer } from "three/webgpu";

import { IRendererSettings } from "#/contract/renderer-settings";
import { RendererTargets } from "#/graph/renderer-targets";
import { TPassScenes } from "#/scene/renderer-scene";

/**
 * What every pass of one frame reads.
 */
export interface IRendererFrame {
  renderer: WebGPURenderer;
  camera: PerspectiveCamera;
  targets: RendererTargets;
  settings: IRendererSettings;
  /** What each pass draws: the G-buffer's surfaces, the wall marks composited into it, and what is composited after. */
  scenes: TPassScenes;
}
