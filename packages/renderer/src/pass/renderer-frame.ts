import { Nullable } from "@xrf/types";
import { PerspectiveCamera, Scene, WebGPURenderer } from "three/webgpu";

import { IRendererSettings } from "#/contract/renderer-settings";
import { IRendererFrameJitter } from "#/sampling/renderer-frame-jitter";
import { TPassRecord } from "#/scene/pass-record";

/**
 * What every pass of one frame reads; what it draws into it was made with.
 */
export interface IRendererFrame {
  renderer: WebGPURenderer;
  /** The camera the scene draws with: the view's, offset by this frame's jitter while a temporal mode resolves. */
  camera: PerspectiveCamera;
  /** The view's own camera, never jittered: what the helpers draw with. */
  viewCamera: PerspectiveCamera;
  /** This frame's jitter, while a temporal mode resolves. */
  jitter: Nullable<IRendererFrameJitter>;
  /** Seconds the renderer has been running, which everything that changes over time follows. */
  time: number;
  settings: IRendererSettings;
  /** What each pass draws of the consumer's scene. */
  scenes: TPassRecord<Scene>;
}
