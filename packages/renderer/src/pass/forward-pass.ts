import { RenderTarget } from "three/webgpu";

import { ERendererPass } from "#/contract/scene/renderer-surface";
import { IRendererFrame } from "#/pass/renderer-frame";
import { IRendererScenePass } from "#/pass/renderer-scene-pass";
import { RendererTargets } from "#/pass/renderer-targets";

/**
 * Composites blended surfaces over the tonemapped frame, tested against the G-buffer's depth, as Base orders it.
 */
export class ForwardPass implements IRendererScenePass {
  public readonly name: string = "forward";
  public readonly scene: ERendererPass = ERendererPass.FORWARD;
  public readonly target: RenderTarget;

  public constructor(targets: RendererTargets) {
    this.target = targets.composite;
  }

  public render({ renderer, camera, scenes }: IRendererFrame): void {
    renderer.setRenderTarget(this.target);
    renderer.render(scenes[this.scene], camera);
  }

  public dispose(): void {}
}
