import { Nullable } from "@xrf/types";
import { RenderTarget, Texture, WebGPURenderer } from "three/webgpu";

import { ERendererDebugView } from "#/contract/renderer-debug-view";
import { IRendererSettings } from "#/contract/renderer-settings";
import { FullScreenDraw } from "#/pass/full-screen-draw";
import { toPresentPassFragment } from "#/pass/present-pass.tsl";
import { createQuadMaterial } from "#/pass/quad-material";
import { IRendererFrame } from "#/pass/renderer-frame";
import { IRendererPass } from "#/pass/renderer-pass";
import { IRendererPipelines } from "#/pass/renderer-pipelines";
import { RendererTargets } from "#/pass/renderer-targets";
import { CameraUniforms } from "#/uniforms/camera-uniforms";

/**
 * Puts the chosen picture on the canvas: the frame, or one target shown raw. A picture's draw is made as it is first
 * wanted, and a capture draws it into a target of its own.
 */
export class PresentPass implements IRendererPass {
  public readonly name: string = "present";

  private readonly draws: Map<ERendererDebugView, FullScreenDraw> = new Map();
  private readonly targets: RendererTargets;
  private readonly camera: CameraUniforms;
  /** What the finished frame is read from: the tonemapped frame, or what smoothed it. */
  private shown: RenderTarget;
  /** The screen's occlusion, while it is on. */
  private ambientOcclusion: Nullable<Texture> = null;

  public constructor(targets: RendererTargets, camera: CameraUniforms) {
    this.targets = targets;
    this.camera = camera;
    this.shown = targets.scene;
  }

  /** What the finished frame is read from now. */
  public get frame(): RenderTarget {
    return this.shown;
  }

  /**
   * @param frame - What the finished frame is read from from now on.
   */
  public setFrame(frame: RenderTarget): void {
    if (frame === this.shown) {
      return;
    }

    this.shown = frame;
    this.forget(ERendererDebugView.FINAL);
  }

  /**
   * @param ambientOcclusion - The screen's occlusion its view shows from now on, or none.
   */
  public setAmbientOcclusion(ambientOcclusion: Nullable<Texture>): void {
    if (ambientOcclusion === this.ambientOcclusion) {
      return;
    }

    this.ambientOcclusion = ambientOcclusion;
    this.forget(ERendererDebugView.AMBIENT_OCCLUSION);
  }

  /** The picture the settings show, made where it was not wanted before. */
  public listPipelines(pipelines: IRendererPipelines, settings: IRendererSettings): void {
    pipelines.draw(this.getDraw(settings.debugView));
  }

  public render({ renderer, settings }: IRendererFrame): void {
    this.getDraw(settings.debugView).render(renderer);
  }

  /**
   * @param renderer - The renderer drawing.
   * @param view - The picture wanted.
   * @param target - Where it goes: the canvas when null.
   */
  public draw(renderer: WebGPURenderer, view: ERendererDebugView, target: Nullable<RenderTarget>): void {
    this.getDraw(view).render(renderer, target);
  }

  public dispose(): void {
    this.draws.forEach((draw: FullScreenDraw) => draw.dispose());
    this.draws.clear();
  }

  private getDraw(view: ERendererDebugView): FullScreenDraw {
    let draw: Nullable<FullScreenDraw> = this.draws.get(view) ?? null;

    if (!draw) {
      draw = new FullScreenDraw(
        createQuadMaterial(toPresentPassFragment(view, this.targets, this.camera, this.shown, this.ambientOcclusion)),
        null
      );
      this.draws.set(view, draw);
    }

    return draw;
  }

  /** Lets a picture's draw go, made again as it is next wanted. */
  private forget(view: ERendererDebugView): void {
    this.draws.get(view)?.dispose();
    this.draws.delete(view);
  }
}
