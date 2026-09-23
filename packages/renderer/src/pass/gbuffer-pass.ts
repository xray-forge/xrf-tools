import { RenderTarget } from "three/webgpu";

import { ERendererPass } from "#/contract/scene/renderer-surface";
import { IRendererFrame } from "#/pass/renderer-frame";
import { IRendererScenePass } from "#/pass/renderer-scene-pass";
import { RendererTargets } from "#/pass/renderer-targets";
import { StaticCull } from "#/scene/static/static-cull";

/**
 * Fills the G-buffer with everything the deferred passes light: the static draws the first cull kept and every plain
 * draw, then the static draws the second cull finds seen past what those drew.
 */
export class GBufferPass implements IRendererScenePass {
  public readonly name: string = "gbuffer";
  public readonly scene: ERendererPass = ERendererPass.DEFERRED;
  public readonly target: RenderTarget;

  private readonly targets: RendererTargets;
  private readonly cull: StaticCull;

  /**
   * @param targets - What the frame draws into.
   * @param cull - What culls the static draws, whose second phase draws between.
   */
  public constructor(targets: RendererTargets, cull: StaticCull) {
    this.target = targets.gbuffer;
    this.targets = targets;
    this.cull = cull;
  }

  public render({ renderer, camera, scenes }: IRendererFrame): void {
    renderer.setClearColor(0x000000, 0);
    renderer.setRenderTarget(this.target);
    renderer.clear(true, true, false);
    // In scene order, where an object's parts stand together and share their buffers from one draw to the next:
    // sorted by depth, every draw rebinds them, and the CPU is what a frame waits on.
    renderer.sortObjects = false;
    renderer.render(scenes[this.scene], camera);
    this.cull.drawLate(renderer, camera, this.targets.depth, this.target.width, this.target.height);
    this.cull.finish(renderer, this.targets.depth, this.target.width, this.target.height);
    renderer.sortObjects = true;
  }

  public dispose(): void {}
}
