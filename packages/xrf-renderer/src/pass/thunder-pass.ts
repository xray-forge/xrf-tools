import { Nullable } from "@xrf/types";
import { RenderTarget, Scene } from "three/webgpu";

import { IRendererFrame } from "#/pass/renderer-frame";
import { IRendererPass } from "#/pass/renderer-pass";
import { RendererTargets } from "#/pass/renderer-targets";
import { SceneThunder } from "#/scene/thunder/scene-thunder";

/**
 * The bolt striking, as `RenderLast` draws it after the rain: its model and its two glows over the tonemapped frame,
 * tested against its depth. Draws nothing while no bolt strikes or its draws are still compiling.
 */
export class ThunderPass implements IRendererPass {
  public readonly name: string = "thunder";

  private readonly target: RenderTarget;
  private readonly scene: SceneThunder;

  /**
   * @param targets - The frame's targets, whose composite the bolt draws over.
   * @param scene - The thunder's draws.
   */
  public constructor(targets: RendererTargets, scene: SceneThunder) {
    this.target = targets.composite;
    this.scene = scene;
  }

  public render({ renderer, camera }: IRendererFrame): void {
    const drawn: Nullable<Scene> = this.scene.drawn;

    if (!drawn) {
      return;
    }

    renderer.setRenderTarget(this.target);
    renderer.render(drawn, camera);
  }

  /** The draws are the scene's, which lets them go with the rest of what the consumer put. */
  public dispose(): void {}
}
