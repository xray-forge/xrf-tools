import { RenderTarget } from "three/webgpu";

import { ERendererPass } from "#/contract/scene/renderer-pass";
import { IRendererFrame } from "#/pass/renderer-frame";
import { IRendererScenePass } from "#/pass/renderer-scene-pass";
import { RendererTargets } from "#/pass/renderer-targets";
import { drawUnsorted } from "#/pass/unsorted-draw";

/**
 * Draws on into the G-buffer every plain draw the deferred passes light, once the static draws' phases and the pyramid
 * are done: a plain draw moves with no version the culls see, so it never hides a static draw.
 */
export class GBufferPass implements IRendererScenePass {
  public readonly name: string = "gbuffer";
  public readonly isScenePass = true as const;
  public readonly scene: ERendererPass = ERendererPass.DEFERRED;
  public readonly target: RenderTarget;

  /**
   * @param targets - What the frame draws into.
   */
  public constructor(targets: RendererTargets) {
    this.target = targets.gbuffer;
  }

  public render({ renderer, camera, scenes }: IRendererFrame): void {
    renderer.setRenderTarget(this.target);
    // In scene order, where an object's parts stand together and share their buffers from one draw to the next:
    // sorted by depth, every draw rebinds them, and the CPU is what a frame waits on.
    drawUnsorted(renderer, () => renderer.render(scenes[this.scene], camera));
  }

  public dispose(): void {}
}
