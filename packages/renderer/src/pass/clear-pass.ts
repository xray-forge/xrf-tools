import { Color, OrthographicCamera, Scene, WebGPURenderer } from "three/webgpu";

import { IRendererPass } from "#/pass/renderer-pass";

/**
 * Clears the canvas to the viewport backdrop.
 */
export class ClearPass implements IRendererPass {
  public readonly name: string = "clear";

  private readonly backdrop: Color = new Color();
  private readonly scene: Scene = new Scene();
  private readonly camera: OrthographicCamera = new OrthographicCamera();

  public constructor() {
    this.scene.background = this.backdrop;
  }

  /**
   * @param backdrop - The colour, as a hex number.
   */
  public setBackdrop(backdrop: number): void {
    this.backdrop.setHex(backdrop);
  }

  public render(renderer: WebGPURenderer): void {
    renderer.setRenderTarget(null);
    renderer.render(this.scene, this.camera);
  }

  public dispose(): void {}
}
