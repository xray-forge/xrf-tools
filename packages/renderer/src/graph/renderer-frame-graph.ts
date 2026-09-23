import { WebGPURenderer } from "three/webgpu";

import { createBaseFramePasses } from "#/graph/base-frame-passes";
import { PresentPass } from "#/pass/present-pass";
import { IRendererFrame } from "#/pass/renderer-frame";
import { IRendererPass } from "#/pass/renderer-pass";
import { IRendererScenePass, isRendererScenePass } from "#/pass/renderer-scene-pass";
import { RendererTargets } from "#/pass/renderer-targets";
import { RendererOverlays } from "#/scene/overlay/renderer-overlays";
import { StaticCull } from "#/scene/static/static-cull";
import { RendererPassInspector } from "#/timing/renderer-pass-inspector";
import { RendererUniforms } from "#/uniforms/renderer-uniforms";

/**
 * The frame: every target it draws into, and its passes in order, the picture presented last.
 */
export class RendererFrameGraph {
  public readonly targets: RendererTargets = new RendererTargets();
  /** The last pass, which a capture also draws into a target of its own. */
  public readonly present: PresentPass;
  /** The passes drawing the consumer's scenes, whose materials compile against their targets. */
  public readonly scenePasses: ReadonlyArray<IRendererScenePass>;
  /** Every pass's name, in frame order, as the frame report states them. */
  public readonly passNames: ReadonlyArray<string>;

  private readonly passes: ReadonlyArray<IRendererPass>;

  public constructor(uniforms: RendererUniforms, overlays: RendererOverlays, cull: StaticCull) {
    this.present = new PresentPass(this.targets, uniforms.camera);
    this.passes = [...createBaseFramePasses(this.targets, uniforms, overlays, cull), this.present];
    this.scenePasses = this.passes.filter(isRendererScenePass);
    this.passNames = this.passes.map((pass: IRendererPass) => pass.name);
  }

  /**
   * @param renderer - The renderer the targets are drawn by.
   * @param width - Drawing buffer width, in device pixels.
   * @param height - Drawing buffer height, in device pixels.
   */
  public resize(renderer: WebGPURenderer, width: number, height: number): void {
    this.targets.resize(width, height);
    this.targets.prepare(renderer);
  }

  /**
   * @param frame - What the frame draws with.
   * @param inspector - What times each pass under its name.
   */
  public render(frame: IRendererFrame, inspector: RendererPassInspector): void {
    for (const pass of this.passes) {
      inspector.enter(pass.name);
      pass.render(frame);
      inspector.leave();
    }
  }

  public dispose(): void {
    this.passes.forEach((pass: IRendererPass) => pass.dispose());
    this.targets.dispose();
  }
}
