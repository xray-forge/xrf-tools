import { RenderTarget } from "three/webgpu";

import { ERendererPass } from "#/contract/scene/renderer-surface";
import { IRendererFrame } from "#/pass/renderer-frame";
import { IRendererScenePass } from "#/pass/renderer-scene-pass";
import { RendererTargets } from "#/pass/renderer-targets";

/**
 * Fills the G-buffer with everything the deferred passes light that the first cull kept: its static draws and every
 * plain draw. The second phase draws on into it after.
 */
export class GBufferPass implements IRendererScenePass {
  public readonly name: string = "gbuffer";
  public readonly scene: ERendererPass = ERendererPass.DEFERRED;
  public readonly target: RenderTarget;

  /**
   * @param targets - What the frame draws into.
   */
  public constructor(targets: RendererTargets) {
    this.target = targets.gbuffer;
  }

  public render({ renderer, camera, scenes }: IRendererFrame): void {
    renderer.setClearColor(0x000000, 0);
    renderer.setRenderTarget(this.target);
    renderer.clear(true, true, false);
    // In scene order, where an object's parts stand together and share their buffers from one draw to the next:
    // sorted by depth, every draw rebinds them, and the CPU is what a frame waits on.
    renderer.sortObjects = false;
    renderer.render(scenes[this.scene], camera);
    renderer.sortObjects = true;
  }

  public dispose(): void {}
}
