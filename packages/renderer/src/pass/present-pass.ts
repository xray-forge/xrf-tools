import { Nullable } from "@xrf/types";
import { NodeMaterial, QuadMesh, RenderTarget, Texture, WebGPURenderer } from "three/webgpu";

import { ERendererDebugView } from "#/contract/renderer-settings";
import { toPresentPassFragment } from "#/pass/present-pass.tsl";
import { createQuadMaterial } from "#/pass/quad-material";
import { IRendererFrame } from "#/pass/renderer-frame";
import { IRendererPass } from "#/pass/renderer-pass";
import { RendererTargets } from "#/pass/renderer-targets";
import { CameraUniforms } from "#/uniforms/camera-uniforms";

/**
 * Puts the chosen picture on the canvas: the frame, or one target shown raw.
 */
export class PresentPass implements IRendererPass {
  public readonly name: string = "present";

  private readonly quad: QuadMesh = new QuadMesh();
  private readonly materials: Map<ERendererDebugView, NodeMaterial> = new Map();
  private readonly targets: RendererTargets;
  private readonly camera: CameraUniforms;
  /** What the finished frame is read from: the tonemapped frame, or what smoothed it. */
  private frame: RenderTarget;
  /** The screen's occlusion, while it is on. */
  private ambientOcclusion: Nullable<Texture> = null;

  public constructor(targets: RendererTargets, camera: CameraUniforms) {
    this.targets = targets;
    this.camera = camera;
    this.frame = targets.scene;
  }

  /**
   * @param frame - What the finished frame is read from from now on.
   */
  public setFrame(frame: RenderTarget): void {
    if (frame === this.frame) {
      return;
    }

    this.frame = frame;
    this.materials.get(ERendererDebugView.FINAL)?.dispose();
    this.materials.delete(ERendererDebugView.FINAL);
  }

  /**
   * @param ambientOcclusion - The screen's occlusion its view shows from now on, or none.
   */
  public setAmbientOcclusion(ambientOcclusion: Nullable<Texture>): void {
    if (ambientOcclusion === this.ambientOcclusion) {
      return;
    }

    this.ambientOcclusion = ambientOcclusion;
    this.materials.get(ERendererDebugView.AMBIENT_OCCLUSION)?.dispose();
    this.materials.delete(ERendererDebugView.AMBIENT_OCCLUSION);
  }

  public render({ renderer, settings }: IRendererFrame): void {
    this.draw(renderer, settings.debugView, null);
  }

  /**
   * @param renderer - The renderer drawing.
   * @param view - The picture wanted.
   * @param target - Where it goes: the canvas when null.
   */
  public draw(renderer: WebGPURenderer, view: ERendererDebugView, target: Nullable<RenderTarget>): void {
    let material: Nullable<NodeMaterial> = this.materials.get(view) ?? null;

    if (!material) {
      material = createQuadMaterial(
        toPresentPassFragment(view, this.targets, this.camera, this.frame, this.ambientOcclusion)
      );
      this.materials.set(view, material);
    }

    this.quad.material = material;
    renderer.setRenderTarget(target);
    this.quad.render(renderer);
  }

  public dispose(): void {
    this.materials.forEach((material: NodeMaterial) => material.dispose());
    this.materials.clear();
  }
}
