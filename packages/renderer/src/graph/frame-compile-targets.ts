import { RenderTarget } from "three/webgpu";

import { IFrameShadowCompile } from "#/graph/frame-shadow-compile";
import { IRendererScenePass } from "#/pass/renderer-scene-pass";

/** Where the frame draws what the compiler builds pipelines for, as the frame holds it now. */
export interface IFrameCompileTargets {
  /** The frame's passes drawing a consumer scene, each compiled against its own target. */
  readonly passes: ReadonlyArray<IRendererScenePass>;
  readonly shadow: IFrameShadowCompile;
  /** Where the grass draws, which its staged builds compile against. */
  readonly grass: RenderTarget;
}
