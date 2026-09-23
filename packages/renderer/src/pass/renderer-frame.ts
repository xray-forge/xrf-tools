import { PerspectiveCamera, Scene, WebGPURenderer } from "three/webgpu";

import { IRendererSettings } from "#/contract/renderer-settings";
import { RendererTargets } from "#/pass/renderer-targets";
import { TPassRecord } from "#/scene/pass-record";

/**
 * What every pass of one frame reads.
 */
export interface IRendererFrame {
  renderer: WebGPURenderer;
  camera: PerspectiveCamera;
  targets: RendererTargets;
  settings: IRendererSettings;
  /** What each pass draws of the consumer's scene. */
  scenes: TPassRecord<Scene>;
}
