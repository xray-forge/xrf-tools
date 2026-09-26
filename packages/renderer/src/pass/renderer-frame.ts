import { Nullable } from "@xrf/types";
import { PerspectiveCamera, Scene, WebGPURenderer } from "three/webgpu";

import { IRendererSettings } from "#/contract/renderer-settings";
import { RendererTargets } from "#/pass/renderer-targets";
import { IRendererFrameJitter } from "#/sampling/renderer-frame-jitter";
import { TPassRecord } from "#/scene/pass-record";

/**
 * What every pass of one frame reads.
 */
export interface IRendererFrame {
  renderer: WebGPURenderer;
  /** The camera the scene draws with: the view's, offset by this frame's jitter while a temporal mode resolves. */
  camera: PerspectiveCamera;
  /** The view's own camera, never jittered: what the helpers draw with. */
  viewCamera: PerspectiveCamera;
  /** This frame's jitter, while a temporal mode resolves. */
  jitter: Nullable<IRendererFrameJitter>;
  targets: RendererTargets;
  settings: IRendererSettings;
  /** What each pass draws of the consumer's scene. */
  scenes: TPassRecord<Scene>;
}
