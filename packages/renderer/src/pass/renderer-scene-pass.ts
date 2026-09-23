import { RenderTarget } from "three/webgpu";

import { ERendererPass } from "#/contract/scene/renderer-surface";
import { IRendererPass } from "#/pass/renderer-pass";

/**
 * A pass drawing one of the consumer's scenes, whose materials are compiled against the target it draws into.
 */
export interface IRendererScenePass extends IRendererPass {
  /** The scene it draws. */
  readonly scene: ERendererPass;
  /** Where it draws, whose attachments its pipelines are built for. */
  readonly target: RenderTarget;
}

/**
 * @param pass - A pass of the frame.
 * @returns Whether it draws one of the consumer's scenes.
 */
export function isRendererScenePass(pass: IRendererPass): pass is IRendererScenePass {
  return "scene" in pass && "target" in pass;
}
