import { Nullable } from "@xrf/types";
import { WebGPURenderer } from "three/webgpu";

import { ERendererAntialiasing, IRendererFeatureSettings } from "#/contract/renderer-features";
import { createBaseFramePasses } from "#/graph/base-frame-passes";
import { AntialiasPass, toPresentedFrame } from "#/pass/antialias/antialias-pass";
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
  public passNames: ReadonlyArray<string> = [];

  private readonly base: ReadonlyArray<IRendererPass>;
  private passes: ReadonlyArray<IRendererPass> = [];
  /** The pass smoothing the frame's edges, while a mode is chosen. */
  private antialias: Nullable<AntialiasPass> = null;
  private antialiasing: ERendererAntialiasing = ERendererAntialiasing.NONE;

  public constructor(uniforms: RendererUniforms, overlays: RendererOverlays, cull: StaticCull) {
    this.present = new PresentPass(this.targets, uniforms.camera);
    this.base = createBaseFramePasses(this.targets, uniforms, overlays, cull);
    this.scenePasses = this.base.filter(isRendererScenePass);
    this.link();
  }

  /**
   * Puts into the frame the passes the features want and takes out the ones they do not, whose targets go with them.
   *
   * @param features - What the features are set to.
   */
  public configure(features: IRendererFeatureSettings): void {
    if (features.antialiasing === this.antialiasing) {
      return;
    }

    this.antialias?.dispose();
    this.antialiasing = features.antialiasing;
    this.antialias =
      features.antialiasing === ERendererAntialiasing.NONE
        ? null
        : new AntialiasPass(features.antialiasing, this.targets);
    this.present.setFrame(toPresentedFrame(this.antialias, this.targets));
    this.link();
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

  /** The frame's passes in order: the base's, whatever the features add, then the picture presented. */
  private link(): void {
    this.passes = [...this.base, ...(this.antialias ? [this.antialias] : []), this.present];
    this.passNames = this.passes.map((pass: IRendererPass) => pass.name);
  }
}
